//! Vagus-eBPF — kernel-level nervous system for the bio ecosystem.
//!
//! Instead of user-space polling, eBPF probes installed in the Linux kernel
//! (via `aya`) report systemic memory pressure, I/O congestion and network
//! anomalies with near-zero overhead and kernel-grade reaction time.
//!
//! Platform strategy:
//! - **Linux**: `EbpfVagus` loads tracepoint/kprobe programs (source in
//!   `ebpf/vagus.bpf.c`, built with `aya-ebpf` / `cargo xtask`).
//! - **Other platforms** (Windows, macOS): `UserSpaceVagus` provides the same
//!   [`VagusNerve`] interface from `/proc`-style or OS APIs, so the rest of
//!   the ecosystem is platform-agnostic.

pub mod probes;

pub use probes::{NerveSignal, SignalKind, VagusNerve};

#[cfg(target_os = "linux")]
pub mod ebpf_loader;

#[cfg(target_os = "linux")]
pub use ebpf_loader::EbpfVagus;

pub mod userspace;
pub use userspace::UserSpaceVagus;
