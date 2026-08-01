use std::{env, fs, path::PathBuf};

use bitvec::{order::Lsb0, vec::BitVec};
use image::{GenericImageView, ImageReader};

const CHAR_SIZE: (u32, u32) = (5, 5);

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    println!("cargo::rerun-if-changed=font.png");
    fs::write(out_dir.join("font.rs"), generate_font_data()).unwrap();

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    // println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
}

fn generate_font_data() -> String {
    let img = ImageReader::open("font.png").unwrap().decode().unwrap();
    assert_eq!(img.height(), CHAR_SIZE.0);
    let count = img.width() / CHAR_SIZE.1;

    let mut out = BitVec::<u8, Lsb0>::new();
    for i in 0..count {
        for y in 0..CHAR_SIZE.1 {
            for x in 0..CHAR_SIZE.0 {
                let value = img.get_pixel(x + CHAR_SIZE.0 * i, y);
                let set = value.0[0..3] == [255, 255, 255];
                out.push(set);
            }
        }
    }

    let mut code = String::new();
    code.push_str("const FONT: &[u8] = &[");
    for byte in out.as_raw_slice() {
        code.push_str(&format!("0x{byte:X}, "));
    }
    code.push_str("];");

    code
}
