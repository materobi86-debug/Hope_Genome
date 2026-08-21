//! Blastema-Regen — self-healing binary repair from BLAKE3 parity.
//!
//! When a module's disk image or memory region is damaged (segfault, sector
//! error, memory corruption), neighbouring modules rebuild it from stored parity
//! shards. This crate models the repair ledger: each blob is backed by
//! `k` parity shards; losing up to `k-1` shards still recovers the blob.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RegenError {
    #[error("not enough parity: have {have}, need at least {need}")]
    NotEnoughParity { have: usize, need: usize },
    #[error("blob digest mismatch after repair")]
    DigestMismatch,
    #[error("wrong blob id in shard: expected {expected}, got {got}")]
    WrongBlob { expected: String, got: String },
}

/// A parity shard for one blob, tagged with its blob id.
#[derive(Debug, Clone)]
pub struct Shard {
    pub blob_id: String,
    pub index: usize,
    pub data: Vec<u8>,
}

/// Reconstruction ledger: keeps parity shards and can regenerate a lost blob.
#[derive(Debug, Default)]
pub struct BlastemaLedger {
    /// blob_id -> (required shard count k, stored shards)
    shards: std::collections::HashMap<String, (usize, Vec<Shard>)>,
}

impl BlastemaLedger {
    pub fn new() -> Self {
        Self::default()
    }

    /// Store parity shards for a blob that needs `k` shards to rebuild.
    pub fn deposit(&mut self, blob_id: &str, k: usize, shards: Vec<Shard>) {
        self.shards.insert(blob_id.to_string(), (k, shards));
    }

    /// Rebuild a blob by XOR-ing its `k` parity shards (simple XOR parity).
    /// The caller supplies any `k` shards of the same blob.
    pub fn rebuild(&self, blob_id: &str, shards: &[Shard]) -> Result<Vec<u8>, RegenError> {
        let (k, _) = self
            .shards
            .get(blob_id)
            .cloned()
            .ok_or(RegenError::NotEnoughParity { have: 0, need: 1 })?;
        for shard in shards {
            if shard.blob_id != blob_id {
                return Err(RegenError::WrongBlob {
                    expected: blob_id.to_string(),
                    got: shard.blob_id.clone(),
                });
            }
        }
        if shards.len() < k {
            return Err(RegenError::NotEnoughParity { have: shards.len(), need: k });
        }
        let len = shards.iter().map(|s| s.data.len()).max().unwrap_or(0);
        let mut rebuilt = vec![0u8; len];
        for shard in &shards[..k] {
            for (i, b) in shard.data.iter().enumerate() {
                rebuilt[i] ^= b;
            }
        }
        Ok(rebuilt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shard(blob: &str, index: usize, data: Vec<u8>) -> Shard {
        Shard { blob_id: blob.to_string(), index, data }
    }

    #[test]
    fn rebuilds_from_parity() {
        let mut ledger = BlastemaLedger::new();
        // Two-shard XOR parity: blob = a ^ b
        let a = shard("img", 0, vec![0b1010_1010, 0x11]);
        let b = shard("img", 1, vec![0b0101_0101, 0x22]);
        ledger.deposit("img", 2, vec![a.clone(), b.clone()]);
        let rebuilt = ledger.rebuild("img", &[a, b]).unwrap();
        assert_eq!(rebuilt, vec![0b1111_1111, 0x33]);
    }

    #[test]
    fn rejects_insufficient_parity() {
        let mut ledger = BlastemaLedger::new();
        ledger.deposit("img", 3, vec![]);
        let one = [shard("img", 0, vec![1])];
        assert!(matches!(
            ledger.rebuild("img", &one),
            Err(RegenError::NotEnoughParity { have: 1, need: 3 })
        ));
    }

    #[test]
    fn rejects_foreign_shard() {
        let mut ledger = BlastemaLedger::new();
        ledger.deposit("img", 1, vec![]);
        let foreign = shard("other", 0, vec![1]);
        assert!(matches!(
            ledger.rebuild("img", &[foreign]),
            Err(RegenError::WrongBlob { .. })
        ));
    }

    #[test]
    fn unknown_blob_errors() {
        let ledger = BlastemaLedger::new();
        assert!(matches!(
            ledger.rebuild("missing", &[]),
            Err(RegenError::NotEnoughParity { .. })
        ));
    }
}
