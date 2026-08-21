//! The Tun: watchdog-driven state freezer and revival path.

mod state;

pub use crate::state::{MemoryWatchdog, RuntimeState, ThreadRecord, TunFrame, Watchpoint};

use bio_shared::{blake3_hex, SealedFrame};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use thiserror::Error;

pub const FRAME_KIND: u8 = 0xAF; // ABA — active biological agent

#[derive(Debug, Error)]
pub enum ReviveError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("snapshot on disk is corrupt")]
    Corrupt,
}

#[derive(Debug, Clone)]
pub struct TunConfig {
    pub snapshot_dir: PathBuf,
    pub max_snapshots: usize,
}

impl Default for TunConfig {
    fn default() -> Self {
        Self {
            snapshot_dir: PathBuf::from("tun-snapshots"),
            max_snapshots: 3,
        }
    }
}

#[derive(Debug)]
pub struct Tun {
    config: TunConfig,
    state: Mutex<RuntimeState>,
    watchpoints: Mutex<std::vec::Vec<Watchpoint>>,
}

impl Tun {
    pub fn new(config: TunConfig) -> Self {
        std::fs::create_dir_all(&config.snapshot_dir).expect("snapshot dir");
        Self {
            config,
            state: Mutex::new(RuntimeState::default()),
            watchpoints: Mutex::new(Vec::new()),
        }
    }

    /// Register a thread within the logical snapshot ("thread stack").
    pub fn thread(&self, id: u64, name: impl Into<String>, pc: u64, spilled_frames: u64) {
        let mut state = self.state.lock().unwrap();
        state.threads.push(ThreadRecord {
            id,
            name: name.into(),
            pc,
            spilled_frames,
        });
    }

    /// Open a UDP workflow. The id survives until `close_udp_workflow`.
    pub fn open_udp_workflow(&self, id: u64) {
        self.state
            .lock()
            .unwrap()
            .open_udp_workflows
            .insert(id);
    }

    /// Close a UDP workflow, removing it from the "un-terminated" set.
    pub fn close_udp_workflow(&self, id: u64) {
        self.state.lock().unwrap().open_udp_workflows.remove(&id);
    }

    /// Set the current memory pressure (0..=1000).
    pub fn set_pressure(&self, milli: u32) {
        let mut state = self.state.lock().unwrap();
        state.memory.pressure_milli = milli;
    }

    /// Update resident memory counters and peak pressure.
    pub fn update_memory(&self, resident_bytes: u64) {
        let mut state = self.state.lock().unwrap();
        let mem = &mut state.memory;
        mem.resident_bytes = resident_bytes;
        mem.peak_bytes = mem.peak_bytes.max(resident_bytes);
        // Simple OOM proxy: a resident spike accompanied by high pressure.
        if mem.pressure_milli > 900
            && resident_bytes > mem.peak_bytes.saturating_sub(mem.peak_bytes / 10)
        {
            mem.oom_events += 1;
        }
    }

    /// Set/write a watchpoint value.
    pub fn watch(&self, key: impl Into<String>, value: impl Into<String>) {
        self.watchpoints.lock().unwrap().push(Watchpoint {
            key: key.into(),
            value: value.into(),
        });
    }

    /// Freeze the current state to disk, returning the sealed frame.
    pub fn freeze(&self) -> SealedFrame {
        let frame = {
            let state = self.state.lock().unwrap().clone();
            let watchpoints = self.watchpoints.lock().unwrap().clone();
            TunFrame { state, watchpoints }
                .freeze()
        };
        let path = self.current_path();
        std::fs::write(&path, frame.to_bytes()).expect("write snapshot");
        self.keep_archive(&frame);
        frame
    }

    /// Revive the latest snapshot from disk, if one exists.
    pub fn revive(&self, path: Option<&Path>) -> Result<Option<SealedFrame>, ReviveError> {
        let current = self.current_path();
        let path = path.unwrap_or(&current);
        if !path.exists() {
            return Ok(None);
        }
        let bytes = std::fs::read(path)?;
        let frame = SealedFrame::from_bytes(&bytes).map_err(|_| ReviveError::Corrupt)?;
        let tun = TunFrame::thaw(&frame).map_err(|_| ReviveError::Corrupt)?;
        *self.state.lock().unwrap() = tun.state;
        *self.watchpoints.lock().unwrap() = tun.watchpoints;
        Ok(Some(frame))
    }

    fn current_path(&self) -> PathBuf {
        self.config
            .snapshot_dir
            .join("tun-latest.tun")
    }

    fn keep_archive(&self, frame: &SealedFrame) {
        if self.config.max_snapshots <= 1 {
            return;
        }
        // Rotate archives: drop oldest.
        let dir = &self.config.snapshot_dir;
        let mut archives = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name = name.to_string_lossy().to_string();
                if name.starts_with("tun-archive-") {
                    archives.push(dir.join(name));
                }
            }
            archives.sort_by_key(|p| std::fs::metadata(p).and_then(|m| Ok(m.modified().ok())).ok());
            while archives.len() >= self.config.max_snapshots {
                if let Some(oldest) = archives.pop() {
                    let _ = std::fs::remove_file(oldest);
                }
            }
        }
        let digest = blake3_hex(frame.to_bytes().as_slice());
        let name = format!("tun-archive-{digest}");
        let _ = std::fs::copy(&self.current_path(), dir.join(name));
    }
}
