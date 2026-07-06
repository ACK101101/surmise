use crate::camera::Cam;
use crate::window::{Win, WinStepOutcome};

use anyhow::{Context, Result, bail};

pub struct WindowsOrchestrator {
    cam: Cam,
    wins: Vec<Win>,
    num_spawned: usize,
}

impl WindowsOrchestrator {
    pub fn new(cam: Cam) -> Result<WindowsOrchestrator> {
        let win = Win::new(0).context("Win oopsie")?;
        Ok(WindowsOrchestrator {
            cam,
            wins: vec![win],
            num_spawned: 1,
        })
    }

    pub fn is_alive(&self) -> bool {
        !self.wins.is_empty()
    }

    pub fn step_wins(&mut self) {
        self.cam.load_next_frame().unwrap(); // TODO: temp, put in separate read thread

        let mut to_shutter_idxs: Vec<usize> = Vec::new();
        let mut win_to_open = 0;

        for (idx, win) in self.wins.iter_mut().enumerate() {
            match win.step(&self.cam.get_frame()) {
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

    pub fn num_open(&self) -> usize {
        self.wins.len()
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
