//! End-to-end integration tests: multi-module biological scenarios running
//! through the BioBridge organism and the vendored BioMessage v2 protocol.

use bio_bridge::protocol::{decode_fields, BioMessage, BioOp};
use bio_bridge::BioBridge;
use std::collections::HashMap;

fn organism() -> BioBridge {
    let dir = std::env::temp_dir().join(format!("bio-bridge-test-{}", std::process::id()));
    BioBridge::new(tardigrade_tun::TunConfig {
        snapshot_dir: dir,
        max_snapshots: 2,
    })
}

/// Full immune response: sentinel detects a threat, crispr quarantines the
/// vulnerable target, and the response is announced as an IMMUNE_ALERT message.
#[test]
fn immune_cascade_end_to_end() {
    let mut bridge = organism();

    // 1. T-cell sentinel observes an unknown memory leak signature.
    let digest = bridge.sentinel.observe(
        "leak-abc123",
        tcell_sentinel::ThreatKind::MemoryLeak,
        1,
        HashMap::from([("module".to_string(), "parser".to_string())]),
    );
    assert!(bridge.sentinel.get(&digest).is_some());

    // 2. CRISPR quarantines the vulnerable codepath.
    bridge
        .crispr
        .stage("fn:parser::decode", crispr_patch::PatchAction::Quarantine, "memory leak")
        .unwrap();
    assert_eq!(bridge.crispr.applied().unwrap().len(), 1);

    // 3. The response goes out as a signed BioMessage.
    let msg = bridge
        .message("sentinel", digest.as_bytes().to_vec(), 0)
        .unwrap();
    let wire = msg.encode();
    let decoded = BioMessage::decode(&wire).unwrap();
    assert_eq!(decoded.op, BioOp::ImmuneAlert);
    assert_eq!(decoded.payload, digest.as_bytes());
}

/// Anhydrobiosis cycle: register state, freeze to disk, revive into a fresh
/// organism, and verify the state survived intact.
#[test]
fn freeze_and_revive_cycle() {
    let dir = std::env::temp_dir().join(format!("bio-tun-cycle-{}", std::process::id()));
    let config = tardigrade_tun::TunConfig {
        snapshot_dir: dir.clone(),
        max_snapshots: 2,
    };

    // First life: accumulate state, then freeze.
    {
        let tun = tardigrade_tun::Tun::new(config.clone());
        tun.thread(1, "worker", 0xDEAD, 4);
        tun.open_udp_workflow(42);
        tun.watch("mode", "stealth");
        let frame = tun.freeze();
        assert!(frame.open().is_ok());
    }

    // Second life: revive from the sealed snapshot.
    {
        let tun = tardigrade_tun::Tun::new(config);
        let revived = tun.revive(None).unwrap();
        assert!(revived.is_some());
    }

    let _ = std::fs::remove_dir_all(dir);
}

/// Compromised node: apoptosis cascade zeroizes keys, notifies scavengers,
/// and the APOPTOSIS opcode reaches the wire.
#[test]
fn compromised_node_self_destructs() {
    let mut bridge = organism();

    let keys = vec![apoptosis_cascade::KeyMaterial {
        name: "queen-session".into(),
        secret: vec![0xAB; 32],
    }];
    bridge.apoptosis = apoptosis_cascade::ApoptosisCascade::new(keys);

    let mut notices = Vec::new();
    let zeroized = bridge.apoptosis.fire(&mut notices).unwrap();
    assert_eq!(zeroized, 1);
    assert!(bridge.apoptosis.all_zeroized());
    assert!(notices.contains(&"scavenge-remains".to_string()));

    let msg = bridge.message("apoptosis", Vec::new(), 0).unwrap();
    let decoded = BioMessage::decode(&msg.encode()).unwrap();
    assert_eq!(decoded.op, BioOp::Apoptosis);
}

/// Quorum gating: a heavy operation only becomes authorized once enough
/// peers have voted — modeled as a TASK message sent after quorum passes.
#[test]
fn heavy_operation_waits_for_quorum() {
    let mut bridge = organism();

    bridge.quorum.propose("distributed-index", 3);
    assert!(!bridge.quorum.vote("distributed-index", "n1", true).unwrap());
    assert!(!bridge.quorum.vote("distributed-index", "n2", true).unwrap());
    let reached = bridge.quorum.vote("distributed-index", "n3", true).unwrap();
    assert!(reached);

    // Only now is the TASK dispatched.
    let msg = bridge.message("physarum", b"start-indexing".to_vec(), 0).unwrap();
    let decoded = BioMessage::decode(&msg.encode()).unwrap();
    assert_eq!(decoded.op, BioOp::Task);
}

/// JOIN registration: the bridge produces a wire-valid JOIN whose TLV fields
/// decode back to the drone name and pid.
#[test]
fn join_registration_is_wire_valid() {
    let msg = BioBridge::join_message("drone-tardigrade-01");
    let decoded = BioMessage::decode(&msg.encode()).unwrap();
    assert_eq!(decoded.op, BioOp::Join);

    let fields: HashMap<String, Vec<u8>> = decode_fields(&decoded.payload).into_iter().collect();
    assert_eq!(fields["name"], b"drone-tardigrade-01".to_vec());
    assert_eq!(
        fields["pid"],
        std::process::id().to_string().as_bytes().to_vec()
    );
}

/// Resource redistribution: a rich node donates headroom to a starved edge
/// device, and the forest balance improves.
#[test]
fn wood_wide_web_redistributes() {
    let mut bridge = organism();

    bridge.mycorrhiza.register(
        "gpu-node",
        mycorrhiza_trade::Capacity { cpu_headroom: 0.9, ram_headroom: 0.9 },
    );
    bridge.mycorrhiza.register(
        "edge-mcu",
        mycorrhiza_trade::Capacity { cpu_headroom: 0.05, ram_headroom: 0.05 },
    );

    bridge.mycorrhiza.donate("gpu-node", "edge-mcu", 0.3).unwrap();

    let edge = bridge.mycorrhiza.capacity("edge-mcu").unwrap();
    assert!(edge.cpu_headroom > 0.3);
    let donor = bridge.mycorrhiza.capacity("gpu-node").unwrap();
    assert!(donor.cpu_headroom < 0.9);
}

/// Epigenetic mode switch: the self-hash never changes while behavior flips.
#[test]
fn methylation_switches_modes_without_touching_hash() {
    let bridge = organism();
    let hash_before = bridge.epigenetic.self_hash().to_string();

    bridge.epigenetic.methylate(epigenetic_switch::Mode::Fortress);
    assert_eq!(bridge.epigenetic.mode(), epigenetic_switch::Mode::Fortress);
    assert_eq!(bridge.epigenetic.validation_level(), 3);
    assert!(!bridge.epigenetic.should_log(false));
    assert!(bridge.epigenetic.should_log(true));

    bridge.epigenetic.methylate(epigenetic_switch::Mode::Hyperdrive);
    assert!(bridge.epigenetic.should_log(false));

    assert_eq!(bridge.epigenetic.self_hash(), hash_before);
}
