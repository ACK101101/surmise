pub mod color;
pub mod cuboid;
pub mod effect;
pub mod pattern;

use image::{Rgb, RgbImage};
use rayon::prelude::*;

use crate::transform::color::ColorMode;

pub fn rbg_image_to_u32(image: &RgbImage, v: &mut Vec<u32>, color_mode: ColorMode) {
    image
        .as_raw()
        .par_chunks_exact(3)
        .map(|c| rgb_to_u32(c[0], c[1], c[2], color_mode))
        .collect_into_vec(v);
}

fn rgb_to_u32(r: u8, g: u8, b: u8, color_mode: ColorMode) -> u32 {
    match color_mode {
        ColorMode::Red => (r as u32) << 16 ,
        ColorMode::Green => (g as u32) << 8 ,
        ColorMode::Blue => b as u32 ,
        _ => ((r as u32) << 16) | ((g as u32) << 8) | (b as u32),
    }
}

pub fn scale_rbg(pix: Rgb<u8>, m: f32) -> Rgb<u8> {
    Rgb([
        ((pix.0[0] as f32) * m) as u8,
        ((pix.0[1] as f32) * m) as u8,
        ((pix.0[2] as f32) * m) as u8,
    ])
}
