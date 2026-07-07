use crate::frame_manager::FrameManager;
use crate::window::{Win, WinStepOutcome};
use crate::geometry::Rect;

use anyhow::{Context, Result, bail};
use rayon::{prelude::*, range};

pub struct WindowsOrchestrator {
    frame_man: FrameManager,
    wins: Vec<Win>,
    num_spawned: usize,
}

impl WindowsOrchestrator {
    pub fn new(frame_man: FrameManager) -> Result<WindowsOrchestrator> {
        let win = Win::new(0).context("Win oopsie")?;
        Ok(WindowsOrchestrator {
            frame_man,
            wins: vec![win],
            num_spawned: 1,
        })
    }

    pub fn is_alive(&self) -> bool {
        !self.wins.is_empty()
    }

    pub fn step_wins(&mut self) {
        let (to_shutter_idxs, win_to_open) = self.update_win_props();

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
        self.wins.len()
    }

    fn update_win_props(&mut self) -> (Vec<usize>, usize) {
        let mut to_shutter_idxs: Vec<usize> = Vec::new();
        let mut win_to_open = 0;

        for (idx, win) in self.wins.iter_mut().enumerate() {
            match win.determine_win_outcome() {
                WinStepOutcome::Shutter => {
                    to_shutter_idxs.push(idx);
                    continue;
                }
                WinStepOutcome::Open => win_to_open += 1,
                _ => {}
            };

            win.update_win_size_pos_snapshot();

            let updated_pixel_chunk = win.update_pixel_chunk();
            win.update_effect_mode(updated_pixel_chunk);
        }

        (to_shutter_idxs, win_to_open)
    }

    fn calculate_frames(&self) {
        let frame = self.frame_man.get_frame();

        for win in self.wins.iter() {
            if let Err(e) = win.win_state.calculate_and_save_frame(&frame) {
                eprintln!("Calc frame oopsie: {e}");
            };
        }
    }

    fn flush_frames(&mut self) {
        for win in self.wins.iter_mut() {
            if let Err(e) = win.window.update_with_buffer(
                &win.win_state.frame,
                win.win_state.win_size_snap.get_width() as usize,
                win.win_state.win_size_snap.get_height() as usize,
            ) {
                eprintln!("Update oopsie: {e}");
            }
        }
    }

    fn open(&mut self) -> Result<()> {
        let win = Win::new(self.num_spawned).context("Win oopsie")?;
        self.wins.push(win);
        self.num_spawned += 1;

        Ok(())
    }

    fn shutter(&mut self, idx: usize) -> Result<()> {
        if idx >= self.wins.len() {
            bail!("idx is out of bounds");
        }

        self.wins.remove(idx);
        Ok(())
    }
}
