use anyhow::{Context, Result, bail};

mod window;
use window::*;

mod camera;
use camera::*;

mod config;

mod transform;

fn main() {
    env_logger::init();

    let mut camera = match Cam::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Cam oopsie: {e}");
            return;
        }
    };

    let mut windows = match Windows::new() {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Windows oopsie: {e}");
            return;
        }
    };

    while !windows.wins.is_empty() {
        let new_camera_buffer = match camera.next() {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Camera oopsie: {e}");
                return;
            }
        };

        let mut to_shutter_idxs: Vec<usize> = Vec::new();
        for (idx, win) in windows.wins.iter_mut().enumerate() {
            let ok = win.step(new_camera_buffer.clone());
            if !ok {
                to_shutter_idxs.push(idx);
            }
        }

        for idx in to_shutter_idxs.into_iter().rev() {
            windows.shutter(idx).expect("shutter borked, idx wrong?");
        }
    }
}

struct Windows {
    wins: Vec<Win>,
}

impl Windows {
    fn new() -> Result<Windows> {
        let win = Win::new().context("Win oopsie")?;
        Ok(Windows { wins: vec![win] })
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
