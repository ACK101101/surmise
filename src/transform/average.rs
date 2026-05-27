use super::{Point, Rect};
use image::{Rgb, RgbImage};

pub fn average(image: &RgbImage, top_left: Point, chunk_dims: Rect) -> Rgb<u8> {
    let (w, h) = image.dimensions();
    let (w, h) = (w as i32, h as i32);
    let (mut r, mut g, mut b) = (0u32, 0u32, 0u32);
    for x_i in top_left.x..top_left.x + chunk_dims.width as i32 {
        for y_i in top_left.y..top_left.y + chunk_dims.height as i32 {
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

    let num_pixels: u32 = chunk_dims.width * chunk_dims.height;

    Rgb([
        (r / num_pixels) as u8,
        (g / num_pixels) as u8,
        (b / num_pixels) as u8,
    ])
}
