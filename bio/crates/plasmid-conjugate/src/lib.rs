//! Plasmid-Conjugate — horizontal gene transfer between peer binaries.
//!
//! Two nodes exchange capabilities (signature keys, filter matrices, runtime
//! functions) directly over UDP/TCP without a central omega-master. Capabilities
//! are sealed with BLAKE3 so a forged plasmid is rejected.

use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlasmidError {
    #[error("capability already present: {0}")]
    AlreadyPresent(String),
    #[error("integrity failure: expected {expected}, got {actual}")]
    Integrity { expected: String, actual: String },
}

/// A transferable capability (e.g. a filter matrix or signing key).
///
/// `codec` declares what the payload carries: `"wasm"`, `"ebpf"`, `"key"`,
/// `"matrix"`, or `"raw"`. Sandboxed executors (e.g. `wasm-organoid`) only
/// accept plasmids whose codec they can run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plasmid {
    pub id: String,
    pub codec: String,
    pub payload: Vec<u8>,
}

impl Plasmid {
    pub fn seal(id: &str, payload: Vec<u8>) -> Self {
        Self {
            id: id.to_string(),
            codec: "raw".to_string(),
            payload,
        }
    }

    /// Seal a plasmid with an explicit codec.
    pub fn seal_typed(id: &str, codec: &str, payload: Vec<u8>) -> Self {
        Self {
            id: id.to_string(),
            codec: codec.to_string(),
            payload,
        }
    }

    /// Builder: set the codec on a sealed plasmid.
    pub fn with_codec(mut self, codec: &str) -> Self {
        self.codec = codec.to_string();
        self
    }

    pub fn digest(&self) -> String {
        let mut h = blake3::Hasher::new();
        h.update(self.id.as_bytes());
        h.update(self.codec.as_bytes());
        h.update(&self.payload);
        h.finalize().to_hex().to_string()
    }
}

/// A node's plasmid repository.
#[derive(Debug, Default)]
pub struct PlasmidStore {
    plasmids: HashMap<String, Plasmid>,
}

impl PlasmidStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Conjugation: acquire a plasmid if its digest is intact.
    pub fn conjugate(&mut self, p: &Plasmid, expected_digest: &str) -> Result<(), PlasmidError> {
        if p.digest() != expected_digest {
            return Err(PlasmidError::Integrity {
                expected: expected_digest.to_string(),
                actual: p.digest(),
            });
        }
        if self.plasmids.contains_key(&p.id) {
            return Err(PlasmidError::AlreadyPresent(p.id.clone()));
        }
        self.plasmids.insert(p.id.clone(), p.clone());
        Ok(())
    }

    pub fn has(&self, id: &str) -> bool {
        self.plasmids.contains_key(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acquire_verified_plasmid() {
        let p = Plasmid::seal("filter-matrix", vec![1, 2, 3]);
        let d = p.digest();
        let mut store = PlasmidStore::new();
        assert!(store.conjugate(&p, &d).is_ok());
        assert!(store.has("filter-matrix"));
    }

    #[test]
    fn reject_tampered() {
        let mut p = Plasmid::seal("key", vec![9, 9]);
        let good = p.digest();
        p.payload[0] ^= 0xFF;
        let mut store = PlasmidStore::new();
        assert!(matches!(
            store.conjugate(&p, &good),
            Err(PlasmidError::Integrity { .. })
        ));
    }
}
