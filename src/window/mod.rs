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
pub enum WindowStepOutcome {
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
}
