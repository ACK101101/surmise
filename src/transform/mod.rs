pub mod average;

use anyhow::{Result, anyhow};
use image::{Rgb, RgbImage};

#[derive(Copy, Clone)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

#[derive(Copy, Clone)]
pub struct Rect {
    pub width: u32,
    pub height: u32,
}

pub fn calc_source_chunk_dims(
    source_dims: Rect, target_dims: Rect, pixel_dims: Rect, 
) -> Result<(Rect, Rect)> {
    if target_dims.width > source_dims.width || target_dims.height > source_dims.height {
        return Err(anyhow!("Can not downsample when the source is smaller than target bruh"));
    }

    // figure out how many chunky pixels fit into the target
    let num_y_pixels: u32 = target_dims.height / pixel_dims.height;
    let num_x_pixels: u32 = target_dims.width / pixel_dims.width;
    let chunk_matrix: Rect = Rect {
        width: num_x_pixels,
        height: num_y_pixels,
    };

    // based on matrix of chunky pixels, map source chunk
    let source_chunk_width: u32 = source_dims.width / num_x_pixels;
    let source_chunk_height: u32 = source_dims.height / num_y_pixels; 
    let chunk_dims: Rect = Rect { 
        width: source_chunk_width, 
        height: source_chunk_height,
    };

    Ok((chunk_matrix, chunk_dims))
}

pub fn downsample(
    source: RgbImage, 
    window_dims: Rect, pixel_dims: Rect,
    chunk_matrix: Rect, chunk_dims: Rect, 
    sampler: impl Fn(&RgbImage, Point, Rect) -> Rgb<u8>,
) -> RgbImage {
    let mut new_image: RgbImage = RgbImage::new(window_dims.width, window_dims.height);

    for col_i in 0..chunk_matrix.width {
        for row_i in 0..chunk_matrix.height {
            // get top left point of chunk of source image
            let top_left = Point {
                x: col_i*chunk_dims.width,
                y: row_i*chunk_dims.height,
            };

            let new_pixel_value = sampler(&source, top_left, chunk_dims);

            // fill pixel chunk with new value
            for x_i in (col_i*pixel_dims.width)..(col_i+1)*pixel_dims.width {
                for y_i in (row_i*pixel_dims.height)..(row_i+1)*pixel_dims.height {
                    // log::debug!("Putting pixel at ({}, {})", x_i, y_i);
                    new_image.put_pixel(x_i, y_i, new_pixel_value); 
                }
            }
        }
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