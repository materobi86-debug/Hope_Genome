//! Ratatui-Organism — living terminal ecosystem visualizer.
//!
//! A real-time TUI dashboard for the whole bio ecosystem:
//! - live bioelectric voltage + status of all 21+ modules,
//! - an ASCII graph of mycorrhizal links and drone symbionts,
//! - a visible apoptosis → blastema regeneration cycle.
//!
//! The state model ([`OrganismState`]) is pure and testable; the binary
//! (`organism-tui`) drives it with a tick loop and renders via ratatui.

pub mod view;

pub use view::{draw, ModuleStatus, ModuleView, OrganismState};
