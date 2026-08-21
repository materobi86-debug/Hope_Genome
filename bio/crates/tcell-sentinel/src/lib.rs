//! T-Cell Sentinel — global immunity database of threat signatures.
//!
//! Records memory-leak fingerprints, unusual packet shapes, and network
//! anomalies as BLAKE3 threat signatures. The database is append-only and
//! queryable, acting as the "memory cells" of the immune system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SentinelError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThreatKind {
    MemoryLeak,
    PacketAnomaly,
    NetworkAnomaly,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThreatSignature {
    /// BLAKE3 digest of the observed anomaly (stack hash, packet hash, etc.).
    pub digest: String,
    pub kind: ThreatKind,
    pub first_seen_ms: u64,
    pub observed_count: u64,
    /// Metadata key/value pairs.
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ImmuneDatabase {
    /// digest -> signature
    signatures: HashMap<String, ThreatSignature>,
    /// Total observations across all signatures.
    pub total_events: u64,
}

impl ImmuneDatabase {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an observation; creates a new signature or increments an
    /// existing one. Returns the digest of the (canonical) signature.
    pub fn observe(
        &mut self,
        digest: &str,
        kind: ThreatKind,
        now_ms: u64,
        tags: HashMap<String, String>,
    ) -> String {
        self.total_events += 1;
        let sig = self
            .signatures
            .entry(digest.to_string())
            .or_insert_with(|| ThreatSignature {
                digest: digest.to_string(),
                kind,
                first_seen_ms: now_ms,
                observed_count: 0,
                tags: HashMap::new(),
            });
        sig.observed_count += 1;
        sig.tags.extend(tags);
        digest.to_string()
    }

    /// Look up a signature by its digest.
    pub fn get(&self, digest: &str) -> Option<&ThreatSignature> {
        self.signatures.get(digest)
    }

    /// Persist the whole database as JSON.
    pub fn save(&self, path: &std::path::Path) -> Result<(), SentinelError> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load the database from JSON.
    pub fn load(path: &std::path::Path) -> Result<Self, SentinelError> {
        let json = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&json)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn observe_creates_signature() {
        let mut db = ImmuneDatabase::new();
        let digest = db.observe(
            "abc",
            ThreatKind::MemoryLeak,
            1,
            HashMap::new(),
        );
        let sig = db.get(&digest).unwrap();
        assert_eq!(sig.observed_count, 1);
    }

    #[test]
    fn repeated_observation_increments() {
        let mut db = ImmuneDatabase::new();
        db.observe("abc", ThreatKind::PacketAnomaly, 1, HashMap::new());
        db.observe("abc", ThreatKind::PacketAnomaly, 2, HashMap::new());
        assert_eq!(db.get("abc").unwrap().observed_count, 2);
        assert_eq!(db.total_events, 2);
    }
}
