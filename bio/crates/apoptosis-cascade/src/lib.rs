//! Apoptosis-Cascade — graceful self-destruction with zero-trace cleanup.
//!
//! When a node is compromised or irreparably damaged, a caspase-like cascade
//! fires: zeroize the memory buffer, drop all cryptographic keys, and notify
//! neighbours so scavenger nodes (macrophages) remove the remains.

use thiserror::Error;
use zeroize::Zeroize;

#[derive(Debug, Error)]
pub enum ApoptosisError {
    #[error("already effete (cascade fired)")]
    AlreadyEffete,
}

#[derive(Debug, Clone)]
pub struct KeyMaterial {
    pub name: String,
    pub secret: Vec<u8>,
}

impl Zeroize for KeyMaterial {
    fn zeroize(&mut self) {
        self.secret.zeroize();
    }
}

#[derive(Debug)]
pub enum ApoptosisState {
    Alive,
    /// Caspase cascade in progress.
    Executing,
    Effete,
}

#[derive(Debug)]
pub struct ApoptosisCascade {
    pub state: ApoptosisState,
    pub keys: Vec<KeyMaterial>,
    pub scrap_notice_sent: bool,
}

impl ApoptosisCascade {
    pub fn new(keys: Vec<KeyMaterial>) -> Self {
        Self {
            state: ApoptosisState::Alive,
            keys,
            scrap_notice_sent: false,
        }
    }

    /// Fire the cascade: zeroize every key and notify neighbours.
    pub fn fire(&mut self, notify: &mut Vec<String>) -> Result<usize, ApoptosisError> {
        if matches!(self.state, ApoptosisState::Effete) {
            return Err(ApoptosisError::AlreadyEffete);
        }
        self.state = ApoptosisState::Executing;
        for k in &mut self.keys {
            k.zeroize();
        }
        notify.push("scavenge-remains".to_string());
        self.scrap_notice_sent = true;
        self.state = ApoptosisState::Effete;
        Ok(self.keys.len())
    }

    /// Confirm no plaintext secret is recoverable.
    pub fn all_zeroized(&self) -> bool {
        self.keys.iter().all(|k| k.secret.iter().all(|b| *b == 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cascade_zeroizes() {
        let keys = vec![KeyMaterial { name: "signer".into(), secret: vec![1, 2, 3] }];
        let mut a = ApoptosisCascade::new(keys);
        let mut notices = Vec::new();
        assert_eq!(a.fire(&mut notices).unwrap(), 1);
        assert!(a.all_zeroized());
        assert!(notices.contains(&"scavenge-remains".to_string()));
        assert!(matches!(a.fire(&mut notices), Err(ApoptosisError::AlreadyEffete)));
    }
}
