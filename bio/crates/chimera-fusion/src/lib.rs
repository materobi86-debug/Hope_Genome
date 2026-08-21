//! Chimera-Fusion — endosymbiotic super-intelligence.
//!
//! The Rust core absorbs Python (PyO3), C/C++, Wasm and eBPF modules at runtime,
//! fusing their strongest traits into a single integrated super-process without
//! memory boundaries. This crate models the fusion registry: a typed host that
//! marshals calls into co-resident runtimes through one unified dispatch surface.

use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FusionError {
    #[error("runtime {0} not absorbed")]
    NotAbsorbed(String),
}

/// Foreign runtimes the Rust core can absorb.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeKind {
    Python,
    Cpp,
    Wasm,
    Ebpf,
}

impl RuntimeKind {
    pub fn name(&self) -> &'static str {
        match self {
            RuntimeKind::Python => "python",
            RuntimeKind::Cpp => "cpp",
            RuntimeKind::Wasm => "wasm",
            RuntimeKind::Ebpf => "ebpf",
        }
    }
}

/// A fusible module handle. Real backends are PyO3/cxx/wasmtime wrappers;
/// this is the in-process registry they register into.
#[derive(Debug, Clone)]
pub struct ModuleHandle {
    pub name: String,
    pub kind: RuntimeKind,
}

#[derive(Debug, Default)]
pub struct ChimeraFusion {
    runtimes: HashMap<RuntimeKind, Vec<ModuleHandle>>,
}

impl ChimeraFusion {
    pub fn new() -> Self {
        Self::default()
    }

    /// Absorb a foreign module into the unified process.
    pub fn absorb(&mut self, handle: ModuleHandle) {
        self.runtimes.entry(handle.kind).or_default().push(handle);
    }

    /// List modules absorbed per runtime.
    pub fn census(&self) -> Vec<(RuntimeKind, usize)> {
        self.runtimes
            .iter()
            .map(|(k, v)| (*k, v.len()))
            .collect()
    }

    /// Dispatch a logical call to the first absorbed module that claims `name`.
    pub fn call(&self, name: &str) -> Result<RuntimeKind, FusionError> {
        for (k, mods) in &self.runtimes {
            if mods.iter().any(|m| m.name == name) {
                return Ok(*k);
            }
        }
        Err(FusionError::NotAbsorbed(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absorb_and_call() {
        let mut f = ChimeraFusion::new();
        f.absorb(ModuleHandle { name: "matmul".into(), kind: RuntimeKind::Python });
        assert_eq!(f.call("matmul").unwrap(), RuntimeKind::Python);
        assert!(matches!(f.call("missing"), Err(FusionError::NotAbsorbed(_))));
    }
}
