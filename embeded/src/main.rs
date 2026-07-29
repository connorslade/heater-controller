#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::{Spawner, task};
use embassy_stm32::{
    adc::{Adc, AdcChannel, Resolution, SampleTime},
    bind_interrupts, dma,
    gpio::{Level, Output, Speed},
    peripherals::DMA1_CH1,
};
use embassy_time::Timer;
use libm::logf;
use panic_probe as _;

const VREFINT_CAL_ADDR: *const u16 = 0x1FFF756A as _;

bind_interrupts!(struct Irqs {
    DMA1_CHANNEL1 => dma::InterruptHandler<DMA1_CH1>;
});

#[task]
async fn blink(mut led: Output<'static>) {
    loop {
        led.toggle();
        Timer::after_millis(500).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let led = Output::new(p.PA0, Level::High, Speed::Low);
    spawner.spawn(blink(led).unwrap());

    let mut adc = Adc::new(p.ADC1, Resolution::BITS12);
    let mut dma = p.DMA1_CH1;
    let mut input = p.PA3.degrade_adc();

    let mut vref = adc.enable_vrefint().degrade_adc();
    let vref_cal = unsafe { *VREFINT_CAL_ADDR };
    info!("vref_cal: {}", vref_cal);

    let mut readings = [0; 2];
    loop {
        adc.read(
            dma.reborrow(),
            Irqs,
            [
                (&mut input, SampleTime::CYCLES12_5),
                (&mut vref, SampleTime::CYCLES12_5),
            ]
            .into_iter(),
            &mut readings,
        )
        .await;

        let [input, vref] = readings;

        let vdda_mv: u32 = (3000u32 * vref_cal as u32) / vref as u32;
        info!("vdda_mv: {}", vdda_mv);

        let input = (input as f32 / 4096 as f32) * (vdda_mv as f32 / 1000.0);
        info!("Input voltage: {}", input);

        let r = 10_000.0 * ((vdda_mv as f32 / 1000.0) / input - 1.0);
        info!("Calculated resistance: {}", r);

        let temp = thermistor_temp_c(r);
        info!("Temp: {}°C ({}°F)", temp, temp * 1.8 + 32.0);

        Timer::after_millis(300).await;
    }
}

fn thermistor_temp_c(resistance: f32) -> f32 {
    const R0: f32 = 10_000.0;
    const T0: f32 = 298.15; // 25°C in Kelvin
    const BETA: f32 = 3470.0;

    let temp_k = 1.0 / (1.0 / T0 + logf(resistance / R0) / BETA);
    temp_k - 273.15
}
