use minifb::*;

mod camera;
use camera::{*};

mod transform;
use transform::{*};

fn main() {
    env_logger::init(); 

    let mut camera = match Cam::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Camera oopsie: {e}");
            return;
        }
    };

    const WINDOW_WIDTH: usize = 960;
    const WINDOW_HEIGHT: usize = 540;
    log::debug!("Window Dims: ({}, {})", WINDOW_WIDTH, WINDOW_HEIGHT);

    let mut pixel_dims: Rect = Rect {
        width: 32,
        height: 16,
    };
    log::debug!("Pixel Chunk Dims: ({}, {})", pixel_dims.width, pixel_dims.height);

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

        if win.is_key_pressed(Key::Down, KeyRepeat::No) && pixel_dims.height > 1 {
            pixel_dims.height = pixel_dims.height / 2;
        } else if win.is_key_pressed(Key::Up, KeyRepeat::No) && pixel_dims.height < window_dims.height/2 {
            pixel_dims.height = pixel_dims.height * 2; 
        }
        if win.is_key_pressed(Key::Right, KeyRepeat::No) && pixel_dims.width > 1 {
            pixel_dims.width = pixel_dims.width / 2;
        } else if win.is_key_pressed(Key::Left, KeyRepeat::No) && pixel_dims.width < window_dims.width/2 {
            pixel_dims.width = pixel_dims.width * 2; 
        }

        let new_camera_buffer = match camera.next() {
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
            source_dims, window_dims as Rect, pixel_dims,
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
            window_dims, pixel_dims,
            chunk_matrix, chunk_dims, 
            average::average,
        );

        let update_buffer = rbg_image_to_u32(&downsampled);
        
        win.update_with_buffer(
            update_buffer.as_slice(), 
            win_width, win_height,
        ).unwrap();
    }
}