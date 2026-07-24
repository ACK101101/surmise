use anyhow::Result;
use minifb::{Key, KeyRepeat, Window as MinifbWindow, WindowOptions};

use crate::config::{DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH};
use crate::geometry::{Point, Rect};
use crate::window::{WindowStepOutcome, lens::Lens};

// --- Driver of Minifb Window ---------------------------------------------------------------------
pub struct Driver {
    minifb_win: MinifbWindow,

    win_size_snap: Rect,
    win_pos_snap: Point,
}

impl Driver {
    pub fn new(idx: usize) -> Result<Driver> {
        let name = format!("surmise {idx}");
        log::debug!(
            "Window Driver {} Dims: ({}, {})",
            name,
            DEFAULT_WINDOW_WIDTH,
            DEFAULT_WINDOW_HEIGHT
        );

        let minifb_win = MinifbWindow::new(
            &name,
            DEFAULT_WINDOW_WIDTH as usize,
            DEFAULT_WINDOW_HEIGHT as usize,
            WindowOptions {
                resize: true,
                topmost: true,
                ..WindowOptions::default()
            },
        )?;

        Ok(Driver {
            minifb_win,

            win_size_snap: Rect::new(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT),
            win_pos_snap: Point { x: 0, y: 0 },
        })
    }

    pub fn update_snapshots(&mut self) {
        let (win_w, win_h) = self.minifb_win.get_size();
        self.win_size_snap = Rect::new(win_w as u32, win_h as u32);

        let (win_x, win_y) = self.minifb_win.get_position();
        self.win_pos_snap = Point {
            x: win_x as i32,
            y: win_y as i32,
        };
    }

    pub fn apply_input(&mut self, lens: &mut Lens) -> WindowStepOutcome {
        // store signal for caller if minifb window needs to be opened or shuttered
        let mut outcome = WindowStepOutcome::Continue;

        // route user input to relevant state changes
        for key in self.minifb_win.get_keys_pressed(KeyRepeat::No) {
            match key {
                // handle outcome keys
                Key::Q => {
                    outcome = WindowStepOutcome::Shutter;
                    break;
                }
                Key::N => {
                    outcome = WindowStepOutcome::Open;
                    break;
                }

                // handle tile keys
                Key::Right => lens.halve_tile_width(&self.win_size_snap),
                Key::Left => lens.double_tile_width(&self.win_size_snap),
                Key::Down => lens.halve_tile_height(&self.win_size_snap),
                Key::Up => lens.double_tile_height(&self.win_size_snap),
                Key::LeftBracket => lens.halve_tile(&self.win_size_snap),
                Key::RightBracket => lens.double_tile(&self.win_size_snap),

                // handle effect keys
                Key::Space => lens.toggle_effect_mode(&self.win_size_snap),
                Key::C => lens.toggle_color_mode(),
                Key::Enter => lens.toggle_pattern_mode(),

                // fallthrough
                _ => (),
            }
        }

        outcome
    }
    
    pub fn flush(&mut self, frame: &[u32]) -> Result<()> {
        self.minifb_win.update_with_buffer(
            frame,
            self.win_size_snap.get_width() as usize,
            self.win_size_snap.get_height() as usize,
        )?;
        Ok(())
    }
}
