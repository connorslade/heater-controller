#![no_std]
#![no_main]

use core::panic::PanicInfo;

use cortex_m::prelude::_embedded_hal_Pwm;
use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts, dma,
    gpio::OutputType,
    peripherals::DMA1_CH1,
    time::Hertz,
    timer::{
        Channel,
        low_level::CountingMode,
        simple_pwm::{PwmPin, SimplePwm},
    },
};
use embassy_time::Ticker;
use micromath::F32Ext;

use crate::{
    config::system::{HEATER_POWER, SAMPLE_PEROID},
    control::Controller,
    thermometer::Thermometer,
};

mod config;
mod control;
mod misc;
mod thermometer;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
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

    pwm.set_period_secs(5);
    pwm.set_duty(Channel::Ch1, 0);
    pwm.enable(Channel::Ch1);

    let mut ticker = Ticker::every(SAMPLE_PEROID);
    let mut thermometer = Thermometer::new(p.ADC1, p.DMA1_CH1, p.PA3);
    let mut controller = Controller::new(37.78); // ≈100°F

    loop {
        let t = thermometer.sample().await;
        let power = controller.update(t, 28.2303);
        pwm.set_duty(
            Channel::Ch1,
            (pwm.max_duty_cycle() as f32 * power).round() as u32,
        );

        info!(
            "T={}°C ({}°F) P={}W ({}%) E={}",
            t,
            t * 1.8 + 32.0,
            power * HEATER_POWER,
            power * 100.0,
            controller.start.elapsed().as_millis()
        );

        ticker.next().await;
    }
}

bind_interrupts!(struct Irqs {
    DMA1_CHANNEL1 => dma::InterruptHandler<DMA1_CH1>;
});

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
