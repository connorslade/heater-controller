#![no_std]
#![no_main]

use core::panic::PanicInfo;

use cortex_m::prelude::{_embedded_hal_Pwm, _embedded_hal_blocking_delay_DelayMs};
// use defmt_rtt as _;
use embassy_stm32::{
    bind_interrupts, dma,
    gpio::OutputType,
    i2c::{Config, I2c},
    peripherals::DMA1_CH1,
    time::Hertz,
    timer::{
        Channel,
        low_level::CountingMode,
        simple_pwm::{PwmPin, SimplePwm},
    },
};
use embassy_time::{Delay, Instant};
use micromath::F32Ext;
use ssd1306::{
    I2CDisplayInterface, Ssd1306, mode::DisplayConfig, prelude::DisplayRotation,
    size::DisplaySize128x64,
};

use crate::{
    config::system::SAMPLE_PEROID, control::Controller, display::render, thermometer::Thermometer,
};

mod config;
mod control;
mod display;
mod misc;
mod thermometer;

#[cortex_m_rt::entry]
unsafe fn main() -> ! {
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

    let mut i2c_config = Config::default();
    i2c_config.frequency = Hertz::khz(400);
    let i2c = I2c::new_blocking(p.I2C1, p.PB6, p.PB7, i2c_config);

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    let mut thermometer = Thermometer::new(p.ADC1, p.DMA1_CH1, p.PA3);
    let mut controller = Controller::new(37.78); // ≈100°F

    let mut cpu_usage = 0.0;
    loop {
        let start = Instant::now();
        let t = thermometer.sample();
        let power = controller.update(t, 28.2303);
        pwm.set_duty(
            Channel::Ch1,
            (pwm.max_duty_cycle() as f32 * power).round() as u32,
        );

        render(&mut display, cpu_usage, &controller, power, t);

        let wait_time = SAMPLE_PEROID
            .checked_sub(start.elapsed())
            .unwrap_or_default()
            .as_millis() as u32;
        cpu_usage = 1.0 - wait_time as f32 / SAMPLE_PEROID.as_millis() as f32;
        Delay.delay_ms(wait_time);
    }
}

bind_interrupts!(struct Irqs {
    DMA1_CHANNEL1 => dma::InterruptHandler<DMA1_CH1>;
});

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
