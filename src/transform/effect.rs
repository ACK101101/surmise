use std::fmt;

// --- Window EffectMode ---------------------------------------------------------------------------
#[derive(Copy, Clone)]
pub enum EffectMode {
    Default, // average
    Reveal,
    Sma,
}

impl fmt::Display for EffectMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EffectMode::Default => write!(f, "Average"),
            EffectMode::Reveal => write!(f, "Reveal"),
            EffectMode::Sma => write!(f, "SMA"),
        }
    }
}

impl EffectMode {
    pub fn toggle(&mut self) {
        *self = match self {
            EffectMode::Default => EffectMode::Reveal,
            EffectMode::Reveal => EffectMode::Sma,
            EffectMode::Sma => EffectMode::Default,
        };
    }
}
