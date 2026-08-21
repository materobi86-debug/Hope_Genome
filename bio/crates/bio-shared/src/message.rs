//! BioMessage v2 envelope shared by the network-facing bio modules.
//!
//! Authenticated message with replay protection: every message carries a
//! sender id, a monotonically increasing sequence number and a nonce; the
//! BLAKE3 digest covers all of it, so replayed or forged frames are rejected.

use crate::frame::{FrameError, SealedFrame};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const MESSAGE_KIND: u8 = 0xB2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageKind {
    Heartbeat,
    Join,
    Task,
    Result,
    Probe,
    Threat,
}

#[derive(Debug, Error)]
pub enum MessageError {
    #[error(transparent)]
    Frame(#[from] FrameError),
    #[error("message decode failed: {0}")]
    Decode(String),
    #[error("replayed message: seq {seq} <= last seen {last}")]
    Replay { seq: u64, last: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BioMessage {
    pub sender: String,
    pub seq: u64,
    pub nonce: u64,
    pub kind: MessageKind,
    pub body: Vec<u8>,
}

impl BioMessage {
    pub fn new(sender: impl Into<String>, seq: u64, nonce: u64, kind: MessageKind, body: Vec<u8>) -> Self {
        Self {
            sender: sender.into(),
            seq,
            nonce,
            kind,
            body,
        }
    }

    /// Encode into a sealed wire frame.
    pub fn to_frame(&self) -> SealedFrame {
        let payload = serde_json::to_vec(self).expect("BioMessage serializes");
        SealedFrame::seal(MESSAGE_KIND, payload)
    }

    /// Decode and verify a wire frame.
    pub fn from_frame(frame: &SealedFrame) -> Result<Self, MessageError> {
        let payload = frame.open()?;
        serde_json::from_slice(payload).map_err(|e| MessageError::Decode(e.to_string()))
    }
}

/// Per-sender replay window: remembers the highest sequence number seen.
#[derive(Debug, Default)]
pub struct ReplayGuard {
    last_seq: std::collections::HashMap<String, u64>,
}

impl ReplayGuard {
    /// Accept the message if its sequence is fresh for this sender.
    pub fn accept(&mut self, msg: &BioMessage) -> Result<(), MessageError> {
        let last = self.last_seq.entry(msg.sender.clone()).or_insert(0);
        if msg.seq <= *last {
            return Err(MessageError::Replay {
                seq: msg.seq,
                last: *last,
            });
        }
        *last = msg.seq;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_roundtrip() {
        let msg = BioMessage::new("node-a", 1, 42, MessageKind::Heartbeat, b"ping".to_vec());
        let frame = msg.to_frame();
        let parsed = BioMessage::from_frame(&frame).unwrap();
        assert_eq!(parsed, msg);
    }

    #[test]
    fn replay_rejected() {
        let mut guard = ReplayGuard::default();
        let m1 = BioMessage::new("node-a", 1, 1, MessageKind::Probe, vec![]);
        let m2 = BioMessage::new("node-a", 1, 2, MessageKind::Probe, vec![]);
        assert!(guard.accept(&m1).is_ok());
        assert!(matches!(
            guard.accept(&m2),
            Err(MessageError::Replay { .. })
        ));
    }
}
