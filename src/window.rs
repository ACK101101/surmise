use crate::config::*;
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

// Window State Handling
pub struct WinState {
    frame: Vec<u32>,
    scratch: Vec<u32>,
    win_size_snap: Rect,
    win_pos_snap: Point,
    pixel_chunk: Rect,
    memory: PixelLattice,
    effect_mode: EffectMode,
}

impl WinState {
    pub fn new(mode: EffectMode) -> WinState {
        WinState {
            frame: vec![0u32; DEFAULT_CAMERA_WIDTH * DEFAULT_CAMERA_HEIGHT],
            scratch: vec![0u32; DEFAULT_CAMERA_WIDTH * DEFAULT_CAMERA_HEIGHT],
            win_size_snap: Rect::new(DEFAULT_WINDOW_WIDTH as u32, DEFAULT_WINDOW_HEIGHT as u32),
            win_pos_snap: Point { x: 0, y: 0 },
            pixel_chunk: Rect::new(DEFAULT_PIXEL_WIDTH as u32, DEFAULT_PIXEL_HEIGHT as u32),
            memory: PixelLattice::new(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT, SMA_WINDOW_SIZE),
            effect_mode: mode,
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

        let downsampled = downsample(
            raw_buf,
            origin,
            self.win_size_snap,
            self.pixel_chunk,
            pixel_chunk_matrix,
            source_chunk_matrix,
            self.effect_mode,
            &mut self.memory,
        );

        rbg_image_to_u32(&downsampled, &mut self.scratch);
        std::mem::swap(&mut self.frame, &mut self.scratch);

        Ok(())
    }
}

#[derive(Clone, Copy)]
pub enum WinStepOutcome {
    Continue,
    Shutter,
    Open,
}

pub fn new_win(idx: usize) -> Result<Window> {
    let name = format!("surmise {idx}");
    log::debug!(
        "Window {} Dims: ({}, {})",
        name,
        DEFAULT_WINDOW_WIDTH,
        DEFAULT_WINDOW_HEIGHT
    );

    let mut window = Window::new(
        &name,
        DEFAULT_WINDOW_WIDTH,
        DEFAULT_WINDOW_HEIGHT,
        WindowOptions {
            resize: true,
            topmost: true,
            ..WindowOptions::default()
        },
    )?;

    window.set_target_fps(DEFAULT_WINDOW_FRAME_RATE); // I think without throttling, we are more likely to flicker due to minifb UAF issues

    Ok(window)
}

pub fn determine_win_outcome(win: &Window) -> WinStepOutcome {
    // determine if want to shutter or open window
    if !win.is_open() || win.is_key_down(Key::Escape) || win.is_key_pressed(Key::Q, KeyRepeat::No) {
        return WinStepOutcome::Shutter;
    }

    if win.is_key_pressed(Key::N, KeyRepeat::No) {
        return WinStepOutcome::Open;
    }

    WinStepOutcome::Continue
}

pub fn update_win_size_pos_snapshot(win: &Window, win_state: &mut WinState) {
    let (win_w, win_h) = win.get_size();
    win_state.win_size_snap = Rect::new(win_w as u32, win_h as u32);

    let (win_x, win_y) = win.get_position();
    win_state.win_pos_snap = Point {
        x: win_x as i32,
        y: win_y as i32,
    };
}

pub fn update_pixel_chunk(win: &Window, win_state: &mut WinState) -> bool {
    let (curr_win_w, curr_win_h) = win_state.win_size_snap.get_dims();
    let (curr_pix_w, curr_pix_h) = win_state.pixel_chunk.get_dims();
    let (new_pix_w, new_pix_h): (u32, u32);

    // calc new pixel width
    if curr_pix_w > 1
        && (win.is_key_pressed(Key::Right, KeyRepeat::No)
            || win.is_key_pressed(Key::LeftBracket, KeyRepeat::No))
    {
        new_pix_w = curr_pix_w / 2;
    } else if curr_pix_w < curr_win_w / 2
        && (win.is_key_pressed(Key::Left, KeyRepeat::No)
            || win.is_key_pressed(Key::RightBracket, KeyRepeat::No))
    {
        new_pix_w = curr_pix_w * 2;
    } else {
        new_pix_w = curr_pix_w;
    }

    // calc new pixel height
    if curr_pix_h > 1
        && (win.is_key_pressed(Key::Down, KeyRepeat::No)
            || win.is_key_pressed(Key::LeftBracket, KeyRepeat::No))
    {
        new_pix_h = curr_pix_h / 2;
    } else if curr_pix_h < curr_win_h / 2
        && (win.is_key_pressed(Key::Up, KeyRepeat::No)
            || win.is_key_pressed(Key::RightBracket, KeyRepeat::No))
    {
        new_pix_h = curr_pix_h * 2;
    } else {
        new_pix_h = curr_pix_h;
    }

    if new_pix_w == curr_pix_w && new_pix_h == curr_pix_h {
        return false;
    }

    win_state.pixel_chunk.resize(new_pix_w, new_pix_h);
    true
}

pub fn update_effect_mode(win: &Window, win_state: &mut WinState, updated_pixel_chunk: bool) {
    let (win_w, win_h) = win_state.win_size_snap.get_dims();
    let (pix_w, pix_h) = win_state.pixel_chunk.get_dims();

    // switch mode
    let mut toggled = false;
    if win.is_key_pressed(Key::Space, KeyRepeat::No) {
        win_state.effect_mode.toggle();
        toggled = true;
        log::debug!("Toggled {}!", win_state.effect_mode);
    }

    if matches!(win_state.effect_mode, EffectMode::Sma) && (updated_pixel_chunk || toggled) {
        win_state.memory = PixelLattice::new(
            (win_w / pix_w) as usize,
            (win_h / pix_h) as usize,
            SMA_WINDOW_SIZE,
        );
    }
}

pub fn flush(win: &mut Window, win_state: &WinState) -> Result<()> {
    win.update_with_buffer(
        &win_state.frame,
        win_state.win_size_snap.get_width() as usize,
        win_state.win_size_snap.get_height() as usize,
    )?;
    Ok(())
}

// TODO: not checking if window size gets changed! not resetting important things if so
