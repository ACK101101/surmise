use image::{Rgb, RgbImage};
use std::cmp::min;
use std::fmt;

use crate::geometry::Rect;
use crate::transform::scale_rbg;

// --- PatternMode -----------------------------------------------------------------------------------
#[derive(Copy, Clone)]
pub enum PatternMode {
    Default, // full color fill
    Dots,
    // TODO: single image throughout
    // TODO: multiple images to map to
}

impl PatternMode {
    pub fn toggle(&mut self) {
        *self = match self {
            PatternMode::Default => PatternMode::Dots,
            PatternMode::Dots => PatternMode::Default,
        };
    }

    pub fn pattern_tile(
        &self,
        new_image: &mut RgbImage,
        tile: Rect,
        new_v: Rgb<u8>,
        col_i: u32,
        row_i: u32,
    ) {
        match self {
            PatternMode::Default => self.fill_rect(new_image, tile, new_v, col_i, row_i),
            PatternMode::Dots => self.fill_circle(new_image, tile, new_v, col_i, row_i),
        }
    }

    fn fill_rect(
        &self,
        new_image: &mut RgbImage,
        tile: Rect,
        new_v: Rgb<u8>,
        col_i: u32,
        row_i: u32,
    ) {
        let (pixel_width, pixel_height) = tile.get_dims();

        for x_i in (col_i * pixel_width)..(col_i + 1) * pixel_width {
            for y_i in (row_i * pixel_height)..(row_i + 1) * pixel_height {
                new_image.put_pixel(x_i, y_i, new_v);
            }
        }
    }

    fn fill_circle(
        &self,
        new_image: &mut RgbImage,
        tile: Rect,
        new_v: Rgb<u8>,
        col_i: u32,
        row_i: u32,
    ) {
        let (pixel_width, pixel_height) = tile.get_dims();
        let center_x = pixel_width / 2 + col_i * pixel_width;
        let center_y = pixel_height / 2 + row_i * pixel_height;
        let radius = (min(pixel_width, pixel_height) / 2) as f32;

        for x_i in (col_i * pixel_width)..(col_i + 1) * pixel_width {
            for y_i in (row_i * pixel_height)..(row_i + 1) * pixel_height {
                let dx_sq = x_i.abs_diff(center_x).pow(2) as f32;
                let dy_sq = y_i.abs_diff(center_y).pow(2) as f32;
                let dist = (dx_sq + dy_sq).sqrt();

                // if pixel is on geometric border, render at 0.5 alpha for anti-aliasing
                let coverage = ((radius - dist) + 0.5).clamp(0.0, 1.0);
                new_image.put_pixel(x_i, y_i, scale_rbg(new_v, coverage));
            }
        }
    }
}

impl fmt::Display for PatternMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PatternMode::Default => write!(f, "Chunky"),
            PatternMode::Dots => write!(f, "Dots"),
        }
    }
}
