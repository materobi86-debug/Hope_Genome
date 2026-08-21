//! Physarum-Path — brainless mesh route optimization.
//!
//! Each Node hosts a `Physarum` instance. Physarum maintains a map from
//! (target, next_hop) pairs to tube "conductance" values, updated by a simple
//! oscillation rule: decay every tube, reinforce the ones carrying traffic.
//! Over time this prunes slow/lossy paths and thickens the fastest — exactly
//! like slime mold thickens efficient tubes and closes inefficient ones.
//!
//! The reinforcement signal comes from the observed RTT and loss for each tube.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::time::Duration;

/// A network neighbour of this node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Neighbour {
    pub node_id: u32,
    pub udp_port: u16,
}

/// A single tube between this node and the next hop toward a target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tube {
    pub target: u32,
    pub via: Neighbour,
}

#[derive(Debug, Clone, Copy)]
pub struct TubeMetrics {
    pub rtt: Duration,
    pub loss_permille: u32,
}

/// Slime-mold style conductance table.
pub struct Physarum {
    /// Tube conductance; 0.0 means collapsed (unused), 1.0 means maximum flow.
    conductance: HashMap<Tube, f64>,
    decay: f64,
    /// The last observation for each tube, if any.
    observations: HashMap<Tube, TubeMetrics>,
    seq: u64,
}

impl Physarum {
    pub fn new(decay: f64) -> Self {
        Self {
            conductance: HashMap::new(),
            observations: HashMap::new(),
            decay,
            seq: 0,
        }
    }

    /// Add a tube to the network if it does not already exist; new tubes start
    /// with a conductance of 0.5 (balanced).
    pub fn add_tube(&mut self, tube: Tube) {
        self.conductance.entry(tube).or_insert(0.5);
    }

    /// Record an observation for a tube.
    pub fn observe(&mut self, tube: Tube, metrics: TubeMetrics) {
        self.observations.insert(tube, metrics);
    }

    /// One oscillator pass: decay all tubes, then reinforce the tubes carrying
    /// the fastest traffic for each target.
    pub fn step(&mut self) {
        let observations = self.observations.clone();
        for (_, g) in self.conductance.iter_mut() {
            *g *= 1.0 - self.decay;
            if *g < 0.05 {
                *g = 0.05;
            }
        }
        // Group tubes by target and find the fastest tube for each.
        let mut fastest: HashMap<u32, (Tube, f64)> = HashMap::new();
        for (tube, _) in self.conductance.iter() {
            if let Some(metrics) = observations.get(tube) {
                let score = slime_score(metrics.rtt, metrics.loss_permille);
                let entry = fastest.entry(tube.target).or_insert((*tube, score));
                if score > entry.1 {
                    *entry = (*tube, score);
                }
            }
        }
        // Thicken the fastest tube for each target.
        for (_, (tube, _)) in fastest {
            if let Some(g) = self.conductance.get_mut(&tube) {
                *g = (*g + 0.25).min(1.0);
            }
        }
        // Collapse tubes that stay below the threshold for too long.
        self.seq += 1;
        let collapsed: Vec<Tube> = self
            .conductance
            .iter()
            .filter(|(t, g)| **g <= 0.05 && t.via.node_id % 16 == 0)
            .map(|(t, _)| *t)
            .collect();
        for tube in &collapsed {
            self.observations.remove(tube);
            self.conductance.remove(tube);
        }
    }

    /// Choose the route with the highest conductance for `target`.
    pub fn best_next_hop(&self, target: u32) -> Option<Neighbour> {
        self.conductance
            .iter()
            .filter(|(t, _)| t.target == target)
            .max_by(|(_, g1), (_, g2)| g1.partial_cmp(g2).unwrap_or(Ordering::Equal))
            .map(|(t, _)| t.via)
    }

    /// Returns (hop, conductance) pairs sorted by conductance descending.
    pub fn tube_ranking(&self, target: u32) -> Vec<(Neighbour, f64)> {
        let mut tubes: Vec<_> = self
            .conductance
            .iter()
            .filter(|(t, _)| t.target == target)
            .map(|(t, g)| (t.via, *g))
            .collect();
        tubes.sort_by(|(_, g1), (_, g2)| g2.partial_cmp(g1).unwrap_or(Ordering::Equal));
        tubes
    }
}

/// Convert a tube observation into a "slime" score in (0, 1].
/// Lower latency and lower loss mean a higher score.
fn slime_score(rtt: Duration, loss_permille: u32) -> f64 {
    let rtt_ms = rtt.as_secs_f64() * 1000.0;
    // Favor low latency: weight 0.6. Favor low loss: weight 0.4.
    let latency = 1.0 / (1.0 + rtt_ms / 100.0);
    let reliability = 1.0 - (loss_permille as f64 / 1000.0 * 0.5);
    (0.6 * latency + 0.4 * reliability).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tube(target: u32, node_id: u32) -> Tube {
        Tube {
            target,
            via: Neighbour { node_id, udp_port: 8888 },
        }
    }

    #[test]
    fn fast_tube_wins_over_slow() {
        let mut p = Physarum::new(0.1);
        let fast = tube(10, 1);
        let slow = tube(10, 2);
        p.add_tube(fast);
        p.add_tube(slow);
        p.observe(fast, TubeMetrics { rtt: Duration::from_millis(5), loss_permille: 0 });
        p.observe(slow, TubeMetrics { rtt: Duration::from_millis(300), loss_permille: 400 });

        for _ in 0..10 {
            p.step();
        }

        assert_eq!(p.best_next_hop(10), Some(fast.via));
        let ranking = p.tube_ranking(10);
        assert_eq!(ranking[0].0, fast.via);
        assert!(ranking[0].1 > ranking[1].1);
    }

    #[test]
    fn lossy_path_is_suppressed() {
        let mut p = Physarum::new(0.2);
        let clean = tube(7, 1);
        let lossy = tube(7, 3);
        p.add_tube(clean);
        p.add_tube(lossy);
        p.observe(clean, TubeMetrics { rtt: Duration::from_millis(50), loss_permille: 0 });
        p.observe(lossy, TubeMetrics { rtt: Duration::from_millis(50), loss_permille: 900 });

        for _ in 0..8 {
            p.step();
        }

        assert_eq!(p.best_next_hop(7), Some(clean.via));
    }

    #[test]
    fn unknown_target_has_no_route() {
        let p = Physarum::new(0.1);
        assert_eq!(p.best_next_hop(99), None);
        assert!(p.tube_ranking(99).is_empty());
    }

    #[test]
    fn slime_score_orders_latency_and_loss() {
        let fast = slime_score(Duration::from_millis(1), 0);
        let slow = slime_score(Duration::from_millis(500), 0);
        let lossy = slime_score(Duration::from_millis(1), 1000);
        assert!(fast > slow);
        assert!(fast > lossy);
        assert!(fast <= 1.0);
    }
}
