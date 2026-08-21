//! Nerve signal types and the platform-agnostic vagus nerve interface.

use serde::{Deserialize, Serialize};

/// What the nervous system sensed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalKind {
    /// Systemic memory pressure (allocation stalls, reclaim activity).
    MemoryPressure,
    /// I/O congestion (block-layer queue depth, latency spikes).
    IoCongestion,
    /// Network anomaly (drops, retransmits, unusual traffic).
    NetworkAnomaly,
}

/// One nerve impulse from the kernel (or the user-space fallback).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NerveSignal {
    pub kind: SignalKind,
    /// Severity 0..=1000 (milli-units, like epigenetic pressure).
    pub severity_milli: u32,
    /// Human-readable detail (e.g. "reclaim 42ms", "retrans 3.2%").
    pub detail: String,
    /// Monotonic timestamp (ns since an arbitrary epoch).
    pub timestamp_ns: u64,
}

/// Platform-agnostic vagus nerve: any backend (eBPF or user-space) that can
/// produce nerve signals implements this.
pub trait VagusNerve {
    /// Drain currently pending signals.
    fn poll(&mut self) -> Vec<NerveSignal>;

    /// Backend name, for diagnostics ("ebpf" / "userspace").
    fn backend(&self) -> &'static str;
}

/// Threshold helper: is a signal critical enough to trigger a response?
pub fn is_critical(signal: &NerveSignal) -> bool {
    signal.severity_milli >= 800
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn critical_threshold() {
        let mild = NerveSignal {
            kind: SignalKind::MemoryPressure,
            severity_milli: 300,
            detail: "ok".into(),
            timestamp_ns: 1,
        };
        let severe = NerveSignal {
            severity_milli: 950,
            ..mild.clone()
        };
        assert!(!is_critical(&mild));
        assert!(is_critical(&severe));
    }
}
