//! Magnetosome-Nav — global orientation along latency/bandwidth "field lines".
//!
//! Like magnetotactic bacteria aligning to Earth's magnetic field, nodes orient
//! to the topology's cheapest gradients (lowest latency, highest bandwidth). This
//! crate tracks per-destination field vectors and points to the best "north".

use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NavError {
    #[error("no field line known for {0}")]
    Unknown(String),
}

/// A field reading for one destination.
#[derive(Debug, Clone, Copy)]
pub struct FieldLine {
    pub latency_us: u64,
    pub bandwidth_kbps: u64,
    pub geo_spread: f64,
}

#[derive(Debug, Default)]
pub struct MagnetosomeNav {
    fields: HashMap<String, FieldLine>,
}

impl MagnetosomeNav {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, dest: &str, line: FieldLine) {
        self.fields.insert(dest.to_string(), line);
    }

    /// Orient to the destination with the best gradient (lowest cost).
    pub fn orient(&self, dest: &str) -> Result<FieldLine, NavError> {
        self.fields
            .get(dest)
            .copied()
            .ok_or_else(|| NavError::Unknown(dest.to_string()))
    }

    /// Pick the destination along the easiest gradient.
    pub fn strongest_pole(&self) -> Option<(&str, FieldLine)> {
        self.fields
            .iter()
            .min_by_key(|(_, f)| cost(**f))
            .map(|(k, f)| (k.as_str(), *f))
    }
}

fn cost(f: FieldLine) -> u64 {
    f.latency_us + (1_000_000 / f.bandwidth_kbps.max(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orients_to_cheapest() {
        let mut nav = MagnetosomeNav::new();
        nav.update("eu", FieldLine { latency_us: 10, bandwidth_kbps: 1000, geo_spread: 0.5 });
        nav.update("us", FieldLine { latency_us: 90, bandwidth_kbps: 1000, geo_spread: 0.5 });
        assert_eq!(nav.strongest_pole().unwrap().0, "eu");
    }
}
