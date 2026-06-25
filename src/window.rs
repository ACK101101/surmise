use crate::config::{
    DEFAULT_PIXEL_HEIGHT, DEFAULT_PIXEL_WIDTH, DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH,
    SMA_WINDOW_SIZE,
};
use crate::geometry::{Point, Rect};
use crate::transform::{
    average, calc_source_chunk_dims, downsample, lattice::PixelLattice, rbg_image_to_u32, reflect_y,
};

use anyhow::Result;
use image::RgbImage;
use minifb::*;
use std::fmt;

#[derive(Copy, Clone)]
pub enum EffectMode {
    Default,
    Reveal,
    Sma,
}

impl fmt::Display for EffectMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EffectMode::Default => write!(f, "Average"),
            EffectMode::Reveal => write!(f, "Reveal"),
            EffectMode::Sma => write!(f, "SMA"),
        }
    }
}

impl EffectMode {
    fn toggle(&mut self) {
        *self = match self {
            EffectMode::Default => EffectMode::Reveal,
            EffectMode::Reveal => EffectMode::Sma,
            EffectMode::Sma => EffectMode::Default,
        };
    }
}

pub struct Win {
    window: Window,
    pixel_chunk: Rect,
    memory: PixelLattice,
    effect_mode: EffectMode,
}

#[derive(Clone, Copy)]
pub enum WinStepOutcome {
    Continue,
    Shutter,
    Open,
}

impl Win {
    pub fn new() -> Result<Win> {
        log::debug!(
            "Window Dims: ({}, {})",
            DEFAULT_WINDOW_WIDTH,
            DEFAULT_WINDOW_HEIGHT
        );

        let pixel_chunk: Rect = Rect::new(DEFAULT_PIXEL_WIDTH as u32, DEFAULT_PIXEL_HEIGHT as u32);
        log::debug!("Pixel Chunk Dims: {pixel_chunk:?}");

        let window = Window::new(
            "surmise",
            DEFAULT_WINDOW_WIDTH,
            DEFAULT_WINDOW_HEIGHT,
            WindowOptions {
                borderless: true,
                resize: true,
                transparency: true,
                ..WindowOptions::default()
            },
        )?;

        let pixel_matrix =
            PixelLattice::new(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT, SMA_WINDOW_SIZE);

        Ok(Win {
            window,
            pixel_chunk,
            memory: pixel_matrix,
            effect_mode: EffectMode::Default,
        })
    }

    pub fn step(&mut self, raw_buf: RgbImage) -> WinStepOutcome {
        let outcome = self.update_pix_size_and_mode();
        if matches!(outcome, WinStepOutcome::Shutter) {
            return outcome;
        }

        let raw_dims: Rect = Rect::new(raw_buf.width(), raw_buf.height());
        log::debug!("Raw Image Dims: {raw_dims:?}");

        let (pixel_chunk_matrix, source_chunk_dims, origin) = match calc_source_chunk_dims(
            raw_dims,
            Rect::from(self.window.get_size()), // note: I don't think this works well with Rectangle resizing... idk why
            Point::from(self.window.get_position()),
            self.pixel_chunk,
            self.effect_mode,
        ) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Chunking oopsie: {e}");
                return outcome;
            }
        };

        log::debug!("Source Chunk Dims: {source_chunk_dims:?}");
        log::debug!("Pixel Chunk Matrix: {pixel_chunk_matrix:?}");

        let flipped_buf = reflect_y(&raw_buf);

        let downsampled = downsample(
            flipped_buf,
            origin,
            Rect::from(self.window.get_size()),
            self.pixel_chunk,
            pixel_chunk_matrix,
            source_chunk_dims,
            average::average,
            self.effect_mode,
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
                return outcome;
            }
        };

        outcome
    }

    // TODO: god this is ugly
    fn update_pix_size_and_mode(&mut self) -> WinStepOutcome {
        let mut outcome = WinStepOutcome::Continue;
        let (w, h) = self.window.get_size();
        let (window_width, window_height) = (w as u32, h as u32);
        let (mut pixel_width, mut pixel_height) = self.pixel_chunk.get_dims();
        let mut dirty_pixel_chunk = false;

        // determine if want to shutter or open window
        if !self.window.is_open() || self.window.is_key_down(Key::Escape) {
            return WinStepOutcome::Shutter;
        } else if self.window.is_key_pressed(Key::Q, KeyRepeat::No) {
            outcome = WinStepOutcome::Shutter
        } else if self.window.is_key_pressed(Key::N, KeyRepeat::No) {
            outcome = WinStepOutcome::Open
        }

        // update local pixel_chunk width
        if self.window.is_key_pressed(Key::Right, KeyRepeat::No) && pixel_width > 1 {
            pixel_width /= 2;
            dirty_pixel_chunk = true;
        } else if self.window.is_key_pressed(Key::Left, KeyRepeat::No)
            && pixel_width < window_width / 2
        {
            pixel_width *= 2;
            dirty_pixel_chunk = true;
        }

        // update local pixel_chunk height
        if self.window.is_key_pressed(Key::Down, KeyRepeat::No) && pixel_height > 1 {
            pixel_height /= 2;
            dirty_pixel_chunk = true;
        } else if self.window.is_key_pressed(Key::Up, KeyRepeat::No)
            && pixel_height < window_height / 2
        {
            pixel_height *= 2;
            dirty_pixel_chunk = true;
        }

        // update local pixel_chunk width and height together
        if self.window.is_key_pressed(Key::LeftBracket, KeyRepeat::No)
            && pixel_width > 1
            && pixel_height > 1
        {
            pixel_width /= 2;
            pixel_height /= 2;
            dirty_pixel_chunk = true;
        } else if self.window.is_key_pressed(Key::RightBracket, KeyRepeat::No)
            && pixel_width < window_width / 2
            && pixel_height < window_height / 2
        {
            pixel_width *= 2;
            pixel_height *= 2;
            dirty_pixel_chunk = true;
        }

        // update pixel_chunk state
        self.pixel_chunk.resize(pixel_width, pixel_height);

        // switch mode
        if self.window.is_key_pressed(Key::Space, KeyRepeat::No) {
            self.effect_mode.toggle();
            log::debug!("Toggled {}!", self.effect_mode);
            if matches!(self.effect_mode, EffectMode::Sma) {
                dirty_pixel_chunk = true;
            }
        }

        // update memory
        if dirty_pixel_chunk && matches!(self.effect_mode, EffectMode::Sma) {
            let pixel_lattice = PixelLattice::new(
                w / pixel_width as usize,
                h / pixel_height as usize,
                SMA_WINDOW_SIZE,
            );
            self.memory = pixel_lattice;
        }

        outcome
    }
}
