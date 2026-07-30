#![no_std]
#![no_main]

use core::panic::PanicInfo;

use cortex_m::prelude::_embedded_hal_Pwm;
use defmt::info;
use defmt_rtt as _;
use embassy_executor::{Spawner, task};
use embassy_stm32::{
    adc::{Adc, AdcChannel, Resolution, SampleTime},
    bind_interrupts,
    dma::{self},
    gpio::OutputType,
    peripherals::{DMA1_CH1, TIM1},
    time::Hertz,
    timer::{
        Channel,
        low_level::CountingMode,
        simple_pwm::{PwmPin, SimplePwm},
    },
};
use embassy_time::{Instant, Timer};
use libm::logf;
use uom::si::{
    electric_potential::millivolt,
    electrical_resistance::ohm,
    f32::{ElectricPotential, ElectricalResistance, Ratio, ThermodynamicTemperature},
    ratio::ratio,
    thermodynamic_temperature::{degree_celsius, kelvin},
};

mod config;

const VREFINT_CAL_ADDR: *const u16 = 0x1FFF756A as _;

bind_interrupts!(struct Irqs {
    DMA1_CHANNEL1 => dma::InterruptHandler<DMA1_CH1>;
});

#[task]
async fn heater(mut pwm: SimplePwm<'static, TIM1>) {
    // Ramp to 10%
    let start = Instant::now();
    let ramp_time = 60.0;
    let max_power = 0.023;

    loop {
        let t = start.elapsed().as_secs() as f32 / ramp_time;
        let power = t.clamp(0.0, 1.0) * max_power;
        pwm.set_duty(Channel::Ch1, (power * pwm.max_duty_cycle() as f32) as u32);
        Timer::after_millis(100).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let pin = PwmPin::new(p.PA0, OutputType::PushPull);
    let mut pwm = SimplePwm::new(
        p.TIM1,
        Some(pin),
        None,
        None,
        None,
        Hertz(1),
        CountingMode::EdgeAlignedUp,
    );

    pwm.set_duty(Channel::Ch1, 0);
    pwm.enable(Channel::Ch1);
    spawner.spawn(heater(pwm).unwrap());

    let mut adc = Adc::new(p.ADC1, Resolution::BITS12);
    let mut dma = p.DMA1_CH1;
    let mut input = p.PA3.degrade_adc();

    let mut vref = adc.enable_vrefint().degrade_adc();
    let vref_cal = unsafe { *VREFINT_CAL_ADDR };

    let mut readings = [0; 2];
    loop {
        adc.read(
            dma.reborrow(),
            Irqs,
            [
                (&mut input, SampleTime::CYCLES640_5),
                (&mut vref, SampleTime::CYCLES640_5),
            ]
            .into_iter(),
            &mut readings,
        )
        .await;

        let [input, vref] = readings;

        let vdd = ElectricPotential::new::<millivolt>((3000.0 * vref_cal as f32) / vref as f32);
        let input = (input as f32 / 4096 as f32) * vdd;
        let r = ElectricalResistance::new::<ohm>(config::thermister::R2)
            * (vdd / input - Ratio::new::<ratio>(1.0));
        let t = thermistor_temp(r);

        // °C, Ω
        info!("{},{}", t.get::<degree_celsius>(), r.get::<ohm>());
        Timer::after_millis(300).await;
    }
}

fn thermistor_temp(resistance: ElectricalResistance) -> ThermodynamicTemperature {
    ThermodynamicTemperature::new::<kelvin>(
        1.0 / (1.0 / config::thermister::T0
            + logf(resistance.get::<ohm>() / config::thermister::R0) / config::thermister::BETA),
    )
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
