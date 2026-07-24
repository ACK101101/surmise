pub mod color;
pub mod effect;
pub mod cuboid;
pub mod pattern;

use image::{Rgb, RgbImage};
use rayon::prelude::*;

use crate::geometry::{Point, Rect};
use crate::transform::color::ColorMode;

pub fn average(image: &RgbImage, top_left: Point, chunk_matrix: Rect) -> Rgb<u8> {
    let (w, h) = image.dimensions();
    let (w, h) = (w as i32, h as i32);
    let (mut r, mut g, mut b) = (0u32, 0u32, 0u32);

    for x_i in top_left.x..top_left.x + chunk_matrix.get_width() as i32 {
        for y_i in top_left.y..top_left.y + chunk_matrix.get_height() as i32 {
            // image comes flipped backwards from raw, reflect here
            let x_i = w - 1 - x_i;

            // protect against grabbing pixels outside image for reveal mode
            if x_i < 0 || x_i >= w || y_i < 0 || y_i >= h {
                continue;
            }

            let pixel = image.get_pixel(x_i as u32, y_i as u32);
            r += pixel.0[0] as u32;
            g += pixel.0[1] as u32;
            b += pixel.0[2] as u32;
        }
    }

    let num_pixels = chunk_matrix.area();

    Rgb([
        (r / num_pixels) as u8,
        (g / num_pixels) as u8,
        (b / num_pixels) as u8,
    ])
}

pub fn rbg_image_to_u32(image: &RgbImage, v: &mut Vec<u32>, color_mode: ColorMode) {
    image
        .as_raw()
        .par_chunks_exact(3)
        .map(|c| rgb_to_u32(c[0], c[1], c[2], color_mode))
        .collect_into_vec(v);
}

fn rgb_to_u32(r: u8, g: u8, b: u8, color_mode: ColorMode) -> u32 {
    match color_mode {
        ColorMode::Red => ((r as u32) << 16) | ((0 as u32) << 8) | (0 as u32),
        ColorMode::Green => ((0 as u32) << 16) | ((g as u32) << 8) | (0 as u32),
        ColorMode::Blue => ((0 as u32) << 16) | ((0 as u32) << 8) | (b as u32),
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
