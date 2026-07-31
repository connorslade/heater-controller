use crate::display::Display;

pub const CHAR_SIZE: (u32, u32) = (5, 5);
include!(concat!(env!("OUT_DIR"), "/font.rs"));

fn font_index(chr: char) -> Option<u32> {
    Some(match chr {
        '0'..='9' => chr as u32 - '0' as u32,
        'A'..='Z' => chr as u32 - 'A' as u32 + 10,
        '.' => 36,
        '°' => 37,
        '%' => 38,
        _ => return None,
    })
}

fn font_bit(bit: u32) -> bool {
    let (byte, bit) = (bit / 8, bit % 8);
    FONT[byte as usize] >> bit & 1 == 1
}

pub fn blit_char(display: &mut Display, pos: (u32, u32), invert: bool, chr: char) {
    let Some(char_index) = font_index(chr) else {
        return;
    };

    for y in 0..CHAR_SIZE.1 {
        for x in 0..CHAR_SIZE.0 {
            let bit_index = (CHAR_SIZE.0 * CHAR_SIZE.1) * char_index + y * CHAR_SIZE.0 + x;
            display.set_pixel(x + pos.0, y + pos.1, font_bit(bit_index) ^ invert);
        }
    }

    if invert {
        for y in 0..CHAR_SIZE.1 {
            display.set_pixel(pos.0 - 1, y + pos.1, true);
            display.set_pixel(pos.0 + CHAR_SIZE.0, y + pos.1, true);
        }

        for x in 0..CHAR_SIZE.0 {
            display.set_pixel(x + pos.0, pos.1 - 1, true);
            display.set_pixel(x + pos.0, pos.1 + CHAR_SIZE.1, true);
        }
    }
}

pub fn blit_str(display: &mut Display, origin: (u32, u32), invert: bool, str: &str) {
    let mut pos = origin;

    for chr in str.chars() {
        if chr == '\n' {
            pos.0 = origin.0;
            pos.1 += CHAR_SIZE.1;
        }

        blit_char(display, pos, invert, chr);
        pos.0 += CHAR_SIZE.0;
    }
}

pub fn blit_int(display: &mut Display, mut origin: (u32, u32), invert: bool, int: u32) -> u32 {
    let digits = if int == 0 { 1 } else { int.ilog10() + 1 };
    for i in (0..digits).rev() {
        let chr = ('0' as u8 + (int / 10u32.pow(i) % 10) as u8) as char;
        blit_char(display, origin, invert, chr);
        origin.0 += CHAR_SIZE.0;
    }

    digits
}
