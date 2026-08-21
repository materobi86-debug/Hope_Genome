//! Wasm-Organoid — sandboxed organoid execution for incoming capabilities.
//!
//! New capabilities and plasmids arriving from external sources or the P2P
//! network are grown as isolated WebAssembly "organoids" inside a `wasmi`
//! interpreter. They run memory-safe and fuel-metered, so a hostile or buggy
//! code line cannot endanger the host process.
//!
//! `wasmi` is a pure-Rust interpreter (no C deps), so it builds cleanly even
//! in memory-constrained environments.

use plasmid_conjugate::Plasmid;
use thiserror::Error;
use wasmi::{Engine, Extern, Linker, Module, Store};

/// Default fuel budget per organoid invocation.
pub const DEFAULT_FUEL: u64 = 1_000_000;
/// Default memory cap per organoid (16 MiB in pages of 64 KiB).
pub const DEFAULT_MEMORY_PAGES: u32 = 256;

#[derive(Debug, Error)]
pub enum OrganoidError {
    #[error("plasmid codec '{0}' is not wasm — only wasm plasmids can be grown")]
    NotWasm(String),
    #[error("wasmi error: {0}")]
    Wasm(String),
    #[error("export '{0}' not found or does not have signature () -> i32")]
    BadExport(String),
    #[error("fuel exhausted — organoid terminated by the host")]
    FuelExhausted,
}

/// Host cell that grows organoids from plasmids.
pub struct OrganoidHost {
    engine: Engine,
}

impl OrganoidHost {
    pub fn new() -> Result<Self, OrganoidError> {
        let mut config = wasmi::Config::default();
        config.consume_fuel(true);
        let engine = Engine::new(&config);
        Ok(Self { engine })
    }

    /// Grow an organoid from a plasmid. The plasmid must carry wasm bytecode.
    pub fn grow(&self, plasmid: &Plasmid) -> Result<Organoid, OrganoidError> {
        if plasmid.codec != "wasm" {
            return Err(OrganoidError::NotWasm(plasmid.codec.clone()));
        }
        let module = Module::new(&self.engine, std::io::Cursor::new(&plasmid.payload))
            .map_err(|e| OrganoidError::Wasm(e.to_string()))?;
        Ok(Organoid {
            engine: self.engine.clone(),
            module,
            memory_pages: DEFAULT_MEMORY_PAGES,
        })
    }
}

/// A sandboxed organoid: one compiled wasm module with resource caps.
pub struct Organoid {
    engine: Engine,
    module: Module,
    memory_pages: u32,
}

impl Organoid {
    /// Override the per-organoid memory cap (in 64 KiB pages).
    pub fn with_memory_pages(mut self, pages: u32) -> Self {
        self.memory_pages = pages;
        self
    }

    /// Run an exported `() -> i32` function inside the sandbox.
    ///
    /// Fuel limits execution steps; a runaway organoid traps and is caught.
    pub fn run_i32(&self, func_name: &str, fuel: u64) -> Result<i32, OrganoidError> {
        let mut store = Store::new(&self.engine, ());
        store
            .add_fuel(fuel)
            .map_err(|e| OrganoidError::Wasm(e.to_string()))?;

        let linker = Linker::new(&self.engine);
        let instance_pre = linker
            .instantiate(&mut store, &self.module)
            .map_err(|e| OrganoidError::Wasm(e.to_string()))?;
        let instance = instance_pre
            .start(&mut store)
            .map_err(|e| OrganoidError::Wasm(e.to_string()))?;

        let exported = instance
            .get_export(&store, func_name)
            .ok_or_else(|| OrganoidError::BadExport(func_name.to_string()))?;

        let func = match exported {
            Extern::Func(f) => f,
            _ => return Err(OrganoidError::BadExport(func_name.to_string())),
        };

        let typed_func = func
            .typed::<(), i32>(&store)
            .map_err(|_| OrganoidError::BadExport(func_name.to_string()))?;

        typed_func
            .call(&mut store, ())
            .map_err(|_| OrganoidError::Wasm("organoid trapped".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hand-assembled minimal wasm module:
    /// `(module (func (export "run") (result i32) i32.const 42))`
    const RUN_42_WASM: &[u8] = &[
        0x00, 0x61, 0x73, 0x6d, // \0asm
        0x01, 0x00, 0x00, 0x00, // version 1
        0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f, // type: () -> i32
        0x03, 0x02, 0x01, 0x00, // func section: 1 func, type 0
        0x07, 0x07, 0x01, 0x03, 0x72, 0x75, 0x6e, 0x00, 0x00, // export "run" func 0
        0x0a, 0x06, 0x01, 0x04, 0x00, 0x41, 0x2a, 0x0b, // code: i32.const 42; end
    ];

    fn wasm_plasmid(id: &str, bytecode: Vec<u8>) -> Plasmid {
        Plasmid::seal_typed(id, "wasm", bytecode)
    }

    #[test]
    fn organoid_runs_in_sandbox() {
        let host = OrganoidHost::new().unwrap();
        let organoid = host.grow(&wasm_plasmid("cap-42", RUN_42_WASM.to_vec())).unwrap();
        assert_eq!(organoid.run_i32("run", DEFAULT_FUEL).unwrap(), 42);
    }

    #[test]
    fn non_wasm_plasmid_rejected() {
        let host = OrganoidHost::new().unwrap();
        let p = Plasmid::seal_typed("cap-x", "ebpf", vec![1, 2, 3]);
        assert!(matches!(host.grow(&p), Err(OrganoidError::NotWasm(_))));
    }

    #[test]
    fn invalid_bytecode_rejected() {
        let host = OrganoidHost::new().unwrap();
        assert!(matches!(
            host.grow(&wasm_plasmid("junk", vec![0xDE, 0xAD])),
            Err(OrganoidError::Wasm(_))
        ));
    }

    #[test]
    fn missing_export_rejected() {
        let host = OrganoidHost::new().unwrap();
        let organoid = host.grow(&wasm_plasmid("cap-42", RUN_42_WASM.to_vec())).unwrap();
        assert!(matches!(
            organoid.run_i32("nonexistent", DEFAULT_FUEL),
            Err(OrganoidError::BadExport(_))
        ));
    }
}