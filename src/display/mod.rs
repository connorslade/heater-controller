use core::ops::{Add, AddAssign, Sub};

use embassy_stm32::{
    i2c::{I2c, Master},
    mode::Blocking,
};
use micromath::F32Ext;
use ssd1306::{
    Ssd1306, mode::BufferedGraphicsMode, prelude::I2CInterface, size::DisplaySize128x64,
};

use crate::control::Controller;

mod font;
mod ui;

pub type Display<'i> = Ssd1306<
    I2CInterface<I2c<'i, Blocking, Master>>,
    DisplaySize128x64,
    BufferedGraphicsMode<DisplaySize128x64>,
>;

struct Ui<'d, 'i> {
    display: &'d mut Display<'i>,
    bounds: Bounds,
}

#[derive(Clone, Copy)]
struct Vec2 {
    x: u32,
    y: u32,
}

#[derive(Clone, Copy)]
struct Bounds {
    min: Vec2,
    max: Vec2,
}

impl<'d, 'i> Ui<'d, 'i> {
    pub fn new(display: &'d mut Display<'i>) -> Self {
        let (width, height) = display.dimensions();
        Self {
            display,
            bounds: Bounds {
                min: Vec2::new(0, 0),
                max: Vec2::new(width as u32, height as u32),
            },
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, value: bool) {
        let pos = self.bounds.min + Vec2::new(x, y);
        if self.bounds.contains(&pos) {
            self.display.set_pixel(pos.x, pos.y, value);
        }
    }

    pub fn space(&mut self, px: Vec2) {
        self.bounds.min += px;
    }

    pub fn space_y(&mut self, px: u32) {
        self.bounds.min.y += px;
    }

    pub fn frame(&mut self, bounds: Bounds, ui: impl FnOnce(&mut Self)) {
        let Bounds { min, max } = bounds;

        let old_bounds = self.bounds;
        self.bounds = bounds.contract(1);
        ui(self);
        self.bounds = old_bounds;

        for x in (min.x + 1)..=(max.x - 1) {
            self.set_pixel(x, min.y, true);
            self.set_pixel(x, max.y, true);
        }

        for y in (min.y + 1)..=(max.y - 1) {
            self.set_pixel(min.x, y, true);
            self.set_pixel(max.x, y, true);
        }

        self.set_pixel(min.x + 1, min.y + 1, true);
        self.set_pixel(min.x + 1, max.y - 1, true);
        self.set_pixel(max.x - 1, min.y + 1, true);
        self.set_pixel(max.x - 1, max.y - 1, true);
    }

    pub fn draw_progress_vertical(&mut self, height: u32) {
        for (i, y) in (0..=self.bounds.height())
            .rev()
            .take(height as usize)
            .enumerate()
        {
            if i + 1 == height as usize {
                for x in 0..=self.bounds.width() {
                    self.set_pixel(x, y, true);
                }
            } else {
                for x in 0..=self.bounds.width() {
                    self.set_pixel(x, y, x & 1 == y & 1);
                }
            }
        }
    }

    pub fn draw_chart<const N: usize>(&mut self, (min, max): (f32, f32), data: [f32; N]) {
        let width = self.bounds.width();
        for x in 0..width {
            let t = x as f32 / (width - 1) as f32;
            let n = N as f32 * t;
            let nf = n.fract();

            let (n1, n2) = (n.floor() as usize, n.ceil() as usize);
            let value = data[n1.min(N - 1)] * (1.0 - nf) + data[n2.min(N - 1)] * nf;

            let y = (1.0 - (value - min) / (max - min)) * self.bounds.height() as f32;
            self.set_pixel(x, y as u32, true);
        }
    }

    pub fn draw_line_horizontal(&mut self, (x, y): (u32, u32), width: u32) {
        for i in 0..width {
            self.set_pixel(x + i, y, i & 1 != 0);
        }
    }
}

impl Vec2 {
    pub const fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    pub const fn splat(val: u32) -> Self {
        Self { x: val, y: val }
    }
}

impl Bounds {
    pub const fn new(min: (u32, u32), max: (u32, u32)) -> Self {
        Self {
            min: Vec2::new(min.0, min.1),
            max: Vec2::new(max.0, max.1),
        }
    }

    pub fn contract(&self, delta: u32) -> Self {
        Self {
            min: self.min + Vec2::splat(delta),
            max: self.max - Vec2::splat(delta),
        }
    }

    pub const fn contains(&self, pos: &Vec2) -> bool {
        pos.x <= self.max.x && pos.x >= self.min.x && pos.y <= self.max.y && pos.y >= self.min.y
    }

    pub const fn width(&self) -> u32 {
        self.max.x - self.min.x
    }

    pub const fn height(&self) -> u32 {
        self.max.y - self.min.y
    }
}

impl Add<Vec2> for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Vec2) -> Self::Output {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign<Vec2> for Vec2 {
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub<Vec2> for Vec2 {
    type Output = Vec2;

    fn sub(self, rhs: Vec2) -> Self::Output {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

pub fn render<'a>(
    display: &mut Display<'a>,
    cpu_usage: f32,
    controller: &Controller,
    power: f32,
    temp: f32,
) {
    display.clear_buffer();

    let mut ui = Ui::new(display);
    ui::ui(&mut ui, cpu_usage, controller, power, temp);

    ui.display.flush().unwrap();
}
