use super::{Point, Rect};
use image::{Rgb, RgbImage};

pub fn average(image: &RgbImage, top_left: Point, chunk_dims: Rect) -> Rgb<u8> {
    let (mut r, mut g, mut b) = (0u32, 0u32, 0u32);
    for x_i in top_left.x..top_left.x + chunk_dims.width {
        for y_i in top_left.y..top_left.y + chunk_dims.height {
            // log::debug!("Getting pixel at ({}, {})", x_i, y_i);
            let pixel = image.get_pixel(x_i, y_i);
            r += pixel.0[0] as u32;
            g += pixel.0[1] as u32;
            b += pixel.0[2] as u32;
        }
    }

    let num_pixels: u32 = chunk_dims.width * chunk_dims.height;

    Rgb([
        (r / num_pixels) as u8,
        (g / num_pixels) as u8,
        (b / num_pixels) as u8,
    ])
}
