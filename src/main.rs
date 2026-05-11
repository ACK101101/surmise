use anyhow::{Result, anyhow};
use minifb::*;
use image::{Rgb, RgbImage};
use nokhwa::{Camera, pixel_format::RgbFormat, utils::*};

#[derive(Copy, Clone)]
struct Point {
    x: u32,
    y: u32,
}

#[derive(Copy, Clone)]
struct Rect {
    width: u32,
    height: u32,
}

fn main() {
    env_logger::init(); 

    let camera_index = CameraIndex::Index(0);
    let requested_format = RequestedFormat::new::<RgbFormat>(
        RequestedFormatType::AbsoluteHighestFrameRate,
    );

    // my camera is 1920 x 1080, 30fps
    let mut camera = Camera::new(camera_index, requested_format).unwrap();
    
    // tries to open camera stream
    match camera.open_stream() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Sum fucked up with opening stream: {e}");
            return;
        }
    };

    const WINDOW_WIDTH: usize = 960;
    const WINDOW_HEIGHT: usize = 540;
    // const WINDOW_DIMS: Rect = Rect {
    //     width: (WINDOW_WIDTH) as u32, 
    //     height: (WINDOW_HEIGHT) as u32,
    // };
    // log::debug!("Window Dims: ({}, {})", WINDOW_DIMS.width, WINDOW_DIMS.height);


    const PIXEL_DIMS: Rect = Rect {
        width: 32,
        height: 16,
    };
    log::debug!("Pixel Chunk Dims: ({}, {})", PIXEL_DIMS.width, PIXEL_DIMS.height);

    let mut win = Window::new(
        "surmise", 
        WINDOW_WIDTH, WINDOW_HEIGHT, 
        WindowOptions { 
            borderless: true, resize: true, transparency: true, ..WindowOptions::default()
        },
    ).unwrap(); 

    while win.is_open() && !win.is_key_down(Key::Escape) {
        let (win_width, win_height) = win.get_size();
        let window_dims: Rect = Rect {
            width: win_width as u32, 
            height: win_height as u32,
        };

        let new_camera_buffer = match get_new_camera_buffer(&mut camera) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Camera oopsie: {e}");
                return;
            }
        };

        let source_dims: Rect = Rect {
            width: new_camera_buffer.width(), 
            height: new_camera_buffer.height(),
        };
        log::debug!("Og Image Dims: ({}, {})", source_dims.width, source_dims.height);

        let (chunk_matrix, chunk_dims) = match calc_source_chunk_dims(
            source_dims, window_dims as Rect, PIXEL_DIMS,
        ) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Chunking oopsie: {e}");
                return;
            }
        };
        log::debug!("Chunk Dims: ({}, {})", chunk_dims.width, chunk_dims.height);
        log::debug!("Chunk Matrix: ({}, {})", chunk_matrix.width, chunk_matrix.height);

        let downsampled = downsample(
            new_camera_buffer, 
            window_dims, PIXEL_DIMS,
            chunk_matrix, chunk_dims, 
            average,
        );

        let update_buffer = rbg_image_to_u32(&downsampled);
        
        win.update_with_buffer(
            update_buffer.as_slice(), 
            win_width, win_height,
        ).unwrap();
    }
}

fn get_new_camera_buffer(camera: &mut Camera) -> Result<RgbImage> {
    // get a frame
    let frame = match camera.frame() {
        Ok(f) => f,
        Err(e) => {
            return Err(anyhow!("Sum fucked up with getting a frame: {e}"));
        }
    };

    // decode into an ImageBuffer
    let decoded = match frame.decode_image::<RgbFormat>() {
        Ok(b) => b,
        Err(e) => {
            return Err(anyhow!("Sum fucked up with decoding buffer: {e}"));
        }
    };

    Ok(decoded)
}

fn rgb_to_u32(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

fn rbg_image_to_u32(image: &RgbImage) -> Vec<u32> {
    let mut vector: Vec<u32> = Vec::new();
    for (_, _, pixel) in image.enumerate_pixels() {
        vector.push(rgb_to_u32(pixel.0[0], pixel.0[1], pixel.0[2]));
    }

    vector
}

fn calc_source_chunk_dims(
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

fn downsample(
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

fn average(image: &RgbImage, top_left: Point, chunk_dims: Rect) -> Rgb<u8> {
    let (mut r, mut g, mut b) = (0u32, 0u32, 0u32);
    for x_i in top_left.x..top_left.x+chunk_dims.width {
        for y_i in top_left.y..top_left.y+chunk_dims.height {
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
