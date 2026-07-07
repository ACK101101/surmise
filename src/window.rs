use crate::config::{
    DEFAULT_CAMERA_HEIGHT, DEFAULT_CAMERA_WIDTH, DEFAULT_PIXEL_HEIGHT, DEFAULT_PIXEL_WIDTH,
    DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH, SMA_WINDOW_SIZE,
};
use crate::geometry::{Point, Rect};
use crate::transform::reflect_y;
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

// Window State Handling
pub struct WindowState {
    frame: Vec<u32>,
    win_size_snap: Rect,
    win_pos_snap: Point,
    pixel_chunk: Rect,
    memory: PixelLattice,
    effect_mode: EffectMode,
}

impl WindowState {
    pub fn new(
        win_size_snap: Rect,
        win_pos_snap: Point,
        pixel_chunk: Rect,
        memory: PixelLattice,
        effect_mode: EffectMode,
    ) -> WindowState {
        WindowState {
            frame: vec![0u32; DEFAULT_CAMERA_WIDTH * DEFAULT_CAMERA_HEIGHT],
            win_size_snap,
            win_pos_snap,
            pixel_chunk,
            memory,
            effect_mode,
        }
    }

    pub fn calculate_and_save_frame(&mut self, raw_buf: &RgbImage) -> Result<()> {
        let (pixel_chunk_matrix, source_chunk_matrix, origin) = match calc_source_chunk_dims(
            Rect::new(raw_buf.width(), raw_buf.height()),
            self.win_size_snap,
            self.win_pos_snap,
            self.pixel_chunk,
            self.effect_mode,
        ) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Chunking oopsie: {e}");
                return Err(e);
            }
        };

        log::debug!("Source Chunk Matrix: {source_chunk_matrix:?}");
        log::debug!("Pixel Chunk Matrix: {pixel_chunk_matrix:?}");

        let mut downsampled = downsample(
            raw_buf,
            origin,
            self.win_size_snap,
            self.pixel_chunk,
            pixel_chunk_matrix,
            source_chunk_matrix,
            self.effect_mode,
            &mut self.memory,
        );

        reflect_y(&mut downsampled);

        self.frame = rbg_image_to_u32(&downsampled);

        Ok(())
    }
}

// Per Window Handling
pub struct Win {
    window: Window,
    win_state: WindowState,
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
                topmost: true,
                ..WindowOptions::default()
            },
        )?;

        let win_state = WindowState::new(
            Rect::new(DEFAULT_WINDOW_WIDTH as u32, DEFAULT_WINDOW_HEIGHT as u32),
            Point { x: 0, y: 0 },
            pixel_chunk,
            PixelLattice::new(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT, SMA_WINDOW_SIZE),
            EffectMode::Default,
        );

        Ok(Win { window, win_state })
    }

    pub fn step(&mut self, raw_buf: &RgbImage) -> WinStepOutcome {
        let outcome = self.determine_win_outcome();
        if matches!(outcome, WinStepOutcome::Shutter) {
            return outcome;
        }

        self.update_win_size_pos_snapshot();

        let source_dims: Rect = Rect::new(raw_buf.width(), raw_buf.height());
        log::debug!("Raw Image Dims: {source_dims:?}");

        let updated_pixel_chunk = self.update_pixel_chunk();
        self.update_effect_mode(updated_pixel_chunk);

        if let Err(e) = self.win_state.calculate_and_save_frame(raw_buf) {
            eprintln!("Calc frame oopsie: {e}");
            return outcome;
        };

        if let Err(e) = self.window.update_with_buffer(
            &self.win_state.frame,
            self.win_state.win_size_snap.get_width() as usize,
            self.win_state.win_size_snap.get_height() as usize,
        ) {
            eprintln!("Update oopsie: {e}");
        }

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

    fn update_win_size_pos_snapshot(&mut self) {
        let (win_w, win_h) = self.window.get_size();
        self.win_state.win_size_snap = Rect::new(win_w as u32, win_h as u32);

        let (win_x, win_y) = self.window.get_position();
        self.win_state.win_pos_snap = Point {
            x: win_x as i32,
            y: win_y as i32,
        };
    }

    fn update_pixel_chunk(&mut self) -> bool {
        let (curr_win_w, curr_win_h) = self.win_state.win_size_snap.get_dims();
        let (curr_pix_w, curr_pix_h) = self.win_state.pixel_chunk.get_dims();
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

        self.win_state.pixel_chunk.resize(new_pix_w, new_pix_h);
        true
    }

    fn update_effect_mode(&mut self, updated_pixel_chunk: bool) {
        let (win_w, win_h) = self.win_state.win_size_snap.get_dims();
        let (pix_w, pix_h) = self.win_state.pixel_chunk.get_dims();

        // switch mode
        let mut toggled = false;
        if self.window.is_key_pressed(Key::Space, KeyRepeat::No) {
            self.win_state.effect_mode.toggle();
            toggled = true;
            log::debug!("Toggled {}!", self.win_state.effect_mode);
        }

        if matches!(self.win_state.effect_mode, EffectMode::Sma) && (updated_pixel_chunk || toggled)
        {
            self.win_state.memory = PixelLattice::new(
                (win_w / pix_w) as usize,
                (win_h / pix_h) as usize,
                SMA_WINDOW_SIZE,
            );
        }
    }

    // TODO: not checking if window size gets changed! not resetting important things if so
}
