use crate::config::{
    DEFAULT_PIXEL_HEIGHT, DEFAULT_PIXEL_WIDTH, DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH,
    SMA_WINDOW_SIZE,
};
use crate::geometry::{Point, Rect};
use crate::transform::{
    calc_source_chunk_dims, downsample, lattice::PixelLattice, rbg_image_to_u32,
};

use anyhow::Result;
use image::RgbImage;
use minifb::*;
use std::fmt;

// Window Effect Modes
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

// Per Window Handling
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
    pub fn new(idx: usize) -> Result<Win> {
        let name = format!("surmise {idx}");
        log::debug!(
            "Window {} Dims: ({}, {})",
            name,
            DEFAULT_WINDOW_WIDTH,
            DEFAULT_WINDOW_HEIGHT
        );

        let pixel_chunk: Rect = Rect::new(DEFAULT_PIXEL_WIDTH as u32, DEFAULT_PIXEL_HEIGHT as u32);
        log::debug!("Pixel Chunk Dims: {pixel_chunk:?}");

        let window = Window::new(
            &name,
            DEFAULT_WINDOW_WIDTH,
            DEFAULT_WINDOW_HEIGHT,
            WindowOptions {
                resize: true,
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

    pub fn step(&mut self, raw_buf: &RgbImage) -> WinStepOutcome {
        let outcome = self.determine_win_outcome();
        if matches!(outcome, WinStepOutcome::Shutter) {
            return outcome;
        }

        let (win_w, win_h) = self.window.get_size();
        let curr_win_dims = Rect::new(win_w as u32, win_h as u32);
        let source_dims: Rect = Rect::new(raw_buf.width(), raw_buf.height());
        log::debug!("Raw Image Dims: {source_dims:?}");

        let updated_pixel_chunk = self.update_pixel_chunk(curr_win_dims);
        self.update_effect_mode(curr_win_dims, updated_pixel_chunk);

        let (pixel_chunk_matrix, source_chunk_matrix, origin) = match calc_source_chunk_dims(
            source_dims,
            curr_win_dims,
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

        log::debug!("Source Chunk Matrix: {source_chunk_matrix:?}");
        log::debug!("Pixel Chunk Matrix: {pixel_chunk_matrix:?}");

        let downsampled = downsample(
            raw_buf,
            origin,
            curr_win_dims,
            self.pixel_chunk,
            pixel_chunk_matrix,
            source_chunk_matrix,
            self.effect_mode,
            &mut self.memory,
        );

        let update_buffer = rbg_image_to_u32(&downsampled);

        match self
            .window
            .update_with_buffer(update_buffer.as_slice(), win_w, win_h)
        {
            Ok(u) => u,
            Err(e) => {
                eprintln!("Update oopsie: {e}");
            }
        };

        outcome
    }

    fn determine_win_outcome(&self) -> WinStepOutcome {
        // determine if want to shutter or open window
        if !self.window.is_open()
            || self.window.is_key_down(Key::Escape)
            || self.window.is_key_pressed(Key::Q, KeyRepeat::No)
        {
            return WinStepOutcome::Shutter;
        }

        if self.window.is_key_pressed(Key::N, KeyRepeat::No) {
            return WinStepOutcome::Open;
        }

        WinStepOutcome::Continue
    }

    fn update_pixel_chunk(&mut self, curr_win_dims: Rect) -> bool {
        let (curr_win_w, curr_win_h) = curr_win_dims.get_dims();
        let (curr_pix_w, curr_pix_h) = self.pixel_chunk.get_dims();
        let (new_pix_w, new_pix_h): (u32, u32);

        // calc new pixel width
        if curr_pix_w > 1
            && (self.window.is_key_pressed(Key::Right, KeyRepeat::No)
                || self.window.is_key_pressed(Key::LeftBracket, KeyRepeat::No))
        {
            new_pix_w = curr_pix_w / 2;
        } else if curr_pix_w < curr_win_w / 2
            && (self.window.is_key_pressed(Key::Left, KeyRepeat::No)
                || self.window.is_key_pressed(Key::RightBracket, KeyRepeat::No))
        {
            new_pix_w = curr_pix_w * 2;
        } else {
            new_pix_w = curr_pix_w;
        }

        // calc new pixel height
        if curr_pix_h > 1
            && (self.window.is_key_pressed(Key::Down, KeyRepeat::No)
                || self.window.is_key_pressed(Key::LeftBracket, KeyRepeat::No))
        {
            new_pix_h = curr_pix_h / 2;
        } else if curr_pix_h < curr_win_h / 2
            && (self.window.is_key_pressed(Key::Up, KeyRepeat::No)
                || self.window.is_key_pressed(Key::RightBracket, KeyRepeat::No))
        {
            new_pix_h = curr_pix_h * 2;
        } else {
            new_pix_h = curr_pix_h;
        }

        if new_pix_w == curr_pix_w && new_pix_h == curr_pix_h {
            return false;
        }

        self.pixel_chunk.resize(new_pix_w, new_pix_h);
        true
    }

    fn update_effect_mode(&mut self, curr_win_dims: Rect, updated_pixel_chunk: bool) {
        let (win_w, win_h) = curr_win_dims.get_dims();
        let (pix_w, pix_h) = self.pixel_chunk.get_dims();

        // switch mode
        let mut toggled = false;
        if self.window.is_key_pressed(Key::Space, KeyRepeat::No) {
            self.effect_mode.toggle();
            toggled = true;
            log::debug!("Toggled {}!", self.effect_mode);
        }

        if matches!(self.effect_mode, EffectMode::Sma) && (updated_pixel_chunk || toggled) {
            self.memory = PixelLattice::new(
                (win_w / pix_w) as usize,
                (win_h / pix_h) as usize,
                SMA_WINDOW_SIZE,
            );
        }
    }

    // TODO: not checking if window size gets changed! not resetting important things if so
}
