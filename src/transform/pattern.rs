use std::fmt;

// --- PatternMode -----------------------------------------------------------------------------------
#[derive(Copy, Clone)]
pub enum PatternMode {
    Default, // full color fill
    Dots,
    // TODO: single image throughout
    // TODO: multiple images to map to
}

impl PatternMode {
    pub fn toggle(&mut self) {
        *self = match self {
            PatternMode::Default => PatternMode::Dots,
            PatternMode::Dots => PatternMode::Default,
        };
    }
}

impl fmt::Display for PatternMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PatternMode::Default => write!(f, "Chunky"),
            PatternMode::Dots => write!(f, "Dots"),
        }
    }
}
