//! Phoenix-Loop — auto self-healing watchdog + hot-patch loop.
//!
//! A watchdog observes decision/code-branch events against a policy. When a
//! branch attempts a rule violation, the loop does not merely deny it: it
//! hot-swaps the faulty branch for a verified, rule-compliant replacement via
//! `crispr-patch`, all at runtime, without restarting the host process.
//! Also integrates Hope Genome Ed25519 action verification.

use bio_shared::BioHopeGenome;
use crispr_patch::{PatchAction, PatchEngine};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PhoenixError {
    #[error("no verified replacement registered for branch {0}")]
    NoReplacement(String),
    #[error(transparent)]
    Patch(#[from] crispr_patch::PatchError),
}

/// A policy rule: a named predicate over branch events.
#[derive(Debug, Clone)]
pub struct Rule {
    pub name: String,
    /// Event keys that constitute a violation when present.
    pub forbidden_keys: Vec<String>,
}

/// An observed decision/code-branch event.
#[derive(Debug, Clone)]
pub struct BranchEvent {
    pub branch_id: String,
    pub attributes: HashMap<String, String>,
}

/// The verified replacement registry: branch_id -> replacement target id.
#[derive(Debug, Default)]
pub struct ReplacementRegistry {
    replacements: HashMap<String, String>,
}

impl ReplacementRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a verified, rule-compliant replacement for a branch.
    pub fn register(&mut self, faulty_branch: &str, verified_target: &str) {
        self.replacements
            .insert(faulty_branch.to_string(), verified_target.to_string());
    }

    pub fn replacement_for(&self, branch: &str) -> Option<&str> {
        self.replacements.get(branch).map(|s| s.as_str())
    }
}

/// The Phoenix loop: watchdog + crispr hot-patch in one cycle.
pub struct PhoenixLoop {
    pub rules: Vec<Rule>,
    pub replacements: ReplacementRegistry,
    pub crispr: PatchEngine,
    /// Audit trail of healed branches.
    pub healed: Vec<String>,
    /// Optional Hope Genome cryptographic proof engine.
    pub hope_genome: Option<BioHopeGenome>,
}

impl PhoenixLoop {
    pub fn new(rules: Vec<Rule>, replacements: ReplacementRegistry) -> Self {
        Self {
            rules,
            replacements,
            crispr: PatchEngine::new(None),
            healed: Vec::new(),
            hope_genome: None,
        }
    }

    /// Enable Hope Genome cryptographic verification with rules.
    pub fn with_hope_genome(mut self, rules: Vec<String>) -> Self {
        if let Ok(genome) = BioHopeGenome::new(rules) {
            self.hope_genome = Some(genome);
        }
        self
    }

    /// Evaluate one branch event. Returns true if the event was allowed.
    /// On violation: deny + hot-swap the branch to its verified replacement.
    pub fn observe(&mut self, event: &BranchEvent) -> Result<bool, PhoenixError> {
        let violation = self.rules.iter().any(|rule| {
            rule.forbidden_keys
                .iter()
                .any(|k| event.attributes.contains_key(k))
        });

        if !violation {
            // If Hope Genome is active, generate cryptographic proof
            if let Some(ref genome) = self.hope_genome {
                let _proof = genome.verify_bio_action(&event.branch_id);
            }
            return Ok(true);
        }

        // Deny + heal: swap the faulty branch for a verified one.
        let replacement = self
            .replacements
            .replacement_for(&event.branch_id)
            .ok_or_else(|| PhoenixError::NoReplacement(event.branch_id.clone()))?
            .to_string();

        self.crispr.stage(
            &event.branch_id,
            PatchAction::Quarantine,
            "phoenix: rule violation detected",
        )?;
        self.crispr.stage(
            &replacement,
            PatchAction::Restore,
            "phoenix: verified replacement activated",
        )?;
        self.healed.push(event.branch_id.clone());
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> PhoenixLoop {
        let rules = vec![Rule {
            name: "no-unsafe-write".into(),
            forbidden_keys: vec!["unsafe_write".into()],
        }];
        let mut reg = ReplacementRegistry::new();
        reg.register("branch:ai-decision-7", "branch:verified-decision-7");
        PhoenixLoop::new(rules, reg).with_hope_genome(vec!["Do no harm".to_string()])
    }

    #[test]
    fn compliant_event_passes() {
        let mut phoenix = setup();
        let event = BranchEvent {
            branch_id: "branch:ai-decision-7".into(),
            attributes: HashMap::from([("action".into(), "read".into())]),
        };
        assert!(phoenix.observe(&event).unwrap());
        assert!(phoenix.healed.is_empty());
        assert!(phoenix.hope_genome.is_some());
    }

    #[test]
    fn violation_is_denied_and_healed() {
        let mut phoenix = setup();
        let event = BranchEvent {
            branch_id: "branch:ai-decision-7".into(),
            attributes: HashMap::from([("unsafe_write".into(), "/etc/passwd".into())]),
        };
        assert!(!phoenix.observe(&event).unwrap());
        assert_eq!(phoenix.healed, vec!["branch:ai-decision-7"]);
        // Both quarantine and replacement are staged.
        assert_eq!(phoenix.crispr.applied().unwrap().len(), 2);
    }

    #[test]
    fn violation_without_replacement_errors() {
        let mut phoenix = setup();
        let event = BranchEvent {
            branch_id: "branch:unknown".into(),
            attributes: HashMap::from([("unsafe_write".into(), "x".into())]),
        };
        assert!(matches!(
            phoenix.observe(&event),
            Err(PhoenixError::NoReplacement(_))
        ));
    }
}
