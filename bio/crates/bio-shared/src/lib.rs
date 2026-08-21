//! Shared primitives for the bio module family.
//!
//! Provides BLAKE3 integrity hashing, sealed frame encoding (used by
//! `tardigrade-tun` snapshots and wire messages), and the BioMessage v2
//! envelope shared by the network-facing modules (`physarum-path`,
//! `tcell-sentinel`, `symbiont-engine`).

pub mod frame;
pub mod hash;
pub mod message;

pub use frame::{FrameError, SealedFrame};
pub use hash::{blake3_hex, verify_digest};
pub use message::{BioMessage, MessageKind};
