//! Xenobot-Swarm — distributed garbage & log sweeper.
//!
//! Autonomous micro-binaries wander the filesystem and memory, collecting orphaned
//! temp files, log remnants and memory leaks — like xenobots motoring through a
//! body to clear debris, then fusing when disjoint sweeps meet.

use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SwarmError {
    #[error("path is not a file")]
    NotAFile,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Debris {
    TempFile(String),
    LogRemnant(String),
    MemoryLeak(u64),
}

#[derive(Debug, Default)]
pub struct XenobotSwarm {
    collected: HashSet<Debris>,
}

impl XenobotSwarm {
    pub fn new() -> Self {
        Self::default()
    }

    /// A single xenobot scoops one debris item.
    pub fn scoop(&mut self, d: Debris) {
        self.collected.insert(d);
    }

    /// Many xenobots sweep a plane of candidate debris at once.
    pub fn sweep<I: IntoIterator<Item = Debris>>(&mut self, plane: I) -> usize {
        let before = self.collected.len();
        for d in plane {
            self.collected.insert(d);
        }
        self.collected.len() - before
    }

    /// Number of distinct debris items cleared.
    pub fn cleared(&self) -> usize {
        self.collected.len()
    }

    /// Settled histogram of what remains.
    pub fn take_all(&mut self) -> Vec<Debris> {
        self.collected.drain().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sweep_dedupes() {
        let mut swarm = XenobotSwarm::new();
        let found = swarm.sweep([
            Debris::TempFile("x.tmp".into()),
            Debris::TempFile("x.tmp".into()),
            Debris::MemoryLeak(4),
        ]);
        assert_eq!(found, 2);
        assert_eq!(swarm.cleared(), 2);
    }
}
