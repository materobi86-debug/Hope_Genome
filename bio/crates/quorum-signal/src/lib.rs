//! Quorum-Signal — distributed consensus before acting.
//!
//! Inspired by bacterial quorum sensing: a node only flips a shared feature
//! (biofilm, virulence, bioluminescence) once enough peers have voted. Each
//! member maintains a local tally; proposals pass when they reach threshold.

use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum QuorumError {
    #[error("unknown proposal: {0}")]
    UnknownProposal(String),
    #[error("duplicate vote from {0}")]
    DuplicateVote(String),
}

/// A proposable feature key, e.g. "bioluminescence".
#[derive(Debug, Clone)]
pub struct Proposal {
    pub id: String,
    pub threshold: usize,
    pub yes: HashSet<String>,
    pub no: HashSet<String>,
}

#[derive(Debug, Default)]
pub struct QuorumTable {
    proposals: HashMap<String, Proposal>,
}

impl QuorumTable {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a proposal; requires `threshold` distinct yes-votes to activate.
    pub fn propose(&mut self, id: &str, threshold: usize) {
        self.proposals.insert(
            id.to_string(),
            Proposal {
                id: id.to_string(),
                threshold,
                yes: HashSet::new(),
                no: HashSet::new(),
            },
        );
    }

    /// A peer casts a vote. Returns true if the yes-vote reaches quorum.
    pub fn vote(&mut self, id: &str, peer: &str, approve: bool) -> Result<bool, QuorumError> {
        let p = self
            .proposals
            .get_mut(id)
            .ok_or_else(|| QuorumError::UnknownProposal(id.to_string()))?;
        // One vote per peer.
        if p.yes.contains(peer) || p.no.contains(peer) {
            return Err(QuorumError::DuplicateVote(peer.to_string()));
        }
        if approve {
            p.yes.insert(peer.to_string());
        } else {
            p.no.insert(peer.to_string());
        }
        Ok(p.yes.len() >= p.threshold)
    }

    pub fn tally(&self, id: &str) -> Option<(usize, usize)> {
        self.proposals.get(id).map(|p| (p.yes.len(), p.no.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quorum_reached() {
        let mut qt = QuorumTable::new();
        qt.propose("biofilm", 3);
        assert!(!qt.vote("biofilm", "n1", true).unwrap());
        assert!(!qt.vote("biofilm", "n2", true).unwrap());
        assert!(qt.vote("biofilm", "n3", true).unwrap());
    }

    #[test]
    fn duplicate_rejected() {
        let mut qt = QuorumTable::new();
        qt.propose("glow", 2);
        assert!(qt.vote("glow", "n1", true).is_ok());
        assert!(matches!(
            qt.vote("glow", "n1", true),
            Err(QuorumError::DuplicateVote(_))
        ));
    }
}
