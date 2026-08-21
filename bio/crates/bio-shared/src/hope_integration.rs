//! Integration module between Bio-Binaries ecosystem and Hope Genome framework.
//!
//! Provides Hope Genome Watchdog & Integrity Proof support for Bio-Binaries frames and events.

use _hope_core as hope_core;
use hope_core::{Action, SealedGenome};

/// Wraps a Hope Genome SealedGenome to verify Bio-Binaries frames or actions.
pub struct BioHopeGenome {
    pub genome: SealedGenome,
}

impl BioHopeGenome {
    /// Create and seal a new BioHopeGenome with ethical rules.
    pub fn new(rules: Vec<String>) -> Result<Self, hope_core::genome::GenomeError> {
        let mut genome = SealedGenome::new(rules)?;
        genome.seal()?;
        Ok(Self { genome })
    }

    /// Verify a Bio-Binaries action using Hope Genome Ed25519 signatures.
    pub fn verify_bio_action(&self, action_name: &str) -> Option<hope_core::IntegrityProof> {
        let action = Action::execute(action_name);
        self.genome.verify_action(&action).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hope_core::VerificationStatus;

    #[test]
    fn test_bio_hope_genome_integration() {
        let bio_genome = BioHopeGenome::new(vec![
            "Do no harm".to_string(),
            "Protect bio-mesh integrity".to_string(),
        ]).unwrap();

        let proof = bio_genome.verify_bio_action("bio_op:freeze_frame").unwrap();
        assert_eq!(proof.status, VerificationStatus::OK);
        assert_eq!(proof.signature.len(), 64); // Ed25519 signature from hope_core
    }
}
