pub mod average;
pub mod lattice;

use crate::window::EffectMode;
use anyhow::{Result, anyhow};
use image::{Rgb, RgbImage};

use crate::geometry::{Point, Rect};

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

pub fn downsample(
    source: &RgbImage,
    origin: Point,
    window_dims: Rect,
    pixel_dims: Rect,
    pixel_chunk_matrix: Rect,
    source_chunk_matrix: Rect,
    sampler: impl Fn(&RgbImage, Point, Rect) -> Rgb<u8>,
    mode: EffectMode,
    memory: &mut lattice::PixelLattice,
) -> RgbImage {
    let (window_width, window_height) = window_dims.get_dims();
    let (pixel_width, pixel_height) = pixel_dims.get_dims();
    let (pixel_matrix_width, pixel_matrix_height) = pixel_chunk_matrix.get_dims();
    let (source_width, source_height) = source_chunk_matrix.get_dims();
    let mut new_image: RgbImage = RgbImage::new(window_width, window_height);

    let use_memory = matches!(mode, EffectMode::Sma)
        && memory.use_memory(pixel_matrix_width as usize, pixel_matrix_height as usize);

    for row_i in 0..pixel_matrix_height {
        for col_i in 0..pixel_matrix_width {
            // get top left point of chunk of source image
            let top_left = Point {
                x: origin.x + (col_i * source_width) as i32,
                y: origin.y + (row_i * source_height) as i32,
            };

            let mut new_pixel_value = sampler(&source, top_left, source_chunk_matrix);

            if use_memory {
                new_pixel_value = memory.sma(new_pixel_value, row_i, col_i);
            }

            // fill pixel chunk with new value
            for x_i in (col_i * pixel_width)..(col_i + 1) * pixel_width {
                for y_i in (row_i * pixel_height)..(row_i + 1) * pixel_height {
                    new_image.put_pixel(x_i, y_i, new_pixel_value);
                }
            }
        }
    }

    if use_memory {
        memory.bump_write_idx();
    }

    new_image
}

pub fn rbg_image_to_u32(image: &RgbImage) -> Vec<u32> {
    let mut vector: Vec<u32> = Vec::new();
    for (_, _, pixel) in image.enumerate_pixels() {
        vector.push(rgb_to_u32(pixel.0[0], pixel.0[1], pixel.0[2]));
    }

    vector
}

fn rgb_to_u32(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

fn add_rgb(p0: Rgb<u8>, p1: Rgb<u8>) -> Rgb<u8> {
    Rgb([
        p0.0[0].saturating_add(p1.0[0]),
        p0.0[1].saturating_add(p1.0[1]),
        p0.0[2].saturating_add(p1.0[2]),
    ])
}

fn scale_rbg(pix: Rgb<u8>, numerator: usize, denominator: usize) -> Rgb<u8> {
    Rgb([
        (((pix.0[0] as usize).saturating_mul(numerator)).saturating_div(denominator)) as u8,
        (((pix.0[1] as usize).saturating_mul(numerator)).saturating_div(denominator)) as u8,
        (((pix.0[2] as usize).saturating_mul(numerator)).saturating_div(denominator)) as u8,
    ])
}

pub fn reflect_y(image: &mut RgbImage) {
    let w = image.width() as usize;
    let row_len = w * 3; // raw pixel is stored as 3 u8's
    for row in image.chunks_exact_mut(row_len) {
        for i in 0..w / 2 {
            let (p_i_start, p_j_start) = (i * 3, (w - 1 - i) * 3);
            row.swap(p_i_start, p_j_start);
            row.swap(p_i_start + 1, p_j_start + 1);
            row.swap(p_i_start + 2, p_j_start + 2);
        }
    }
}
