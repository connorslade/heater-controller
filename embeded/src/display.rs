use defmt::info;
use embassy_stm32::{
    i2c::{I2c, Master},
    mode::Blocking,
};
use ssd1306::{
    Ssd1306, mode::BufferedGraphicsMode, prelude::I2CInterface, size::DisplaySize128x64,
};

use crate::{
    config::system::HEATER_POWER,
    misc::font::{CHAR_SIZE, blit_int, blit_str},
};

pub type Display<'a> = Ssd1306<
    I2CInterface<I2c<'a, Blocking, Master>>,
    DisplaySize128x64,
    BufferedGraphicsMode<DisplaySize128x64>,
>;
pub type Bounds = [(u32, u32); 2];

const POWER_METER: Bounds = [(0, 0), (6, 63)];
// const TEMPATURE_PLOT: Bounds = [(8, 32), (127, 63)];

const TEMPATURE_PANEL: Bounds = [(8, 0), (50, 30)];
const HEATER_PANEL: Bounds = [(8, 32), (50, 63)];
const PANNELS: &[Bounds] = &[POWER_METER, TEMPATURE_PANEL, HEATER_PANEL];

pub fn render<'a>(display: &mut Display<'a>, power: f32) {
    display.clear_buffer();

    {
        let n = blit_int(display, (10, 34), false, (power * HEATER_POWER) as u32);
        blit_str(display, (10 + CHAR_SIZE.0 * n, 34), false, "W");
    }

    {
        let n = blit_int(display, (10, 41), false, (power * 100.0) as u32);
        blit_str(display, (10 + CHAR_SIZE.0 * n, 41), false, "%");
    }

    {
        blit_str(display, (11, 48), true, "HEATING");
    }

    draw_progress_vertical(
        display,
        POWER_METER,
        (power * bounds_height(POWER_METER) as f32) as u32,
    );

    for panel in PANNELS {
        frame(display, *panel);
    }

    display.flush().unwrap();
}

fn draw_progress_vertical(display: &mut Display, [min, max]: Bounds, height: u32) {
    for y in (min.1..=max.1).rev().take(height as usize) {
        for x in min.0..=max.0 {
            display.set_pixel(x, y, x & 1 == y & 1);
        }
    }
}

fn frame(display: &mut Display, [min, max]: [(u32, u32); 2]) {
    for x in (min.0 + 1)..=(max.0 - 1) {
        display.set_pixel(x, min.1, true);
        display.set_pixel(x, max.1, true);
    }

    for y in (min.1 + 1)..=(max.1 - 1) {
        display.set_pixel(min.0, y, true);
        display.set_pixel(max.0, y, true);
    }

    display.set_pixel(min.0 + 1, min.1 + 1, true);
    display.set_pixel(min.0 + 1, max.1 - 1, true);
    display.set_pixel(max.0 - 1, min.1 + 1, true);
    display.set_pixel(max.0 - 1, max.1 - 1, true);
}

fn bounds_height([min, max]: Bounds) -> u32 {
    max.1 - min.1
}
