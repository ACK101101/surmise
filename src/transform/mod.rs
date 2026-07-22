pub mod lattice;

use anyhow::{Result, anyhow};
use image::{Rgb, RgbImage};
use rayon::prelude::*;
use std::fmt;

use crate::geometry::{Point, Rect};
use crate::window::EffectMode;

// Transform Mapping Modes
#[derive(Copy, Clone)]
pub enum TransformMode {
    Default, // chunky pixel
    Red,
    Green,
    Blue,
    Single,
    Multiple,
    Dots,
}

impl TransformMode {
    pub fn toggle(&mut self) {
        *self = match self {
            TransformMode::Default => TransformMode::Red,
            TransformMode::Red => TransformMode::Green,
            TransformMode::Green => TransformMode::Blue,
            TransformMode::Blue => TransformMode::Dots,
            TransformMode::Dots => TransformMode::Default,
            _ => TransformMode::Default, // TODO: placeholder
        };
    }
}

impl fmt::Display for TransformMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransformMode::Default => write!(f, "Chunky"),
            TransformMode::Red => write!(f, "Red"),
            TransformMode::Green => write!(f, "Green"),
            TransformMode::Blue => write!(f, "Blue"),
            TransformMode::Dots => write!(f, "Dots"),
            _ => write!(f, "Unimplemented"),
        }
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

pub fn rbg_image_to_u32(image: &RgbImage, v: &mut Vec<u32>, transform_mode: TransformMode) {
    image
        .as_raw()
        .par_chunks_exact(3)
        .map(|c| rgb_to_u32(c[0], c[1], c[2], transform_mode))
        .collect_into_vec(v);
}

fn rgb_to_u32(r: u8, g: u8, b: u8, transform_mode: TransformMode) -> u32 {
    match transform_mode {
        TransformMode::Red => ((r as u32) << 16) | ((0 as u32) << 8) | (0 as u32),
        TransformMode::Green => ((0 as u32) << 16) | ((g as u32) << 8) | (0 as u32),
        TransformMode::Blue => ((0 as u32) << 16) | ((0 as u32) << 8) | (b as u32),
        _ => ((r as u32) << 16) | ((g as u32) << 8) | (b as u32),
    }
}
