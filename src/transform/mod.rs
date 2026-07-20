pub mod lattice;

use anyhow::{Result, anyhow};
use image::{Rgb, RgbImage};
use rayon::prelude::*;

use crate::geometry::{Point, Rect};
use crate::window::EffectMode;

// Transform Mapping Modes
#[derive(Copy, Clone)]
pub enum TransformMode {
    Default, // chunky pixel
    Monochrome,
    Single,
    Multiple,
    Dots,
}

impl TransformMode {
    fn toggle(&mut self) {
        *self = match self {
            TransformMode::Default => TransformMode::Monochrome,
            TransformMode::Monochrome => TransformMode::Dots,
            TransformMode::Dots => TransformMode::Default,
            _ => TransformMode::Default, // TODO: placeholder
        };
    }
}

// TODO: maybe fold in window.rs for clarity
pub fn calc_source_chunk_dims(
    source_dims: Rect,
    window_dims: Rect,
    window_pos: Point,
    pixel_dims: Rect,
    mode: EffectMode,
) -> Result<(Rect, Rect, Point)> {
    if !source_dims.can_contain(&window_dims) {
        return Err(anyhow!(
            "Can not downsample when the source is smaller than window bruh"
        ));
    }

    // figure out how many chunky pixels fit into the target
    let pixel_chunk_matrix = window_dims / pixel_dims;

    let relevant_source_matrix: Rect = match mode {
        EffectMode::Reveal => window_dims,
        _ => source_dims,
    };

    // based on matrix of chunky pixels, map source chunks to pixel chunks
    let source_chunk_matrix = relevant_source_matrix / pixel_chunk_matrix;

    // where to start processing source image
    let origin: Point = match mode {
        EffectMode::Reveal => window_pos,
        _ => Point { x: 0, y: 0 },
    };

    Ok((pixel_chunk_matrix, source_chunk_matrix, origin))
}

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

pub fn rbg_image_to_u32(image: &RgbImage, v: &mut Vec<u32>) {
    image
        .as_raw()
        .par_chunks_exact(3)
        .map(|c| rgb_to_u32(c[0], c[1], c[2]))
        .collect_into_vec(v);
}

fn rgb_to_u32(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}
