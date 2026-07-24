pub mod driver;
pub mod lens;

use anyhow::Result;

use driver::Driver;
use lens::Lens;

pub struct Window {
    driver: Driver,
    lens: Lens,
}

#[derive(Clone, Copy)]
pub enum InputOutcome {
    Continue,
    Shutter,
    Open,
}

impl Window {
    pub fn new(idx: usize) -> Result<Window> {
        let driver = Driver::new(idx)?;
        let lens = Lens::new();

        Ok(Window { driver, lens })
    }

    pub fn poll_input(&mut self) -> InputOutcome {
        let (win_size_snap, win_pos_snap) = self.driver.snap_snapshots();
        self.lens.set_snapshots(win_size_snap, win_pos_snap);
        let outcome = self.driver.apply_input(&mut self.lens);
        outcome
    }

    pub fn give_lens(&mut self) -> &mut Lens {
        &mut self.lens
    }

    pub fn flush(&mut self) -> Result<()> {
        let (win_size_snap, _) = self.lens.get_snapshots();
        self.driver.flush(self.lens.get_frame(), &win_size_snap)
    }
}
