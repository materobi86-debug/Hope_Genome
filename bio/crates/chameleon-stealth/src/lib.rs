//! Chameleon-Stealth — traffic signature camouflage.
//!
//! DPI evasion: BioMessage packets continuously morph their structure, ports and
//! timing to blend in with background noise — the UDP payload is wrapped as a
//! benign DNS-looking/HTTPS-ish probe, and the source port is drawn from a
//! rotating natural-looking pool.

use rand::Rng;

/// Camouflage disguises: what a packet pretends to be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Disguise {
    Dns,
    Ntp,
    Https,
}

impl Disguise {
    /// Rotate among the three disguises.
    pub fn rotates(&self, rng: &mut impl Rng) -> Self {
        let next_idx = rng.gen_range(0..3u8);
        match next_idx {
            0 => Disguise::Dns,
            1 => Disguise::Ntp,
            _ => Disguise::Https,
        }
    }
}

/// A chameleon that produces monotonically shifting phantoms.
pub struct Chameleon {
    port_pool: Vec<u16>,
    pub mode: Disguise,
}

impl Chameleon {
    const POOLS: [(Disguise, &[u16]); 3] = [
        (Disguise::Dns, &[53, 123, 5353]),
        (Disguise::Ntp, &[123, 53, 443]),
        (Disguise::Https, &[443, 53, 123]),
    ];

    pub fn new(mode: Disguise) -> Self {
        let pool = Self::POOLS
            .iter()
            .find(|(m, _)| *m == mode)
            .map(|(_, p)| p.to_vec())
            .unwrap();
        Self { port_pool: pool, mode }
    }

    /// Change the disguise to match background noise now.
    pub fn camou(&mut self, next: Disguise) {
        self.mode = next;
        self.port_pool = Self::POOLS
            .iter()
            .find(|(m, _)| *m == next)
            .map(|(_, p)| p.to_vec())
            .unwrap();
    }

    /// Pick a naturally-looking source port for the current disguise.
    pub fn src_port(&self, rng: &mut impl Rng) -> u16 {
        let idx = rng.gen_range(0..self.port_pool.len());
        // Jitter around the base port.
        let base = self.port_pool[idx];
        base + rng.gen_range(0..10)
    }

    /// Wrap a raw BioMessage payload to look like a `Disguise` frame.
    pub fn disguise_payload(&self, raw: &[u8], rng: &mut impl Rng) -> Vec<u8> {
        let header: &[u8] = match self.mode {
            Disguise::Dns => b"\x12\x34\x01\x00\x00\x01\x00\x00\x00\x00\x00\x00",
            Disguise::Ntp => b"\x1b\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00",
            Disguise::Https => b"\x16\x03\x03",
        };
        let mut out = Vec::new();
        out.extend_from_slice(header);
        let pad: usize = rng.gen_range(4..16);
        out.resize(out.len() + pad, 0);
        out.extend_from_slice(raw);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disguise_contains_payload() {
        let c = Chameleon::new(Disguise::Dns);
        let w = c.disguise_payload(b"bio", &mut rand::thread_rng());
        assert!(w.windows(3).any(|w| w == b"bio"));
    }
}
