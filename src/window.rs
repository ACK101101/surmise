use crate::transform::{Rect, average, calc_source_chunk_dims, downsample, rbg_image_to_u32};
use anyhow::Result;
use image::RgbImage;
use minifb::*;
use std::fmt;

enum Mode {
    Average,
    Reveal,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mode::Average => write!(f, "Average"),
            Mode::Reveal => write!(f, "Reveal"),
        }
    }
}

impl Mode {
    // todo: this can't be right
    fn toggle(&mut self) {
        match self {
            Mode::Average => {
                let new_mode = Mode::Reveal;
                *self = new_mode;
            }
            Mode::Reveal => {
                let new_mode = Mode::Average;
                *self = new_mode;
            }
        }
    }
}

pub struct Win {
    window: Window,
    pixel_chunk: Rect,
    mode: Mode,
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
        log::debug!(
            "Pixel Chunk Dims: ({}, {})",
            pixel_chunk.width,
            pixel_chunk.height
        );

        let window = Window::new(
            "surmise",
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            WindowOptions {
                borderless: true,
                resize: true,
                transparency: true,
                ..WindowOptions::default()
            },
        )?;

        Ok(Win {
            window,
            pixel_chunk,
            mode: Mode::Average,
        })
    }

    pub fn step(&mut self, raw_buf: RgbImage) -> bool {
        if self.should_close() {
            return false;
        }

        self.update_pix_size_and_mode();

        let raw_dims: Rect = Rect {
            width: raw_buf.width(),
            height: raw_buf.height(),
        };
        log::debug!("Raw Image Dims: ({}, {})", raw_dims.width, raw_dims.height);

        let (chunk_matrix, chunk_dims) = match calc_source_chunk_dims(
            raw_dims,
            Rect::from(self.window.get_size()),
            self.pixel_chunk,
        ) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Chunking oopsie: {e}");
                return true;
            }
        };
        log::debug!("Chunk Dims: ({}, {})", chunk_dims.width, chunk_dims.height);
        log::debug!(
            "Chunk Matrix: ({}, {})",
            chunk_matrix.width,
            chunk_matrix.height
        );

        let downsampled = downsample(
            raw_buf,
            Rect::from(self.window.get_size()),
            self.pixel_chunk,
            chunk_matrix,
            chunk_dims,
            average::average,
        );

        let update_buffer = rbg_image_to_u32(&downsampled);

        let (w, h) = self.window.get_size();
        match self
            .window
            .update_with_buffer(update_buffer.as_slice(), w, h)
        {
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

    fn update_pix_size_and_mode(&mut self) {
        let (w, h) = self.window.get_size();
        let (window_width, window_height) = (w as u32, h as u32);

        // update pixel_chunk width
        if self.window.is_key_pressed(Key::Right, KeyRepeat::No) && self.pixel_chunk.width > 1 {
            self.pixel_chunk.width = self.pixel_chunk.width / 2;
        } else if self.window.is_key_pressed(Key::Left, KeyRepeat::No)
            && self.pixel_chunk.width < window_width / 2
        {
            self.pixel_chunk.width = self.pixel_chunk.width * 2;
        }

        // update pixel_chunk height
        if self.window.is_key_pressed(Key::Down, KeyRepeat::No) && self.pixel_chunk.height > 1 {
            self.pixel_chunk.height = self.pixel_chunk.height / 2;
        } else if self.window.is_key_pressed(Key::Up, KeyRepeat::No)
            && self.pixel_chunk.height < window_height / 2
        {
            self.pixel_chunk.height = self.pixel_chunk.height * 2;
        }

        // update pixel_chunk width and height together
        if self.window.is_key_pressed(Key::LeftBracket, KeyRepeat::No)
            && self.pixel_chunk.width > 1
            && self.pixel_chunk.height > 1
        {
            self.pixel_chunk.width = self.pixel_chunk.width / 2;
            self.pixel_chunk.height = self.pixel_chunk.height / 2;
        } else if self.window.is_key_pressed(Key::RightBracket, KeyRepeat::No)
            && self.pixel_chunk.width < window_width / 2
            && self.pixel_chunk.height < window_height / 2
        {
            self.pixel_chunk.width = self.pixel_chunk.width * 2;
            self.pixel_chunk.height = self.pixel_chunk.height * 2;
        }

        // switch mode
        if self.window.is_key_pressed(Key::Space, KeyRepeat::No) {
            self.mode.toggle();
            println!("Toggled {}!", self.mode)
        }
    }
}
