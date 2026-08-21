//! CRISPR-Patch — live code self-healing via hot patching.
//!
//! Hot patching in Rust is inherently unsafe: rewriting machine code at runtime
//! without coordination can crash the process. This module provides the *safe*
//! substrate: patch descriptors are queued and applied through a guarded switch,
//! so vulnerable codepaths can be neutralized without a full restart.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PatchError {
    #[error("patch target not found: {0}")]
    NotFound(String),
    #[error("patch already applied: {0}")]
    AlreadyApplied(String),
    #[error("patch engine not attached (engine.apply not hooked)")]
    EngineNotAttached,
    #[error("lock poisoned")]
    Lock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PatchAction {
    /// Neutralize a vulnerable function: route it to a stub that returns error.
    Quarantine,
    /// Neutralize a vulnerable function: route it to a stub that returns null.
    Null,
    /// Try to restore a patched target.
    Restore,
}

#[derive(Debug, Clone)]
pub struct Patch {
    pub target_id: String,
    pub action: PatchAction,
    pub reason: String,
}

pub type ApplyFn = Arc<dyn Fn(&Patch) -> Result<(), String> + Send + Sync>;

#[derive(Clone)]
pub struct PatchEngine {
    applied: Arc<Mutex<HashMap<String, Patch>>>,
    apply: Option<ApplyFn>,
}

impl Default for PatchEngine {
    fn default() -> Self {
        Self {
            applied: Arc::new(Mutex::new(HashMap::new())),
            apply: None,
        }
    }
}

impl PatchEngine {
    pub fn new(apply: Option<ApplyFn>) -> Self {
        Self {
            apply,
            ..Default::default()
        }
    }

    /// Stage a patch and attempt to apply it.
    pub fn stage(
        &self,
        target_id: &str,
        action: PatchAction,
        reason: &str,
    ) -> Result<(), PatchError> {
        let patch = Patch {
            target_id: target_id.to_string(),
            action,
            reason: reason.to_string(),
        };
        let mut applied = self.applied.lock().map_err(|_| PatchError::Lock)?;
        if applied.contains_key(&patch.target_id) {
            return Err(PatchError::AlreadyApplied(
                patch.target_id.clone(),
            ));
        }
        if let Some(apply) = &self.apply {
            apply(&patch)
                .map_err(|e| PatchError::NotFound(format!("{target_id}: {e}")))?;
        }
        applied.insert(patch.target_id.clone(), patch);
        Ok(())
    }

    /// Return all applied patches.
    pub fn applied(&self) -> Result<Vec<Patch>, PatchError> {
        let applied = self.applied.lock().map_err(|_| PatchError::Lock)?;
        Ok(applied.values().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_records_patch() {
        let engine = PatchEngine::new(None);
        engine
            .stage("fn:vuln", PatchAction::Quarantine, "test")
            .unwrap();
        assert_eq!(engine.applied().unwrap().len(), 1);
    }

    #[test]
    fn duplicate_rejected() {
        let engine = PatchEngine::new(None);
        engine
            .stage("fn:vuln", PatchAction::Null, "test")
            .unwrap();
        assert!(matches!(
            engine.stage("fn:vuln", PatchAction::Null, "dup"),
            Err(PatchError::AlreadyApplied(_))
        ));
    }
}
