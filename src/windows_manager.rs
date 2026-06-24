use crate::window::{Win, WinStepOutcome};

use image::RgbImage;

use anyhow::{Context, Result, bail};

pub struct WindowsManager {
    wins: Vec<Win>,
}

impl WindowsManager {
    pub fn new() -> Result<WindowsManager> {
        let win = Win::new().context("Win oopsie")?;
        Ok(WindowsManager { wins: vec![win] })
    }

    pub fn is_alive(&mut self) -> bool {
        !self.wins.is_empty()
    }

    pub fn step_wins(&mut self, next_frame_buf: RgbImage) {
        let mut to_shutter_idxs: Vec<usize> = Vec::new();
        let mut win_to_open = 0;

        for (idx, win) in self.wins.iter_mut().enumerate() {
            match win.step(next_frame_buf.clone()) {
                WinStepOutcome::Shutter => to_shutter_idxs.push(idx),
                WinStepOutcome::Open => win_to_open += 1,
                _ => {}
            }
        }

        for idx in to_shutter_idxs.into_iter().rev() {
            self.shutter(idx).expect("shutter borked, idx wrong?");
        }

        for _ in 0..win_to_open {
            self.open().expect("open borked");
        }
    }

    fn open(&mut self) -> Result<()> {
        let win = Win::new().context("Win oopsie")?;
        self.wins.push(win);

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
