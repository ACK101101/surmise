use crate::transform::{
    EMA_SMOOTHING, Point, Rect, average, calc_source_chunk_dims, downsample, rbg_image_to_u32,
    reflect_y,
};
use anyhow::Result;
use image::{Rgb, RgbImage};
use minifb::*;
use std::fmt;

#[derive(Copy, Clone)]
pub enum Mode {
    Default,
    Reveal,
    EMA,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mode::Default => write!(f, "Average"),
            Mode::Reveal => write!(f, "Reveal"),
            Mode::EMA => write!(f, "EMA"),
        }
    }
}

impl Mode {
    // todo: this can't be right
    fn toggle(&mut self) {
        match self {
            Mode::Default => {
                let new_mode = Mode::Reveal;
                *self = new_mode;
            }
            Mode::Reveal => {
                let new_mode = Mode::EMA;
                *self = new_mode;
            }
            Mode::EMA => {
                let new_mode = Mode::Default;
                *self = new_mode;
            }
        }
    }
}

pub struct Win {
    window: Window,
    pixel_chunk: Rect,
    memory: PixelMatrix,
    mode: Mode,
}

// TODO: methods, prob shouldn't be all pub
pub struct PixelMatrix {
    pub pixels: Vec<Rgb<u8>>,
    pub width: usize,
    pub height: usize,
    pub steps: usize,
}

impl Win {
    pub fn new() -> Result<Win> {
        const WINDOW_WIDTH: usize = 960;
        const WINDOW_HEIGHT: usize = 540;
        log::debug!("Window Dims: ({}, {})", WINDOW_WIDTH, WINDOW_HEIGHT);

        // TODO: pull to config later
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

        let pixel_matrix = PixelMatrix {
            pixels: Vec::new(),
            width: WINDOW_WIDTH / pixel_chunk.width as usize,
            height: WINDOW_HEIGHT / pixel_chunk.height as usize,
            steps: EMA_SMOOTHING, // TODO: init to 0?
        };

        Ok(Win {
            window,
            pixel_chunk,
            memory: pixel_matrix,
            mode: Mode::Default,
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

        let (pixel_chunk_matrix, source_chunk_dims, origin) = match calc_source_chunk_dims(
            raw_dims,
            Rect::from(self.window.get_size()), // note: I don't think this works well with Rectangle resizing... idk why
            Point::from(self.window.get_position()),
            self.pixel_chunk,
            self.mode,
        ) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Chunking oopsie: {e}");
                return true;
            }
        };

        log::debug!(
            "Source Chunk Dims: ({}, {})",
            source_chunk_dims.width,
            source_chunk_dims.height
        );
        log::debug!(
            "Pixel Chunk Matrix: ({}, {})",
            pixel_chunk_matrix.width,
            pixel_chunk_matrix.height
        );

        let flipped_buf = reflect_y(&raw_buf);

        let downsampled = downsample(
            flipped_buf,
            origin,
            Rect::from(self.window.get_size()),
            self.pixel_chunk,
            pixel_chunk_matrix,
            source_chunk_dims,
            average::average,
            self.mode,
            &mut self.memory,
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

    // TODO: can prob make more efficient, use event or smth?
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
            log::debug!("Toggled {}!", self.mode);
            match self.mode {
                Mode::EMA => {
                    // TODO: use constructor
                    // TODO: update width and height
                    let pixel_matrix = PixelMatrix {
                        pixels: Vec::new(),
                        width: w / self.pixel_chunk.width as usize,
                        height: h / self.pixel_chunk.height as usize,
                        steps: EMA_SMOOTHING,
                    };
                    self.memory = pixel_matrix;
                }
                _ => (),
            }
        }
    }
}
