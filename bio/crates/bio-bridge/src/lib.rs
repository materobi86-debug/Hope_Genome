//! Bio-Bridge — connects the 21 bio-module crates to the Bio-Binaries v2
//! BioMessage protocol.
//!
//! The protocol module is vendored from `Bio-Binaries/src/bio_protocol.rs`
//! (commit of v0.2.1) so the bridge stays dependency-light: no tokio, no
//! reqwest, no image codecs — just blake3 + std. The wire format is
//! byte-identical to what `omega-master` speaks on UDP port 8888.

pub mod protocol;

use protocol::{encode_fields, BioMessage, BioOp};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("module not mounted: {0}")]
    NotMounted(String),
}

/// The organism: all 21 bio modules mounted into one coherent body.
pub struct BioBridge {
    pub tardigrade: tardigrade_tun::Tun,
    pub physarum: physarum_path::Physarum,
    pub sentinel: tcell_sentinel::ImmuneDatabase,
    pub crispr: crispr_patch::PatchEngine,
    pub symbiont: symbiont_engine::SymbiontEngine,
    pub epigenetic: epigenetic_switch::EpigeneticSwitch,
    pub quorum: quorum_signal::QuorumTable,
    pub mycorrhiza: mycorrhiza_trade::Forest,
    pub plasmid: plasmid_conjugate::PlasmidStore,
    pub blastema: blastema_regen::BlastemaLedger,
    pub chameleon: chameleon_stealth::Chameleon,
    pub electrocyte: electrocyte_burst::ElectrocyteBurst,
    pub magnetosome: magnetosome_nav::MagnetosomeNav,
    pub xenobot: xenobot_swarm::XenobotSwarm,
    pub apoptosis: apoptosis_cascade::ApoptosisCascade,
    pub yamanaka: yamanaka_deaging::CellularState,
    pub morpho: morpho_electric_mesh::MorphoMesh,
    pub biophoton: biophoton_telepathy::TelepathyBus,
    pub transposon: transposon_hive::TransposonHive,
    pub vault: mycelial_void_vault::VoidVault,
    pub chimera: chimera_fusion::ChimeraFusion,
}

impl BioBridge {
    pub fn new(config: tardigrade_tun::TunConfig) -> Self {
        let mut quorum = quorum_signal::QuorumTable::new();
        quorum.propose("homeostasis", 1);
        BioBridge {
            tardigrade: tardigrade_tun::Tun::new(config),
            physarum: physarum_path::Physarum::new(0.1),
            sentinel: tcell_sentinel::ImmuneDatabase::new(),
            crispr: crispr_patch::PatchEngine::new(None),
            symbiont: symbiont_engine::SymbiontEngine::new(),
            epigenetic: epigenetic_switch::EpigeneticSwitch::new(
                "bio-bridge-v0",
                epigenetic_switch::Mode::Stealth,
            ),
            quorum,
            mycorrhiza: mycorrhiza_trade::Forest::new(),
            plasmid: plasmid_conjugate::PlasmidStore::new(),
            blastema: blastema_regen::BlastemaLedger::new(),
            chameleon: chameleon_stealth::Chameleon::new(chameleon_stealth::Disguise::Dns),
            electrocyte: electrocyte_burst::ElectrocyteBurst::new(),
            magnetosome: magnetosome_nav::MagnetosomeNav::new(),
            xenobot: xenobot_swarm::XenobotSwarm::new(),
            apoptosis: apoptosis_cascade::ApoptosisCascade::new(Vec::new()),
            yamanaka: yamanaka_deaging::CellularState::default(),
            morpho: morpho_electric_mesh::MorphoMesh::new(),
            biophoton: biophoton_telepathy::TelepathyBus::new(),
            transposon: transposon_hive::TransposonHive::new(),
            vault: mycelial_void_vault::VoidVault,
            chimera: chimera_fusion::ChimeraFusion::new(),
        }
    }

    /// Map a module event to a BioOp for omega-master routing.
    pub fn to_op(module_name: &str) -> Option<BioOp> {
        match module_name {
            "tardigrade" => Some(BioOp::Freeze),
            "crispr" => Some(BioOp::CrisprPatch),
            "sentinel" => Some(BioOp::ImmuneAlert),
            "apoptosis" => Some(BioOp::Apoptosis),
            "physarum" => Some(BioOp::Task),
            "morpho" => Some(BioOp::HomeoSync),
            _ => Some(BioOp::Heartbeat),
        }
    }

    /// Build a real protocol message for a module event.
    pub fn message(
        &self,
        module_name: &str,
        payload: Vec<u8>,
        generation: u32,
    ) -> Result<BioMessage, BridgeError> {
        let op = Self::to_op(module_name)
            .ok_or_else(|| BridgeError::NotMounted(module_name.to_string()))?;
        Ok(BioMessage::new(op, generation, payload))
    }

    /// Build a JOIN message for drone registration with omega-master.
    pub fn join_message(name: &str) -> BioMessage {
        let payload = encode_fields(&[
            ("name", name.as_bytes()),
            ("pid", std::process::id().to_string().as_bytes()),
        ]);
        BioMessage::new(BioOp::Join, 0, payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_mounts_all_modules() {
        let mut bridge = BioBridge::new(tardigrade_tun::TunConfig::default());
        // Every module reachable: exercise one method per module family.
        bridge.tardigrade.watch("k", "v");
        bridge.sentinel.observe("d", tcell_sentinel::ThreatKind::Unknown, 0, Default::default());
        bridge.quorum.vote("homeostasis", "node-1", true).unwrap();
        bridge.electrocyte.contribute(1, 10);
        assert_eq!(bridge.electrocyte.fire().unwrap(), 10);
    }

    #[test]
    fn module_events_map_to_bio_ops() {
        assert_eq!(BioBridge::to_op("tardigrade"), Some(BioOp::Freeze));
        assert_eq!(BioBridge::to_op("crispr"), Some(BioOp::CrisprPatch));
        assert_eq!(BioBridge::to_op("sentinel"), Some(BioOp::ImmuneAlert));
        assert_eq!(BioBridge::to_op("apoptosis"), Some(BioOp::Apoptosis));
    }

    #[test]
    fn message_encodes_to_wire_format() {
        let bridge = BioBridge::new(tardigrade_tun::TunConfig::default());
        let msg = bridge.message("tardigrade", b"freeze-now".to_vec(), 0).unwrap();
        let wire = msg.encode();
        let decoded = BioMessage::decode(&wire).unwrap();
        assert_eq!(decoded.op, BioOp::Freeze);
        assert_eq!(decoded.payload, b"freeze-now");
    }

    #[test]
    fn join_message_roundtrips() {
        let msg = BioBridge::join_message("drone-42");
        let decoded = BioMessage::decode(&msg.encode()).unwrap();
        assert_eq!(decoded.op, BioOp::Join);
        let fields = protocol::decode_fields(&decoded.payload);
        assert_eq!(fields[0], ("name".to_string(), b"drone-42".to_vec()));
    }
}
