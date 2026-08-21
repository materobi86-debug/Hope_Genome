//! Mycorrhiza-Trade — Wood Wide Web resource redistribution.
//!
//! Nodes advertise CPU/RAM headroom; abundant nodes donate to starved edge
//! devices so the whole forest stays balanced. This crate models the ledger-side
//! capacity tracking and donation matching.

use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TradeError {
    #[error("donor {0} has nothing to give")]
    EmptyDonor(String),
    #[error("unknown node: {0}")]
    UnknownNode(String),
}

#[derive(Debug, Clone, Default)]
pub struct Capacity {
    /// Normalized 0..1 headroom (higher = richer).
    pub cpu_headroom: f64,
    pub ram_headroom: f64,
}

/// A mycorrhizal network forest.
#[derive(Debug, Default)]
pub struct Forest {
    nodes: HashMap<String, Capacity>,
}

impl Forest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, id: &str, cap: Capacity) {
        self.nodes.insert(id.to_string(), cap);
    }

    pub fn capacity(&self, id: &str) -> Option<&Capacity> {
        self.nodes.get(id)
    }

    /// Donate a fraction of headroom from a rich node to a poor node.
    /// Both converge toward the forest mean.
    pub fn donate(
        &mut self,
        donor: &str,
        recipient: &str,
        amount: f64,
    ) -> Result<(), TradeError> {
        let d = self
            .nodes
            .get(donor)
            .ok_or_else(|| TradeError::UnknownNode(donor.to_string()))?
            .clone();
        let r = self
            .nodes
            .get(recipient)
            .ok_or_else(|| TradeError::UnknownNode(recipient.to_string()))?
            .clone();
        if d.cpu_headroom + d.ram_headroom <= 0.0 {
            return Err(TradeError::EmptyDonor(donor.to_string()));
        }
        let a = amount.clamp(0.0, d.cpu_headroom.min(d.ram_headroom));
        let mut dn = d.clone();
        let mut rc = r.clone();
        dn.cpu_headroom -= a;
        dn.ram_headroom -= a;
        rc.cpu_headroom += a;
        rc.ram_headroom += a;
        self.nodes.insert(donor.to_string(), dn);
        self.nodes.insert(recipient.to_string(), rc);
        Ok(())
    }

    /// Mean headroom across the forest; measure of balance.
    pub fn balance(&self) -> (f64, f64) {
        let n = self.nodes.len().max(1) as f64;
        let (mut cpu, mut ram) = (0.0, 0.0);
        for c in self.nodes.values() {
            cpu += c.cpu_headroom;
            ram += c.ram_headroom;
        }
        (cpu / n, ram / n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn donation_balances() {
        let mut f = Forest::new();
        f.register("old-tree", Capacity { cpu_headroom: 0.9, ram_headroom: 0.8 });
        f.register("seedling", Capacity { cpu_headroom: 0.1, ram_headroom: 0.1 });
        f.donate("old-tree", "seedling", 0.3).unwrap();
        assert!(f.capacity("seedling").unwrap().cpu_headroom > 0.3);
    }
}
