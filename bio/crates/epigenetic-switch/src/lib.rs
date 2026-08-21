//! Epigenetic-Switch — runtime methylation profile that flips modes
//! without altering the binary or its self-hash.
//!
//! The code and its BLAKE3 self-hash stay byte-identical; only the runtime
//! profile (`eqm-methy`) changes, immediately re-expressing behavior:
//!
//! - `Stealth`: quiet, minimal logging, low power
//! - `Hyperdrive`: maximum throughput, full logging
//! - `Fortress`: strict validation, defensive checks everywhere

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Stealth,
    Hyperdrive,
    Fortress,
}

impl Mode {
    pub fn as_u8(self) -> u8 {
        match self {
            Mode::Stealth => 0,
            Mode::Hyperdrive => 1,
            Mode::Fortress => 2,
        }
    }

    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Mode::Stealth),
            1 => Some(Mode::Hyperdrive),
            2 => Some(Mode::Fortress),
            _ => None,
        }
    }
}

#[derive(Debug, Error)]
pub enum EpigeneticError {
    #[error("invalid mode: {0}")]
    InvalidMode(u8),
}

/// The `eqm-methy` profile: runtime methylation bits stored out-of-band from
/// the binary. Uses atomics so it can be flipped from any thread instantly.
#[derive(Debug)]
pub struct EpigeneticSwitch {
    // The self-hash is fixed at build time. It must NEVER change at runtime;
    // that is the invariant proving the binary is unmodified.
    self_hash: String,
    mode: AtomicU8,
    /// Switch flip counter; useful for auditing.
    flips: AtomicU64,
}

impl EpigeneticSwitch {
    /// Construct with the binary's own BLAKE3 self-hash.
    pub fn new(self_hash: impl Into<String>, initial: Mode) -> Self {
        Self {
            self_hash: self_hash.into(),
            mode: AtomicU8::new(initial.as_u8()),
            flips: AtomicU64::new(0),
        }
    }

    pub fn mode(&self) -> Mode {
        Mode::from_u8(self.mode.load(Ordering::Acquire)).unwrap_or(Mode::Stealth)
    }

    pub fn self_hash(&self) -> &str {
        &self.self_hash
    }

    pub fn flips(&self) -> u64 {
        self.flips.load(Ordering::Acquire)
    }

    /// Methylate: switch the profile instantly, without touching the binary.
    pub fn methylate(&self, mode: Mode) {
        self.mode.store(mode.as_u8(), Ordering::Release);
        self.flips.fetch_add(1, Ordering::AcqRel);
    }

    // Behavioral gates driven by the current methylation profile.

    /// Logging verbosity: Stealth logs nothing, Hyperdrive logs everything,
    /// Fortress logs only security events.
    pub fn should_log(&self, is_security_event: bool) -> bool {
        match self.mode() {
            Mode::Stealth => false,
            Mode::Hyperdrive => true,
            Mode::Fortress => is_security_event,
        }
    }

    /// Validation strictness for incoming packets.
    pub fn validation_level(&self) -> u8 {
        match self.mode() {
            Mode::Stealth => 1,
            Mode::Hyperdrive => 2,
            Mode::Fortress => 3,
        }
    }

    /// Power cap percentage (Stealth limits power).
    pub fn power_cap_pct(&self) -> u8 {
        match self.mode() {
            Mode::Stealth => 40,
            Mode::Hyperdrive => 100,
            Mode::Fortress => 80,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_never_changes() {
        let sw = EpigeneticSwitch::new("SELF", Mode::Stealth);
        let before = sw.self_hash().to_string();
        sw.methylate(Mode::Hyperdrive);
        sw.methylate(Mode::Fortress);
        sw.methylate(Mode::Stealth);
        assert_eq!(sw.self_hash(), before);
        assert_eq!(sw.flips(), 3);
    }

    #[test]
    fn mode_behavior() {
        let sw = EpigeneticSwitch::new("SELF", Mode::Stealth);
        assert!(!sw.should_log(false));
        sw.methylate(Mode::Hyperdrive);
        assert!(sw.should_log(false));
        sw.methylate(Mode::Fortress);
        assert!(!sw.should_log(false));
        assert!(sw.should_log(true));
    }
}
