//! Sealed frame: a payload hardened with a BLAKE3 digest.
//!
//! Wire layout (little-endian):
//!
//! ```text
//! +--------+--------+----------+---------+----------------+
//! | magic  | kind   | payload  | digest  | payload bytes  |
//! | 4 B    | 1 B    | len 4 B  | 32 B    | len B          |
//! +--------+--------+----------+---------+----------------+
//! ```
//!
//! The digest covers magic + kind + len + payload, so any tampering with the
//! header or the body invalidates the frame.

use crate::hash::verify_digest;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const FRAME_MAGIC: &[u8; 4] = b"BIOF";
pub const DIGEST_LEN: usize = 32;
pub const HEADER_LEN: usize = 4 + 1 + 4 + DIGEST_LEN;

#[derive(Debug, Error)]
pub enum FrameError {
    #[error("frame too short: {0} bytes")]
    TooShort(usize),
    #[error("bad magic: expected BIOF")]
    BadMagic,
    #[error("payload truncated: header says {expected}, got {actual}")]
    Truncated { expected: usize, actual: usize },
    #[error("digest mismatch: expected {expected}, got {actual}")]
    DigestMismatch { expected: String, actual: String },
    #[error("payload is not valid serialized content")]
    CorruptedFrame,
}

/// A payload sealed with a BLAKE3 integrity digest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SealedFrame {
    pub kind: u8,
    pub payload: Vec<u8>,
    pub digest: String,
}

impl SealedFrame {
    /// Seal a payload, computing its BLAKE3 digest.
    pub fn seal(kind: u8, payload: Vec<u8>) -> Self {
        let digest = digest_of(kind, &payload);
        Self {
            kind,
            payload,
            digest,
        }
    }

    /// Verify the digest and return the payload if intact.
    pub fn open(&self) -> Result<&[u8], FrameError> {
        let actual = digest_of(self.kind, &self.payload);
        if verify_digest(&self.digest, &actual) {
            Ok(&self.payload)
        } else {
            Err(FrameError::DigestMismatch {
                expected: self.digest.clone(),
                actual,
            })
        }
    }

    /// Serialize to wire bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(HEADER_LEN + self.payload.len());
        out.extend_from_slice(FRAME_MAGIC);
        out.push(self.kind);
        out.extend_from_slice(&(self.payload.len() as u32).to_le_bytes());
        out.extend_from_slice(&hex_to_bytes(&self.digest));
        out.extend_from_slice(&self.payload);
        out
    }

    /// Parse and verify wire bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, FrameError> {
        if bytes.len() < HEADER_LEN {
            return Err(FrameError::TooShort(bytes.len()));
        }
        if &bytes[0..4] != FRAME_MAGIC {
            return Err(FrameError::BadMagic);
        }
        let kind = bytes[4];
        let len = u32::from_le_bytes(bytes[5..9].try_into().unwrap()) as usize;
        let digest = bytes_to_hex(&bytes[9..9 + DIGEST_LEN]);
        let payload_start = HEADER_LEN;
        if bytes.len() < payload_start + len {
            return Err(FrameError::Truncated {
                expected: len,
                actual: bytes.len() - payload_start,
            });
        }
        let payload = bytes[payload_start..payload_start + len].to_vec();
        let frame = Self {
            kind,
            payload,
            digest,
        };
        frame.open()?;
        Ok(frame)
    }
}

fn digest_of(kind: u8, payload: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(FRAME_MAGIC);
    hasher.update(&[kind]);
    hasher.update(&(payload.len() as u32).to_le_bytes());
    hasher.update(payload);
    hasher.finalize().to_hex().to_string()
}

fn hex_to_bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect()
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let frame = SealedFrame::seal(7, b"anhydrobiosis".to_vec());
        let bytes = frame.to_bytes();
        let parsed = SealedFrame::from_bytes(&bytes).unwrap();
        assert_eq!(parsed, frame);
        assert_eq!(parsed.open().unwrap(), b"anhydrobiosis");
    }

    #[test]
    fn tamper_detection() {
        let frame = SealedFrame::seal(7, b"anhydrobiosis".to_vec());
        let mut bytes = frame.to_bytes();
        let last = bytes.len() - 1;
        bytes[last] ^= 0xFF;
        assert!(matches!(
            SealedFrame::from_bytes(&bytes),
            Err(FrameError::DigestMismatch { .. })
        ));
    }

    #[test]
    fn truncation_detection() {
        let frame = SealedFrame::seal(1, vec![0u8; 64]);
        let bytes = frame.to_bytes();
        assert!(matches!(
            SealedFrame::from_bytes(&bytes[..bytes.len() - 10]),
            Err(FrameError::Truncated { .. })
        ));
    }
}
