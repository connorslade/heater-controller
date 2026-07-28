#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use {defmt_serial as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    config.enable_debug_during_sleep = true;
    let p = embassy_stm32::init(config);
    info!("Hello World!");

    let mut led = Output::new(p.PA0, Level::High, Speed::Low);

    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(200).await;

        info!("low");
        led.set_low();
        Timer::after_millis(200).await;
    }
}
