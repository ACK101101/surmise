use anyhow::{Context, Result, bail};
use image::RgbImage;
use minifb::*;
use rayon::prelude::*;
use std::sync::Arc;

use crate::config::{DEFAULT_CAMERA_HEIGHT, DEFAULT_CAMERA_WIDTH};
use crate::frame_manager::FrameManager;
use crate::transform::effect::EffectMode;
use crate::win::*;

pub struct WindowsOrchestrator {
    frame: Arc<RgbImage>,
    frame_man: FrameManager,
    mfb_wins: Vec<Window>,
    win_states: Vec<WinState>,
    num_spawned: usize,
}

impl WindowsOrchestrator {
    pub fn new(frame_man: FrameManager) -> Result<WindowsOrchestrator> {
        let win = new_win(0).context("Win oopsie")?;
        let win_state = WinState::new(EffectMode::Default);

        let frame = Arc::new(RgbImage::new(DEFAULT_CAMERA_WIDTH, DEFAULT_CAMERA_HEIGHT));

        Ok(WindowsOrchestrator {
            frame,
            frame_man,
            mfb_wins: vec![win],
            win_states: vec![win_state],
            num_spawned: 1,
        })
    }

    pub fn is_alive(&self) -> bool {
        !self.mfb_wins.is_empty()
    }

    pub fn step_wins(&mut self) {
        let (to_shutter_idxs, win_to_open) = self.update_win_states();

        self.calculate_frames();

        self.flush_frames();

        for idx in to_shutter_idxs.into_iter().rev() {
            self.shutter(idx).expect("shutter borked, idx wrong?");
        }

        for _ in 0..win_to_open {
            self.open().expect("open borked");
        }
    }

    pub fn num_open(&self) -> usize {
        self.mfb_wins.len()
    }

    fn update_win_states(&mut self) -> (Vec<usize>, usize) {
        let mut to_shutter_idxs: Vec<usize> = Vec::new();
        let mut win_to_open = 0;

        for (idx, (win, win_state)) in self
            .mfb_wins
            .iter()
            .zip(self.win_states.iter_mut())
            .enumerate()
        {
            match determine_win_outcome(win) {
                WinStepOutcome::Shutter => {
                    to_shutter_idxs.push(idx);
                    continue;
                }
                WinStepOutcome::Open => win_to_open += 1,
                _ => {}
            };

            update_win_size_pos_snapshot(win, win_state);

            let updated_pixel_chunk = update_pixel_chunk(win, win_state);
            update_effect_mode(win, win_state, updated_pixel_chunk);
            update_transform_mode(win, win_state);
            update_color_mode(win, win_state);
        }

        (to_shutter_idxs, win_to_open)
    }

    fn calculate_frames(&mut self) {
        self.frame = self.frame_man.get_frame();

        self.win_states.par_iter_mut().for_each(|win_state| {
            if let Err(e) = win_state.calculate_and_save_frame(&self.frame) {
                eprintln!("Calc frame oopsie: {e}");
            }
        });
    }

    fn flush_frames(&mut self) {
        for (win, win_state) in self.mfb_wins.iter_mut().zip(self.win_states.iter()) {
            if let Err(e) = flush(win, win_state) {
                eprintln!("Update oopsie: {e}");
            }
        }
    }

    fn open(&mut self) -> Result<()> {
        let win = new_win(self.num_spawned).context("Win oopsie")?;
        let win_state = WinState::new(EffectMode::Default);
        self.mfb_wins.push(win);
        self.win_states.push(win_state);
        self.num_spawned += 1;

        Ok(())
    }

    fn shutter(&mut self, idx: usize) -> Result<()> {
        if idx >= self.mfb_wins.len() || idx >= self.win_states.len() {
            bail!("idx is out of bounds");
        }

        self.mfb_wins.remove(idx);
        self.win_states.remove(idx);
        Ok(())
    }
}
