//! Transposon-Hive — self-rewriting executable code.
//!
//! Modeled on retroviral memory: when a drone solves a *novel* threat or task
//! the solution is compiled to a portable bytecode capability and burned into the
//! disk executables of its peers, giving the whole hive permanent immunity.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HiveError {
    #[error("capability already installed: {0}")]
    AlreadyInstalled(String),
}

/// A compiled capability: eBPF-like bytecode or machine-code blob.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capability {
    pub id: String,
    /// eBPF, wasm, or raw machine code, distinguished by this tag.
    pub codec: String,
    pub bytecode: Vec<u8>,
}

/// A drone's genome: installed capabilities that self-rewrite the binary.
#[derive(Debug, Default)]
pub struct TransposonHive {
    capabilities: Vec<Capability>,
}

impl TransposonHive {
    pub fn new() -> Self {
        Self::default()
    }
    /// Transposable element: insert a new capability into the genome.
    pub fn integrate(&mut self, cap: Capability) -> Result<(), HiveError> {
        if self.capabilities.iter().any(|c| c.id == cap.id) {
            return Err(HiveError::AlreadyInstalled(cap.id));
        }
        self.capabilities.push(cap);
        Ok(())
    }

    /// The full genome of installed capabilities.
    pub fn genome(&self) -> &[Capability] {
        &self.capabilities
    }

    /// Serialize the genome for writing into a peer's executable.
    pub fn serialize(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(&self.capabilities)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integrates_once() {
        let mut h = TransposonHive::new();
        h.integrate(Capability { id: "anti-threat-x".into(), codec: "ebpf".into(), bytecode: vec![1] }).unwrap();
        assert!(matches!(
            h.integrate(Capability { id: "anti-threat-x".into(), codec: "ebpf".into(), bytecode: vec![2] }),
            Err(HiveError::AlreadyInstalled(_))
        ));
    }
}
