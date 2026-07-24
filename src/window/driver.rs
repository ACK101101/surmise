use anyhow::Result;
use minifb::{Key, KeyRepeat, Window as MinifbWindow, WindowOptions};

use crate::config::{DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH};
use crate::geometry::{Point, Rect};
use crate::window::{InputOutcome, lens::Lens};

// --- Driver of Minifb Window ---------------------------------------------------------------------
pub struct Driver {
    minifb_win: MinifbWindow,
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

        Ok(Driver { minifb_win })
    }

    pub fn snap_snapshots(&mut self) -> (Rect, Point) {
        let (win_w, win_h) = self.minifb_win.get_size();
        let win_size_snap = Rect::new(win_w as u32, win_h as u32);

        let (win_x, win_y) = self.minifb_win.get_position();
        let win_pos_snap = Point {
            x: win_x as i32,
            y: win_y as i32,
        };

        (win_size_snap, win_pos_snap)
    }

    pub fn apply_input(&mut self, lens: &mut Lens) -> InputOutcome {
        // store signal for caller if minifb window needs to be opened or shuttered
        let mut outcome = InputOutcome::Continue;

        // route user input to relevant state changes
        for key in self.minifb_win.get_keys_pressed(KeyRepeat::No) {
            match key {
                // handle outcome keys
                Key::Q => {
                    outcome = InputOutcome::Shutter;
                    break;
                }
                Key::N => {
                    outcome = InputOutcome::Open;
                    break;
                }

                // handle tile keys
                Key::Right => lens.halve_tile_width(),
                Key::Left => lens.double_tile_width(),
                Key::Down => lens.halve_tile_height(),
                Key::Up => lens.double_tile_height(),
                Key::LeftBracket => lens.halve_tile(),
                Key::RightBracket => lens.double_tile(),

                // handle effect keys
                Key::Space => lens.toggle_effect_mode(),
                Key::C => lens.toggle_color_mode(),
                Key::Enter => lens.toggle_pattern_mode(),

                // fallthrough
                _ => (),
            }
        }

        outcome
    }

    pub fn flush(&mut self, frame: &[u32], win_size_snap: &Rect) -> Result<()> {
        self.minifb_win.update_with_buffer(
            frame,
            win_size_snap.get_width() as usize,
            win_size_snap.get_height() as usize,
        )?;
        Ok(())
    }
}
