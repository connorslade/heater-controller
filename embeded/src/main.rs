#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let mut pin = Output::new(p.PA0, Level::High, Speed::Low);

    loop {
        Timer::after_millis(500).await;
        pin.set_high();
        Timer::after_millis(500).await;
        pin.set_low();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
