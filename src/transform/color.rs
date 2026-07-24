use image::{Rgb, RgbImage};
use std::fmt;

use crate::geometry::{Point, Rect};

// --- ColorMode -----------------------------------------------------------------------------------
#[derive(Copy, Clone)]
pub enum ColorMode {
    Default, // no color transform
    Gray,
    Red,
    Green,
    Blue,
    // TODO: add more granular control
    // TODO: things like auto-gen palattes based on base color?
}

impl ColorMode {
    pub fn toggle(&mut self) {
        *self = match self {
            ColorMode::Default => ColorMode::Gray,
            ColorMode::Gray => ColorMode::Red,
            ColorMode::Red => ColorMode::Green,
            ColorMode::Green => ColorMode::Blue,
            ColorMode::Blue => ColorMode::Default,
        };
    }

    // TODO: refactor?
    // averages tile then applies color transformation
    pub fn color_tile(
        &self,
        raw_image: &RgbImage,
        top_left_start: Point,
        chunk_matrix: Rect,
    ) -> Rgb<u8> {
        let (w, h) = raw_image.dimensions();
        let (w, h) = (w as i32, h as i32);
        let (mut r, mut g, mut b) = (0u32, 0u32, 0u32);

        for x_i in top_left_start.x..top_left_start.x + chunk_matrix.get_width() as i32 {
            for y_i in top_left_start.y..top_left_start.y + chunk_matrix.get_height() as i32 {
                // image comes flipped backwards from raw, reflect here
                let x_i = w - 1 - x_i;

                // protect against grabbing pixels outside image for reveal mode
                if x_i < 0 || x_i >= w || y_i < 0 || y_i >= h {
                    continue;
                }

                let pixel = raw_image.get_pixel(x_i as u32, y_i as u32);
                r += pixel.0[0] as u32;
                g += pixel.0[1] as u32;
                b += pixel.0[2] as u32;
            }
        }

        let num_pixels = chunk_matrix.area();

        let mean = ((r + g + b) / num_pixels / 3) as u8;
        let (r, g, b) = (
            (r / num_pixels) as u8,
            (g / num_pixels) as u8,
            (b / num_pixels) as u8,
        );

        match self {
            ColorMode::Default => Rgb([r, g, b]),
            ColorMode::Gray => Rgb([mean, mean, mean]),
            ColorMode::Red => Rgb([r, 0, 0]),
            ColorMode::Green => Rgb([0, g, 0]),
            ColorMode::Blue => Rgb([0, 0, b]),
        }
    }
}

impl fmt::Display for ColorMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColorMode::Default => write!(f, "Default"),
            ColorMode::Gray => write!(f, "Gray"),
            ColorMode::Red => write!(f, "Red"),
            ColorMode::Green => write!(f, "Green"),
            ColorMode::Blue => write!(f, "Blue"),
        }
    }
}
