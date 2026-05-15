use minifb::*;

use anyhow::{Result};
use image::{RgbImage};
use crate::transform::{Rect, calc_source_chunk_dims, downsample, rbg_image_to_u32, average};


pub struct Win {
    window: Window,
    window_dims: Rect,
    pixel_chunk: Rect,
}

impl Win {
    pub fn new() -> Result<Win> {
        const WINDOW_WIDTH: usize = 960;
        const WINDOW_HEIGHT: usize = 540;
        log::debug!("Window Dims: ({}, {})", WINDOW_WIDTH, WINDOW_HEIGHT);

        let pixel_chunk: Rect = Rect {
            width: 32,
            height: 16,
        };
        log::debug!("Pixel Chunk Dims: ({}, {})", pixel_chunk.width, pixel_chunk.height);

        let window = Window::new(
            "surmise", 
            WINDOW_WIDTH, WINDOW_HEIGHT, 
            WindowOptions { 
                borderless: true, resize: true, transparency: true, ..WindowOptions::default()
            },
        )?; 

        Ok( Win {
            window,
            window_dims: Rect { width: WINDOW_WIDTH as u32, height: WINDOW_HEIGHT as u32 },
            pixel_chunk
        })
    }

    pub fn step(&mut self, raw_buf: RgbImage) -> bool {
        if self.should_close() {
            return false;
        }

        self.update_win_and_pix_size();

        let raw_dims: Rect = Rect {
            width: raw_buf.width(), 
            height: raw_buf.height(),
        };
        log::debug!("Raw Image Dims: ({}, {})", raw_dims.width, raw_dims.height);

        let (chunk_matrix, chunk_dims) = match calc_source_chunk_dims(
            raw_dims, self.window_dims, self.pixel_chunk,
        ) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Chunking oopsie: {e}");
                return true;
            }
        };
        log::debug!("Chunk Dims: ({}, {})", chunk_dims.width, chunk_dims.height);
        log::debug!("Chunk Matrix: ({}, {})", chunk_matrix.width, chunk_matrix.height);

        let downsampled = downsample(
            raw_buf, 
            self.window_dims, self.pixel_chunk,
            chunk_matrix, chunk_dims, 
            average::average,
        );

        let update_buffer = rbg_image_to_u32(&downsampled);
        
        match self.window.update_with_buffer(
            update_buffer.as_slice(), 
            self.window_dims.width as usize, self.window_dims.height as usize,
        ) {
            Ok(u) => u,
            Err(e) => {
                eprintln!("Update oopsie: {e}");
                return true;
            } 
        };

        true
    }

    fn should_close(&mut self) -> bool {
        if !self.window.is_open() || self.window.is_key_down(Key::Escape) {
            return true;
        }

        false
    }

    fn update_win_and_pix_size(&mut self) {
        let (win_width, win_height) = self.window.get_size();
        self.window_dims = Rect {
            width: win_width as u32, 
            height: win_height as u32,
        };

        if self.window.is_key_pressed(Key::Down, KeyRepeat::No) && self.pixel_chunk.height > 1 {
            self.pixel_chunk.height = self.pixel_chunk.height / 2;
        } else if self.window.is_key_pressed(Key::Up, KeyRepeat::No) && self.pixel_chunk.height < self.window_dims.height / 2 {
            self.pixel_chunk.height = self.pixel_chunk.height * 2; 
        }

        if self.window.is_key_pressed(Key::Right, KeyRepeat::No) && self.pixel_chunk.width > 1 {
            self.pixel_chunk.width = self.pixel_chunk.width / 2;
        } else if self.window.is_key_pressed(Key::Left, KeyRepeat::No) && self.pixel_chunk.width < self.window_dims.width / 2 {
            self.pixel_chunk.width = self.pixel_chunk.width * 2; 
        }

        if self.window.is_key_pressed(Key::LeftBracket, KeyRepeat::No) && self.pixel_chunk.width > 1 && self.pixel_chunk.height > 1 {
            self.pixel_chunk.width = self.pixel_chunk.width / 2;
            self.pixel_chunk.height = self.pixel_chunk.height / 2;
        } else if self.window.is_key_pressed(Key::RightBracket, KeyRepeat::No) && self.pixel_chunk.width < self.window_dims.width / 2 && self.pixel_chunk.height < self.window_dims.height / 2 {
            self.pixel_chunk.width = self.pixel_chunk.width * 2; 
            self.pixel_chunk.height = self.pixel_chunk.height * 2; 
        }
    }
}