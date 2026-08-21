//! The on-disk (and runtime) representation of frozen process state.

use bio_shared::SealedFrame;
use serde::{Deserialize, Serialize};

/// A thread stack chain point captured at freeze time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThreadRecord {
    pub id: u64,
    pub name: String,
    pub pc: u64,
    pub spilled_frames: u64,
}

/// Panoramic position of the memory watchdog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryWatchdog {
    pub resident_bytes: u64,
    pub peak_bytes: u64,
    pub oom_events: u32,
    /// Allocation pressure 0..=1000; crossing 900 marks imminent OOM.
    pub pressure_milli: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeState {
    pub generation: u64,
    pub threads: Vec<ThreadRecord>,
    pub memory: MemoryWatchdog,
    /// Ids of workflows that had not received an EOF/close at freeze time.
    pub open_udp_workflows: std::collections::HashSet<u64>,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            generation: 1,
            threads: Vec::new(),
            memory: MemoryWatchdog {
                resident_bytes: 0,
                peak_bytes: 0,
                oom_events: 0,
                pressure_milli: 0,
            },
            open_udp_workflows: std::collections::HashSet::new(),
        }
    }
}

/// Full captured state: runtime state + application watchpoints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Watchpoint {
    pub key: String,
    pub value: String,
}

/// The "tun" (cask) that holds the frozen state + watchpoints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TunFrame {
    pub state: RuntimeState,
    pub watchpoints: Vec<Watchpoint>,
}

impl TunFrame {
    pub fn freeze(&self) -> SealedFrame {
        let payload = serde_json::to_vec(self).expect("serializable");
        SealedFrame::seal(super::FRAME_KIND, payload)
    }

    pub fn thaw(frame: &SealedFrame) -> Result<Self, bio_shared::FrameError> {
        let payload = frame.open()?;
        serde_json::from_slice(payload)
            .map_err(|_| bio_shared::FrameError::CorruptedFrame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let mut s = RuntimeState::default();
        s.open_udp_workflows.insert(42);
        let frame = TunFrame {
            state: s,
            watchpoints: vec![Watchpoint {
                key: "pwr".to_string(),
                value: "0x13".to_string(),
            }],
        }
        .freeze();
        assert!(TunFrame::thaw(&frame).is_ok());
    }
}
