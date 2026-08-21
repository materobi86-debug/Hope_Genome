//! Yamanaka-Deaging — zero-downtime state reversion.
//!
//! When memory fragmentation / leakage / config drift hits a critical threshold,
//! the runtime state is *rejuvenated in place* back to a pristine stem-cell
//! baseline — learned intelligence matrices are kept, but leak counters and
//! fragmentation metrics are zeroed in milliseconds, without restarting.

use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeagingError {
    #[error("not yet senescent: below threshold")]
    NotSenescent,
}

/// The "intelligence matrices" that survive deaging: learned weights.
#[derive(Debug, Clone, Default)]
pub struct LearnedState {
    pub matrices: HashMap<String, Vec<f64>>,
}

/// The "senescence" metrics that get zeroed during deaging.
#[derive(Debug, Clone, Copy, Default)]
pub struct Senescence {
    pub fragmentation: f64,
    pub leak_bytes: u64,
    pub drift: f64,
}

impl Senescence {
    pub fn critical(&self) -> bool {
        self.fragmentation > 0.7 || self.leak_bytes > 1_000_000 || self.drift > 0.5
    }
}

/// A yamanaka-deaged process state: learned + senescent halves.
#[derive(Debug, Clone, Default)]
pub struct CellularState {
    pub learned: LearnedState,
    pub senescence: Senescence,
}

impl CellularState {
    /// Rejuvenate: keep learned matrices, zero senescence, keep the process alive.
    pub fn deage(&mut self) -> Result<(), DeagingError> {
        if !self.senescence.critical() {
            return Err(DeagingError::NotSenescent);
        }
        self.senescence = Senescence {
            fragmentation: 0.0,
            leak_bytes: 0,
            drift: 0.0,
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_learned_but_resets_senescence() {
        let mut st = CellularState {
            learned: LearnedState {
                matrices: HashMap::from([("attn".to_string(), vec![0.5])]),
            },
            senescence: Senescence { fragmentation: 0.9, leak_bytes: 2_000_000, drift: 0.8 },
        };
        assert!(st.deage().is_ok());
        assert_eq!(st.learned.matrices.len(), 1);
        assert_eq!(st.senescence.fragmentation, 0.0);
    }

    #[test]
    fn refuses_when_healthy() {
        let mut st = CellularState::default();
        assert!(matches!(st.deage(), Err(DeagingError::NotSenescent)));
    }
}
