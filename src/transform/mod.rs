pub mod average;
pub mod lattice;

use std::mem;

use crate::window::Mode;
use anyhow::{Result, anyhow};
use image::{Rgb, RgbImage};

#[derive(Copy, Clone)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl From<(isize, isize)> for Point {
    fn from((x, y): (isize, isize)) -> Self {
        Point {
            x: x as i32,
            y: y as i32,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Rect {
    pub width: u32,
    pub height: u32,
}

impl From<(usize, usize)> for Rect {
    fn from((width, height): (usize, usize)) -> Self {
        Rect {
            width: width as u32,
            height: height as u32,
        }
    }
}

pub fn calc_source_chunk_dims(
    source_dims: Rect,
    window_dims: Rect,
    window_pos: Point,
    pixel_dims: Rect,
    mode: Mode,
) -> Result<(Rect, Rect, Point)> {
    if window_dims.width > source_dims.width || window_dims.height > source_dims.height {
        return Err(anyhow!(
            "Can not downsample when the source is smaller than window bruh"
        ));
    }

    // figure out how many chunky pixels fit into the target
    let num_y_pixels: u32 = window_dims.height / pixel_dims.height;
    let num_x_pixels: u32 = window_dims.width / pixel_dims.width;
    let pixel_chunk_matrix: Rect = Rect {
        width: num_x_pixels,
        height: num_y_pixels,
    };

    let relevant_source_dims: Rect = match mode {
        Mode::Reveal => window_dims,
        _ => source_dims,
    };

    // based on matrix of chunky pixels, map source chunks to pixel chunks
    let source_chunk_width: u32 = relevant_source_dims.width / num_x_pixels;
    let source_chunk_height: u32 = relevant_source_dims.height / num_y_pixels;
    let source_chunk_dims: Rect = Rect {
        width: source_chunk_width,
        height: source_chunk_height,
    };

    // where to start processing source image
    let origin: Point = match mode {
        Mode::Reveal => window_pos,
        _ => Point { x: 0, y: 0 },
    };

    Ok((pixel_chunk_matrix, source_chunk_dims, origin))
}

pub fn downsample(
    source: RgbImage,
    origin: Point,
    window_dims: Rect,
    pixel_dims: Rect,
    pixel_chunk_matrix: Rect,
    source_chunk_dims: Rect,
    sampler: impl Fn(&RgbImage, Point, Rect) -> Rgb<u8>,
    mode: Mode,
    memory: &mut lattice::PixelLattice,
) -> RgbImage {
    let mut new_image: RgbImage = RgbImage::new(window_dims.width, window_dims.height);

    let use_memory = matches!(mode, Mode::Sma)
        && memory.use_memory(
            pixel_chunk_matrix.width as usize,
            pixel_chunk_matrix.height as usize,
        );

    for row_i in 0..pixel_chunk_matrix.height {
        for col_i in 0..pixel_chunk_matrix.width {
            // get top left point of chunk of source image
            let top_left = Point {
                x: origin.x + (col_i * source_chunk_dims.width) as i32,
                y: origin.y + (row_i * source_chunk_dims.height) as i32,
            };

            let mut new_pixel_value = sampler(&source, top_left, source_chunk_dims);

            if use_memory {
                new_pixel_value = memory.sma(new_pixel_value, row_i, col_i);
            }

            // fill pixel chunk with new value
            for x_i in (col_i * pixel_dims.width)..(col_i + 1) * pixel_dims.width {
                for y_i in (row_i * pixel_dims.height)..(row_i + 1) * pixel_dims.height {
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

pub fn reflect_y(image: &RgbImage) -> RgbImage {
    let (w, h) = image.dimensions();
    let mut new_image = RgbImage::new(w, h);
    for (x, y, p) in image.enumerate_pixels() {
        new_image.put_pixel(w - x - 1, y, *p);
    }

    new_image
}
