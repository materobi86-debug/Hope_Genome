//! User-space vagus nerve fallback for non-Linux platforms (and as a degraded
//! mode on Linux when eBPF loading is not permitted).
//!
//! Reads coarse system vitals and converts them into nerve signals. On Linux
//! this uses /proc; elsewhere it reports synthetic "no data" signals so the
//! interface stays uniform.

use crate::probes::{NerveSignal, VagusNerve};
#[cfg(target_os = "linux")]
use crate::probes::SignalKind;

#[cfg(target_os = "linux")]
fn now_ns() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

pub struct UserSpaceVagus {
    pending: Vec<NerveSignal>,
}

impl Default for UserSpaceVagus {
    fn default() -> Self {
        Self::new()
    }
}

impl UserSpaceVagus {
    pub fn new() -> Self {
        Self { pending: Vec::new() }
    }

    /// Sample system vitals once and enqueue any signals above the noise floor.
    pub fn sample(&mut self) {
        #[cfg(target_os = "linux")]
        self.sample_linux();
        #[cfg(not(target_os = "linux"))]
        self.sample_generic();
    }

    #[cfg(target_os = "linux")]
    fn sample_linux(&mut self) {
        // Memory pressure from /proc/meminfo.
        if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
            let mut total = 0u64;
            let mut avail = 0u64;
            for line in meminfo.lines() {
                if let Some(v) = line.strip_prefix("MemTotal:") {
                    total = parse_kb(v);
                } else if let Some(v) = line.strip_prefix("MemAvailable:") {
                    avail = parse_kb(v);
                }
            }
            if total > 0 {
                let used_milli = ((total - avail) * 1000 / total) as u32;
                if used_milli > 700 {
                    self.pending.push(NerveSignal {
                        kind: SignalKind::MemoryPressure,
                        severity_milli: used_milli,
                        detail: format!("mem used {} milli", used_milli),
                        timestamp_ns: now_ns(),
                    });
                }
            }
        }
        // I/O congestion from /proc/pressure/io (PSI), if available.
        if let Ok(psi) = std::fs::read_to_string("/proc/pressure/io") {
            if let Some(some_line) = psi.lines().find(|l| l.starts_with("some")) {
                if let Some(avg10) = extract_avg10(some_line) {
                    let milli = (avg10 * 10.0) as u32;
                    if milli > 100 {
                        self.pending.push(NerveSignal {
                            kind: SignalKind::IoCongestion,
                            severity_milli: milli.min(1000),
                            detail: format!("psi io avg10 {}", avg10),
                            timestamp_ns: now_ns(),
                        });
                    }
                }
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn sample_generic(&mut self) {
        // No /proc on this platform: emit nothing; callers treat silence as healthy.
    }
}

#[cfg(target_os = "linux")]
fn parse_kb(s: &str) -> u64 {
    s.trim()
        .split_whitespace()
        .next()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0)
}

#[cfg(target_os = "linux")]
fn extract_avg10(line: &str) -> Option<f64> {
    // Format: some avg10=0.00 avg60=0.00 avg300=0.00 total=0
    for part in line.split_whitespace() {
        if let Some(v) = part.strip_prefix("avg10=") {
            return v.parse::<f64>().ok();
        }
    }
    None
}

impl VagusNerve for UserSpaceVagus {
    fn poll(&mut self) -> Vec<NerveSignal> {
        self.sample();
        std::mem::take(&mut self.pending)
    }

    fn backend(&self) -> &'static str {
        "userspace"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poll_returns_vec_and_reports_backend() {
        let mut nerve = UserSpaceVagus::new();
        let _signals = nerve.poll();
        assert_eq!(nerve.backend(), "userspace");
    }
}
