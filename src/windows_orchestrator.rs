use anyhow::{Context, Result, bail};
use image::RgbImage;
use rayon::prelude::*;
use std::sync::Arc;

use crate::config::{DEFAULT_CAMERA_HEIGHT, DEFAULT_CAMERA_WIDTH};
use crate::frame_manager::FrameManager;
use crate::window::{Window, InputOutcome, lens::Lens};

pub struct WindowsOrchestrator {
    frame: Arc<RgbImage>,
    frame_man: FrameManager,
    windows: Vec<Window>,
    num_spawned: usize,
}

impl WindowsOrchestrator {
    pub fn new(frame_man: FrameManager) -> Result<WindowsOrchestrator> {
        let window = Window::new(0).context("Win oopsie")?;

        let frame = Arc::new(RgbImage::new(DEFAULT_CAMERA_WIDTH, DEFAULT_CAMERA_HEIGHT));

        Ok(WindowsOrchestrator {
            frame,
            frame_man,
            windows: vec![window],
            num_spawned: 1,
        })
    }

    pub fn is_alive(&self) -> bool {
        !self.windows.is_empty()
    }

    pub fn step_wins(&mut self) {
        let (to_shutter_idxs, win_to_open) = self.poll_inputs();

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
        self.windows.len()
    }

    fn poll_inputs(&mut self) -> (Vec<usize>, usize) {
        let mut to_shutter_idxs: Vec<usize> = Vec::new();
        let mut win_to_open = 0;

        for (idx, window) in self.windows.iter_mut().enumerate() {
            match window.poll_input() {
                InputOutcome::Shutter => {
                    to_shutter_idxs.push(idx);
                    continue;
                }
                InputOutcome::Open => win_to_open += 1,
                _ => {}
            };
        }

        (to_shutter_idxs, win_to_open)
    }

    fn calculate_frames(&mut self) {
        self.frame = self.frame_man.get_frame();
        let mut lenses: Vec<&mut Lens> = self.windows.iter_mut().map(|w| w.give_lens()).collect();

        lenses.par_iter_mut().for_each(|l| {
            if let Err(e) = l.calculate_and_save_frame(&self.frame) {
                eprintln!("Calc frame oopsie: {e}");
            }
        });
    }

    fn flush_frames(&mut self, ) {
        for window in self.windows.iter_mut() {
            if let Err(e) = window.flush() {
                eprintln!("Update oopsie: {e}");
            }
        }
    }

    fn open(&mut self) -> Result<()> {
        let window = Window::new(self.num_spawned).context("Win oopsie")?;
        self.windows.push(window);
        self.num_spawned += 1;
        Ok(())
    }

    fn shutter(&mut self, idx: usize) -> Result<()> {
        if idx >= self.windows.len() {
            bail!("idx is out of bounds");
        }

        self.windows.remove(idx);
        Ok(())
    }
}
