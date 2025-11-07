#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressureState {
    Healthy,
    Soft,
    Hard,
}

pub fn classify(avail_pct: f64, soft: u8, hard: u8) -> PressureState {
    if avail_pct < hard as f64 {
        PressureState::Hard
    } else if avail_pct < soft as f64 {
        PressureState::Soft
    } else {
        PressureState::Healthy
    }
}
