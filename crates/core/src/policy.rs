use crate::psi::PSIMetrics;

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

/* Dual-threshold model: PSI OR meminfo (whichever triggers first).
 * Allows early intervention on memory pressure before available% drops. */
pub fn classify_with_psi(
    avail_pct: f64,
    soft_mem: u8,
    hard_mem: u8,
    psi_metrics: Option<&PSIMetrics>,
    psi_soft: f64,
    psi_hard: f64,
) -> PressureState {
    let mem_state = classify(avail_pct, soft_mem, hard_mem);
    
    if let Some(psi) = psi_metrics {
        let psi_avg10 = psi.some_avg10;
        
        if psi_avg10 >= psi_hard || mem_state == PressureState::Hard {
            return PressureState::Hard;
        }
        
        if psi_avg10 >= psi_soft || mem_state == PressureState::Soft {
            return PressureState::Soft;
        }
    }
    
    mem_state
}

