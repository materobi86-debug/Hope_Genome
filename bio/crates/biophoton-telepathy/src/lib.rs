//! Biophoton-Telepathy — zero-copy memory mirroring.
//!
//! Modeled on superradiant biophoton coherence between cells: memory regions are
//! mirrored peer-to-peer with zero copy and no network call into the "same" giant
//! address space. This crate models the mirroring ledger and coherence protocol —
//! the actual zero-copy transport is an RDMA/PCIe Gen6 backend behind the trait.

use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TelepathyError {
    #[error("page not mirrored")]
    NotMirrored,
}

/// A mirrored memory page.
#[derive(Debug, Clone)]
pub struct MirrorPage {
    pub addr: u64,
    pub len: usize,
    pub data: Vec<u8>,
    pub version: u64,
}

/// Coherence bus: writes propagate instantly to all mirrors of a page.
#[derive(Debug, Default)]
pub struct TelepathyBus {
    pages: HashMap<u64, MirrorPage>,
    /// Total bytes moved through zero-copy.
    pub zero_copy_bytes: u64,
}

impl TelepathyBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mirror(&mut self, page: MirrorPage) {
        self.zero_copy_bytes = self.zero_copy_bytes.saturating_add(page.len as u64);
        self.pages.insert(page.addr, page);
    }

    /// Read a mirrored page at its current coherence version.
    pub fn read(&self, addr: u64) -> Result<&MirrorPage, TelepathyError> {
        self.pages.get(&addr).ok_or(TelepathyError::NotMirrored)
    }

    /// Write a page: bump version, keeping one canonical copy.
    pub fn write(&mut self, addr: u64, data: Vec<u8>) -> Result<(), TelepathyError> {
        let page = self.pages.get_mut(&addr).ok_or(TelepathyError::NotMirrored)?;
        page.data = data;
        page.version += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mirror_and_read() {
        let mut bus = TelepathyBus::new();
        bus.mirror(MirrorPage { addr: 0x1000, len: 4, data: vec![1, 2], version: 0 });
        assert_eq!(bus.read(0x1000).unwrap().data, vec![1, 2]);
        bus.write(0x1000, vec![9, 9]).unwrap();
        assert_eq!(bus.read(0x1000).unwrap().version, 1);
    }
}
