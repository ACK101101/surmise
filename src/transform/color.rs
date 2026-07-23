use std::fmt;

// --- ColorMode -----------------------------------------------------------------------------------
#[derive(Copy, Clone)]
pub enum ColorMode {
    Default, // no color transform
    Red,
    Green,
    Blue,
    // TODO: add more granular control
    // TODO: things like auto-gen palattes based on base color?
}

impl ColorMode {
    pub fn toggle(&mut self) {
        *self = match self {
            ColorMode::Default => ColorMode::Red,
            ColorMode::Red => ColorMode::Green,
            ColorMode::Green => ColorMode::Blue,
            ColorMode::Blue => ColorMode::Default,
        };
    }
}

impl fmt::Display for ColorMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColorMode::Default => write!(f, "Default"),
            ColorMode::Red => write!(f, "Red"),
            ColorMode::Green => write!(f, "Green"),
            ColorMode::Blue => write!(f, "Blue"),
        }
    }
}
