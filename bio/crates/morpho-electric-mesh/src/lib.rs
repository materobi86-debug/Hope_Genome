//! Morpho-Electric-Mesh — zero-orchestrator self-structuring.
//!
//! Modeled on Michael Levin's bioelectric morphogenesis: no central brain tells
//! cells which organ to become — local voltage (resource) gradients decide. Here,
//! each node observes the shared voltage vector from its peers and differentiates
//! into an *organ* (heart = compute core, lung = I/O buffer, nerve = router)
//! purely from the gradient, with no omega-master.

use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MorphoError {
    #[error("no peer voltage readings")]
    NoPeers,
}

/// What role a node differentiates into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Organ {
    Heart,
    Lung,
    Nerve,
}

/// A voltage reading shared over internal UDP.
#[derive(Debug, Clone, Copy)]
pub struct VoltageVector {
    pub cpu_potential: f64,
    pub io_potential: f64,
    pub net_potential: f64,
}

impl Organ {
    pub fn as_str(&self) -> &'static str {
        match self {
            Organ::Heart => "heart",
            Organ::Lung => "lung",
            Organ::Nerve => "nerve",
        }
    }
}

/// Self-structuring mesh: each node votes for a role by gradient, no coordinator.
#[derive(Debug, Default)]
pub struct MorphoMesh {
    peers: HashMap<String, VoltageVector>,
}

impl MorphoMesh {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn report(&mut self, peer: &str, v: VoltageVector) {
        self.peers.insert(peer.to_string(), v);
    }

    /// Differentiate into whichever organ the local gradient favors.
    /// The node becomes the max-potential domain it currently leads in.
    pub fn differentiate(&self, local: VoltageVector) -> Result<Organ, MorphoError> {
        if self.peers.is_empty() {
            return Err(MorphoError::NoPeers);
        }
        let mut totals = local;
        for v in self.peers.values() {
            totals.cpu_potential += v.cpu_potential;
            totals.io_potential += v.io_potential;
            totals.net_potential += v.net_potential;
        }
        let mut choices = [
            (totals.cpu_potential, Organ::Heart),
            (totals.io_potential, Organ::Lung),
            (totals.net_potential, Organ::Nerve),
        ];
        choices.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        Ok(choices[0].1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differentiates_by_gradient() {
        let mut m = MorphoMesh::new();
        m.report("peer-a", VoltageVector { cpu_potential: 0.2, io_potential: 0.1, net_potential: 0.1 });
        let local = VoltageVector { cpu_potential: 0.8, io_potential: 0.1, net_potential: 0.1 };
        assert_eq!(m.differentiate(local).unwrap(), Organ::Heart);
    }
}
