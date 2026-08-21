//! Electrocyte-Burst — micro-task aggregation into a power spike.
//!
//! Hundreds of small edge devices submit one result each at a microsecond-
//! synchronized moment; the host sums them into a single huge burst of completed
//! work, like serially stacked electrocytes producing one 860 V discharge.

use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BurstError {
    #[error("organ not registered")]
    NotRegistered,
}

/// One accumulated result from an edge device (an electrocyte).
#[derive(Debug, Clone, PartialEq)]
pub struct Voltage {
    /// 0.15 V per cell, metaphorically one result unit.
    pub cell_id: u64,
    pub result: u64,
}

#[derive(Debug, Default)]
pub struct ElectrocyteBurst {
    cells: Vec<Voltage>,
    pub deadline: Option<Duration>,
}

impl ElectrocyteBurst {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn arm(&mut self, deadline: Duration) {
        self.deadline = Some(deadline);
    }

    /// An edge device contributes a partial result.
    pub fn contribute(&mut self, cell_id: u64, result: u64) {
        self.cells.push(Voltage { cell_id, result });
    }

    /// Fire the burst: aggregate all partials into one total.
    pub fn fire(&self) -> Result<u64, BurstError> {
        if self.cells.is_empty() {
            return Err(BurstError::NotRegistered);
        }
        Ok(self.cells.iter().map(|c| c.result).sum())
    }

    pub fn count(&self) -> usize {
        self.cells.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn burst_sums() {
        let mut b = ElectrocyteBurst::new();
        b.contribute(1, 10);
        b.contribute(2, 20);
        b.contribute(3, 30);
        assert_eq!(b.fire().unwrap(), 60);
    }
}
