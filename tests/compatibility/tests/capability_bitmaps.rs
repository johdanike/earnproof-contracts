//! Capability bitmap compatibility gate (#293).
//!
//! `goldens/capabilities.json` records every governed capability identifier
//! and the bits each contract claimed when the golden was last updated. Within
//! one bitmap version the current build must keep every identifier's number
//! and name and must keep every bit a contract already claimed.
//!
//! Run: cargo test -p contract-compatibility-gate --test capability_bitmaps

use earnproof_shared::capabilities::{
    capabilities_are_additive, capabilities_are_valid, capability_bit, CAPABILITY_BITMAP_VERSION,
    CAPABILITY_REGISTRY,
};
use earnproof_shared::{
    ISSUER_REGISTRY_CAPABILITIES, PROOF_REGISTRY_CAPABILITIES, PROTOCOL_CONFIG_CAPABILITIES,
};
use std::collections::BTreeMap;

#[derive(serde::Deserialize)]
struct Golden {
    bitmap_version: u32,
    capabilities: Vec<GoldenCapability>,
    contracts: BTreeMap<String, Vec<u32>>,
}

#[derive(serde::Deserialize)]
struct GoldenCapability {
    id: u32,
    name: String,
}

fn golden() -> Golden {
    let text = std::fs::read_to_string("goldens/capabilities.json")
        .expect("goldens/capabilities.json is readable");
    serde_json::from_str(&text).expect("goldens/capabilities.json is valid")
}

fn bitmap(ids: &[u32]) -> u64 {
    ids.iter().fold(0, |bits, id| bits | capability_bit(*id))
}

fn current(contract: &str) -> u64 {
    match contract {
        "issuer-registry" => ISSUER_REGISTRY_CAPABILITIES,
        "proof-registry" => PROOF_REGISTRY_CAPABILITIES,
        "protocol-config" => PROTOCOL_CONFIG_CAPABILITIES,
        other => panic!("unknown contract in capability golden: {other}"),
    }
}

#[test]
fn golden_matches_the_current_bitmap_version() {
    assert_eq!(golden().bitmap_version, CAPABILITY_BITMAP_VERSION);
}

#[test]
fn identifiers_are_never_renumbered_or_renamed() {
    let golden = golden();
    for entry in &golden.capabilities {
        let spec = CAPABILITY_REGISTRY
            .iter()
            .find(|spec| spec.id == entry.id)
            .unwrap_or_else(|| panic!("capability {} was removed", entry.id));
        assert_eq!(spec.name, entry.name, "capability {} was renamed", entry.id);
    }
    // New identifiers are appended after the golden ones.
    assert!(CAPABILITY_REGISTRY.len() >= golden.capabilities.len());
}

#[test]
fn contracts_never_drop_a_claimed_capability() {
    let golden = golden();
    assert_eq!(golden.contracts.len(), 3);
    for (contract, ids) in &golden.contracts {
        let previous = bitmap(ids);
        let now = current(contract);
        assert!(
            capabilities_are_valid(CAPABILITY_BITMAP_VERSION, now),
            "{contract} claims an undefined capability"
        );
        assert!(
            capabilities_are_additive(previous, now),
            "{contract} dropped capabilities {:#x}",
            previous & !now
        );
    }
}

#[test]
fn dropping_a_capability_would_be_detected() {
    let golden = golden();
    let ids = &golden.contracts["issuer-registry"];
    let previous = bitmap(ids);
    let dropped = previous & !capability_bit(ids[0]);
    assert!(!capabilities_are_additive(previous, dropped));
    assert!(capabilities_are_additive(previous, previous));
}
