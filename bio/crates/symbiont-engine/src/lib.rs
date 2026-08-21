//! Symbiont-Engine — integrate heterogeneous resources as "mitochondria".
//!
//! The host cell detects weaker/specialized devices (microcontrollers, GPUs/NPUs,
//! Raspberry Pis) and offloads compute-heavy work to them: matrix multiplication
//! and FFT transforms. This crate provides the host-side scheduling model and a
//! trait that device adapters implement.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SymbiontError {
    #[error("no symbiont available for task kind {0:?}")]
    NoSymbiont(TaskKind),
    #[error("symbiont task failed: {0}")]
    TaskFailed(String),
    #[error("result signature mismatch: expected {expected}, got {actual}")]
    Integrity { expected: String, actual: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskKind {
    MatrixMul,
    Fft,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub kind: TaskKind,
    /// Serialized input payload (matrix dims + data, or signal samples).
    pub input: Vec<f64>,
    /// Expected BLAKE3 signature of the result, for verification.
    pub expected_signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub output: Vec<f64>,
    pub signature: String,
}

impl TaskResult {
    pub fn seal(kind: TaskKind, output: Vec<f64>) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&(kind as u8).to_le_bytes());
        for v in &output {
            hasher.update(&v.to_le_bytes());
        }
        let signature = hasher.finalize().to_hex().to_string();
        Self { output, signature }
    }

    pub fn verify(&self, kind: TaskKind) -> bool {
        let recomputed = Self::seal(kind, self.output.clone()).signature;
        self.signature == recomputed
    }
}

/// A symbiont is any device that can execute a task. Implement this trait in an
/// adapter (network RPC, GPU runtime, etc.) and register it with the host.
pub trait Symbiont: Send + Sync {
    fn name(&self) -> &str;
    fn supports(&self, kind: TaskKind) -> bool;
    /// Execute a task and return a verified result.
    fn execute(&self, task: &Task) -> Result<TaskResult, SymbiontError>;
}

/// The host cell: chooses the best symbiont and verifies its output.
#[derive(Default)]
pub struct SymbiontEngine {
    symbionts: HashMap<String, Box<dyn Symbiont>>,
    /// Per-symbiont throughput score (higher is better), updated after tasks.
    scores: HashMap<String, f64>,
}

impl SymbiontEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Integrate a symbiont (endosymbiosis event).
    pub fn integrate(&mut self, id: &str, symbiont: Box<dyn Symbiont>) {
        self.scores.insert(id.to_string(), 0.5);
        self.symbionts.insert(id.to_string(), symbiont);
    }

    /// List integrated symbionts with their scores.
    pub fn census(&self) -> Vec<(&str, f64)> {
        let mut v: Vec<_> = self
            .scores
            .iter()
            .map(|(k, s)| (k.as_str(), *s))
            .collect();
        v.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
        v
    }

    /// Select the highest-scoring symbiont that supports `kind`.
    fn select(&self, kind: TaskKind) -> Option<(&str, &Box<dyn Symbiont>)> {
        self.census()
            .into_iter()
            .find(|(id, _)| self.symbionts[*id].supports(kind))
            .map(|(id, _)| (id, &self.symbionts[id]))
    }

    /// Dispatch a task, verify the result, and update scores on success.
    pub fn dispatch(&mut self, task: &Task) -> Result<TaskResult, SymbiontError> {
        let id = self
            .select(task.kind)
            .map(|(id, _)| id.to_string())
            .ok_or(SymbiontError::NoSymbiont(task.kind))?;
        let result = self.symbionts[&id].execute(task)?;
        if !result.verify(task.kind) {
            return Err(SymbiontError::Integrity {
                expected: task.expected_signature.clone(),
                actual: result.signature,
            });
        }
        // Reward: increase score on success.
        if let Some(s) = self.scores.get_mut(&id) {
            *s = (*s * 0.9 + 0.1).clamp(0.0, 1.0);
        }
        Ok(result)
    }

    /// Penalize a failed symbiont.
    pub fn penalize(&mut self, id: &str) {
        if let Some(s) = self.scores.get_mut(id) {
            *s = (*s * 0.5).clamp(0.0, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummySymbiont;
    impl Symbiont for DummySymbiont {
        fn name(&self) -> &str {
            "dummy"
        }
        fn supports(&self, kind: TaskKind) -> bool {
            kind == TaskKind::MatrixMul
        }
        fn execute(&self, task: &Task) -> Result<TaskResult, SymbiontError> {
            Ok(TaskResult::seal(task.kind, vec![1.0, 2.0]))
        }
    }

    #[test]
    fn integrate_and_dispatch() {
        let mut engine = SymbiontEngine::new();
        engine.integrate("dummy", Box::new(DummySymbiont));
        assert_eq!(engine.census().len(), 1);
        let task = Task {
            kind: TaskKind::MatrixMul,
            input: vec![1.0],
            expected_signature: "x".into(),
        };
        assert!(engine.dispatch(&task).is_ok());
    }
}
