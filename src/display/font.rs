use crate::display::{Ui, Vec2};

pub const CHAR_SIZE: Vec2 = Vec2::new(5, 5);
include!(concat!(env!("OUT_DIR"), "/font.rs"));

fn font_index(chr: char) -> Option<u32> {
    Some(match chr {
        '0'..='9' => chr as u32 - '0' as u32,
        'A'..='Z' => chr as u32 - 'A' as u32 + 10,
        '.' => 36,
        '°' => 37,
        '%' => 38,
        ':' => 39,
        '\u{E000}'.. => 40 + chr as u32 - '\u{E000}' as u32,
        _ => return None,
    })
}

fn font_bit(bit: u32) -> bool {
    let (byte, bit) = (bit / 8, bit % 8);
    FONT[byte as usize] >> bit & 1 == 1
}

pub fn blit_char(ui: &mut Ui, pos: Vec2, invert: bool, chr: char) {
    let Some(char_index) = font_index(chr) else {
        return;
    };

    for y in 0..CHAR_SIZE.y {
        for x in 0..CHAR_SIZE.x {
            let bit_index = (CHAR_SIZE.x * CHAR_SIZE.y) * char_index + y * CHAR_SIZE.x + x;
            ui.set_pixel(x + pos.x, y + pos.y, font_bit(bit_index) ^ invert);
        }
    }

    if invert {
        for y in 0..CHAR_SIZE.y {
            ui.set_pixel(pos.x - 1, y + pos.y, true);
            ui.set_pixel(pos.x + CHAR_SIZE.x, y + pos.y, true);
        }

        for x in 0..CHAR_SIZE.x {
            ui.set_pixel(x + pos.x, pos.y - 1, true);
            ui.set_pixel(x + pos.x, pos.y + CHAR_SIZE.y, true);
        }
    }
}

impl<'d, 'i> Ui<'d, 'i> {
    fn _draw_string(&mut self, (x, y): (u32, u32), invert: bool, str: &str) {
        let origin = Vec2::new(x, y);
        let mut pos = origin;

        for chr in str.chars() {
            if chr == '\n' {
                pos.x = origin.x;
                pos.y += CHAR_SIZE.y;
            }

            blit_char(self, pos, invert, chr);
            pos.x += CHAR_SIZE.x;
        }
    }

    pub fn draw_string(&mut self, pos: (u32, u32), str: &str) {
        self._draw_string(pos, false, str);
    }

    pub fn draw_string_inverted(&mut self, pos: (u32, u32), str: &str) {
        self._draw_string(pos, true, str);
    }

    pub fn draw_int(&mut self, (x, y): (u32, u32), int: u32) -> u32 {
        let mut origin = Vec2::new(x, y);
        let digits = if int == 0 { 1 } else { int.ilog10() + 1 };

        for i in (0..digits).rev() {
            let chr = (b'0' + (int / 10u32.pow(i) % 10) as u8) as char;
            blit_char(self, origin, false, chr);
            origin.x += CHAR_SIZE.x;
        }

        digits
    }

    pub fn draw_value(&mut self, (x, y): (u32, u32), prefix: &str, suffix: &str, value: u32) {
        self.draw_string((x, y), prefix);
        let mut n = prefix.chars().count() as u32;
        n += self.draw_int((x + CHAR_SIZE.x * n, y), value);
        self.draw_string((x + CHAR_SIZE.x * n, y), suffix);
    }
}
