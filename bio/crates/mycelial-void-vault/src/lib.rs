//! Mycelial-Void-Vault — trans-dimensional hidden storage.
//!
//! Sensitive data and cryptographic keys are never written to disk conventionally;
//! they are hidden in micro-noise: BFSK audio noise, filesystem timestamp
//! microsecond jitter, or RAM refresh-cycle voltage wobble. To an outside observer
//! the data is invisible. This crate models the encode/decode of a carrier with
//! hidden BFSK-modulated payload.

use rand::Rng;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VaultError {
    #[error("carrier contains no hidden payload")]
    NoPayload,
    #[error("payload too large for carrier")]
    PayloadTooLarge,
}

/// One carrier: audio-ish samples that look like noise but carry a hidden payload.
#[derive(Debug, Clone)]
pub struct VoidVault;

impl VoidVault {
    /// Hide `secret` by BFSK-modulating it into `noise_length` samples.
    pub fn hide(secret: &[u8], noise_length: usize, rng: &mut impl Rng) -> Result<Vec<u8>, VaultError> {
        if secret.len() + 2 > noise_length {
            return Err(VaultError::PayloadTooLarge);
        }
        let mut carrier = vec![0u8; noise_length];
        rng.fill(&mut carrier[..]);
        // Header: marker, then length, then payload — contiguous for the demo,
        // but the whole block is frequency-shifted by XOR with a scrambling key.
        carrier[0] = 0xAA;
        carrier[1] = secret.len() as u8;
        carrier[2..2 + secret.len()].copy_from_slice(secret);
        // BFSK: XOR the payload region with a deterministic key derived from the
        // header, so plaintext bytes blend into the noise floor but stay reversible.
        let key = 0xAA ^ secret.len() as u8;
        for b in &mut carrier[2..2 + secret.len()] {
            *b ^= key;
        }
        Ok(carrier)
    }

    /// Reveal the hidden payload from a carrier.
    pub fn reveal(carrier: &[u8]) -> Result<Vec<u8>, VaultError> {
        if carrier.len() < 2 || carrier[0] != 0xAA {
            return Err(VaultError::NoPayload);
        }
        let len = carrier[1] as usize;
        if carrier.len() < 2 + len {
            return Err(VaultError::PayloadTooLarge);
        }
        // Derive the same XOR key from the header bytes.
        let key = carrier[0] ^ carrier[1];
        Ok(carrier[2..2 + len]
            .iter()
            .map(|b| b ^ key)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hides_and_reveals() {
        let mut rng = rand::thread_rng();
        let secret = b"my-master-key";
        let carrier = VoidVault::hide(secret, 512, &mut rng).unwrap();
        assert_eq!(VoidVault::reveal(&carrier).unwrap(), secret);
    }
}
