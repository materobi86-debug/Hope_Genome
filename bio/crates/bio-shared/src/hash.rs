//! BLAKE3 integrity helpers shared by every bio module.

use blake3::Hasher;

/// Hex-encoded BLAKE3 digest of a byte slice.
pub fn blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// Streaming hasher for payloads that arrive in chunks (frames, streams).
pub type Digest = Hasher;

/// Constant-time-ish digest comparison. Returns true when both digests match.
pub fn verify_digest(expected: &str, actual: &str) -> bool {
    if expected.len() != actual.len() {
        return false;
    }
    expected
        .bytes()
        .zip(actual.bytes())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_digest() {
        let a = blake3_hex(b"tun-state");
        let b = blake3_hex(b"tun-state");
        assert_eq!(a, b);
        assert!(verify_digest(&a, &b));
        assert!(!verify_digest(&a, &blake3_hex(b"tampered")));
    }
}
