//! Ultrasonic-Modem — air-gap BioMessage transfer over 18–22 kHz BFSK.
//!
//! Extends the acoustic cryo-modem concept into the ultrasonic band, inaudible
//! to humans. Two fully isolated (air-gapped) machines exchange encrypted
//! BioMessage packets through their speakers and microphones.
//!
//! Encoding: BFSK — bit 0 = 18 kHz tone, bit 1 = 22 kHz tone, each bit held
//! for `samples_per_bit` PCM samples at 48 kHz. Decoding uses the Goertzel
//! algorithm to detect which of the two tones dominates in each bit window.
//!
//! This crate is pure DSP (no audio-I/O dependency): it produces and consumes
//! `f32` PCM buffers; the host wires them to any audio backend.

use std::f32::consts::PI;
use thiserror::Error;

pub const SAMPLE_RATE: u32 = 48_000;
pub const FREQ_ZERO: f32 = 18_000.0;
pub const FREQ_ONE: f32 = 22_000.0;
/// Default bit duration: ~2 ms ≈ 96 samples at 48 kHz.
pub const DEFAULT_SAMPLES_PER_BIT: usize = 96;

#[derive(Debug, Error)]
pub enum ModemError {
    #[error("buffer too short for {expected} bits, got samples for {actual}")]
    BufferTooShort { expected: usize, actual: usize },
    #[error("empty payload")]
    EmptyPayload,
}

/// BFSK modulator/demodulator for the ultrasonic band.
#[derive(Debug, Clone)]
pub struct UltrasonicModem {
    pub samples_per_bit: usize,
}

impl Default for UltrasonicModem {
    fn default() -> Self {
        Self {
            samples_per_bit: DEFAULT_SAMPLES_PER_BIT,
        }
    }
}

impl UltrasonicModem {
    pub fn new(samples_per_bit: usize) -> Self {
        Self { samples_per_bit }
    }

    /// Modulate bytes into ultrasonic PCM (f32, mono, 48 kHz).
    pub fn modulate(&self, data: &[u8]) -> Result<Vec<f32>, ModemError> {
        if data.is_empty() {
            return Err(ModemError::EmptyPayload);
        }
        let mut pcm = Vec::with_capacity(data.len() * 8 * self.samples_per_bit);
        for byte in data {
            for bit in (0..8).rev() {
                let freq = if (byte >> bit) & 1 == 1 {
                    FREQ_ONE
                } else {
                    FREQ_ZERO
                };
                for n in 0..self.samples_per_bit {
                    let t = n as f32 / SAMPLE_RATE as f32;
                    pcm.push((2.0 * PI * freq * t).sin() * 0.8);
                }
            }
        }
        Ok(pcm)
    }

    /// Demodulate ultrasonic PCM back into bytes.
    pub fn demodulate(&self, pcm: &[f32], byte_count: usize) -> Result<Vec<u8>, ModemError> {
        let bits = byte_count * 8;
        let needed = bits * self.samples_per_bit;
        if pcm.len() < needed {
            return Err(ModemError::BufferTooShort {
                expected: bits,
                actual: pcm.len() / self.samples_per_bit,
            });
        }
        let mut out = Vec::with_capacity(byte_count);
        for byte_idx in 0..byte_count {
            let mut byte = 0u8;
            for bit in 0..8 {
                let bit_idx = byte_idx * 8 + bit;
                let start = bit_idx * self.samples_per_bit;
                let window = &pcm[start..start + self.samples_per_bit];
                let p0 = goertzel_power(window, FREQ_ZERO, SAMPLE_RATE);
                let p1 = goertzel_power(window, FREQ_ONE, SAMPLE_RATE);
                if p1 > p0 {
                    byte |= 1 << (7 - bit);
                }
            }
            out.push(byte);
        }
        Ok(out)
    }
}

/// Goertzel algorithm: power of `target_freq` present in `samples`.
pub fn goertzel_power(samples: &[f32], target_freq: f32, sample_rate: u32) -> f32 {
    let k = (samples.len() as f32 * target_freq / sample_rate as f32).round() as usize;
    let omega = 2.0 * PI * k as f32 / samples.len() as f32;
    let coeff = 2.0 * omega.cos();
    let (mut s1, mut s2) = (0.0f32, 0.0f32);
    for &x in samples {
        let s0 = x + coeff * s1 - s2;
        s2 = s1;
        s1 = s0;
    }
    s1 * s1 + s2 * s2 - coeff * s1 * s2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modulate_demodulate_roundtrip() {
        let modem = UltrasonicModem::default();
        let payload = b"air-gap-biomessage";
        let pcm = modem.modulate(payload).unwrap();
        // All energy must be above the audible band (>= 18 kHz by design).
        let decoded = modem.demodulate(&pcm, payload.len()).unwrap();
        assert_eq!(decoded, payload);
    }

    #[test]
    fn goertzel_detects_correct_tone() {
        let modem = UltrasonicModem::default();
        let pcm = modem.modulate(&[0b1000_0000]).unwrap();
        let first_bit = &pcm[..modem.samples_per_bit];
        let p_one = goertzel_power(first_bit, FREQ_ONE, SAMPLE_RATE);
        let p_zero = goertzel_power(first_bit, FREQ_ZERO, SAMPLE_RATE);
        assert!(p_one > p_zero * 10.0);
    }

    #[test]
    fn empty_payload_rejected() {
        let modem = UltrasonicModem::default();
        assert!(matches!(
            modem.modulate(&[]),
            Err(ModemError::EmptyPayload)
        ));
    }

    #[test]
    fn short_buffer_rejected() {
        let modem = UltrasonicModem::default();
        assert!(matches!(
            modem.demodulate(&[0.0; 10], 4),
            Err(ModemError::BufferTooShort { .. })
        ));
    }
}
