//! Capability bitmap tests (#293): every claimed bit maps to an entrypoint
//! that dispatches, the report is valid for its bitmap version, and the
//! bitmap does not change when the stored contract version changes.

use earnproof_shared::capabilities::*;
use earnproof_shared::PauseScope;
use earnproof_shared::{
    capabilities_are_valid, PROOF_REGISTRY_CAPABILITIES, PROOF_REGISTRY_INTERFACE_VERSION,
};
use issuer_registry::{IssuerRegistryContract, IssuerRegistryContractClient};
use proof_registry::{ProofRegistryContract, ProofRegistryContractClient};
use protocol_config::{ProtocolConfigContract, ProtocolConfigContractClient};
use soroban_sdk::{contracttype, testutils::Address as _, Address, BytesN, Env};

/// Local mirror of the contract's instance key, used only to simulate the
/// version bump an upgrade performs.
#[contracttype]
enum UpgradeKey {
    ContractVersion,
}

fn hash(env: &Env, value: u8) -> BytesN<32> {
    BytesN::from_array(env, &[value; 32])
}

fn setup() -> (Env, ProofRegistryContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let config_id = env.register(ProtocolConfigContract, ());
    ProtocolConfigContractClient::new(&env, &config_id).initialize(&admin);
    let issuer_id = env.register(IssuerRegistryContract, ());
    IssuerRegistryContractClient::new(&env, &issuer_id).initialize(&admin);
    let id = env.register(ProofRegistryContract, ());
    let client = ProofRegistryContractClient::new(&env, &id);
    client.initialize(&admin, &issuer_id, &config_id);
    (env, client, id)
}

/// Calls a representative entrypoint for `capability`. Returns `false` when no
/// probe exists, which fails the test if the contract claims the bit.
fn probe(env: &Env, client: &ProofRegistryContractClient<'static>, capability: u32) -> bool {
    let _ = env;
    match capability {
        CAP_TWO_STEP_ADMIN_TRANSFER => {
            client.get_admin_transfer_status();
            true
        }
        CAP_MIGRATION_STATUS => {
            client.get_migration_status();
            true
        }
        CAP_SUCCESSOR_DECOMMISSION => {
            client.get_successor();
            true
        }
        CAP_UPGRADE_TIMELOCK => {
            client.is_upgrade_allowed(&hash(env, 1));
            true
        }
        CAP_CONFIG_DIGEST => {
            let _ = client.try_get_config_digest();
            true
        }
        CAP_TTL_STATUS_QUERY => {
            client.get_instance_ttl_status();
            true
        }
        CAP_PAUSE_ACKNOWLEDGEMENTS => {
            let _ = client.try_get_pause_acknowledgements(&0, &1);
            true
        }
        CAP_SCOPED_PAUSE => {
            client.pause_scope(&PauseScope::Disputes);
            client.unpause_scope(&PauseScope::Disputes);
            true
        }
        CAP_PROOF_DISPUTES => {
            let _ = client.try_get_dispute(&hash(env, 3));
            true
        }
        CAP_PROOF_ARCHIVAL => {
            client.is_archived(&hash(env, 3));
            true
        }
        CAP_CAPABILITY_BITMAP => {
            client.get_capabilities();
            true
        }
        CAP_STORAGE_LAYOUT_QUERY => {
            client.get_storage_layout();
            true
        }
        CAP_DEPLOYMENT_IDENTITY => {
            client.get_deployment_identity();
            true
        }
        CAP_DEPENDENCY_IDENTITY_VALIDATION => {
            let _ = client.try_initialize_in_domain(
                &Address::generate(env),
                &client.get_issuer_registry(),
                &client.get_protocol_config(),
                &hash(env, 4),
            );
            true
        }
        CAP_DECOMMISSION_GRACE => {
            client.get_decommission_state();
            true
        }
        _ => false,
    }
}

#[test]
fn every_claimed_capability_has_a_dispatching_entrypoint() {
    let (env, client, _id) = setup();
    for capability in 0..64u32 {
        if client.has_capability(&capability) {
            assert!(
                probe(&env, &client, capability),
                "capability {capability} is claimed but has no entrypoint probe"
            );
        }
    }
}

#[test]
fn report_matches_the_governed_constants() {
    let (_env, client, _id) = setup();
    let report = client.get_capabilities();
    assert_eq!(report.bitmap_version, CAPABILITY_BITMAP_VERSION);
    assert_eq!(report.interface_version, PROOF_REGISTRY_INTERFACE_VERSION);
    assert_eq!(report.contract_version, 1);
    assert_eq!(report.capabilities, PROOF_REGISTRY_CAPABILITIES);
    assert!(capabilities_are_valid(
        report.bitmap_version,
        report.capabilities
    ));
    for spec in CAPABILITY_REGISTRY.iter() {
        assert_eq!(
            client.has_capability(&spec.id),
            report.capabilities & capability_bit(spec.id) != 0,
            "{}",
            spec.name
        );
    }
}

#[test]
fn undefined_and_out_of_range_identifiers_are_unsupported() {
    let (_env, client, _id) = setup();
    assert!(!client.has_capability(&CAPABILITY_COUNT_V1));
    assert!(!client.has_capability(&63));
    assert!(!client.has_capability(&64));
    assert!(!client.has_capability(&u32::MAX));
    assert!(client.has_capability(&CAP_CAPABILITY_BITMAP));
}

#[test]
fn bitmap_is_stable_across_contract_version_changes() {
    let (env, client, id) = setup();
    let before = client.get_capabilities();
    env.as_contract(&id, || {
        env.storage()
            .instance()
            .set(&UpgradeKey::ContractVersion, &7_u32)
    });
    let after = client.get_capabilities();
    assert_eq!(after.contract_version, 7);
    assert_eq!(after.capabilities, before.capabilities);
    assert_eq!(after.bitmap_version, before.bitmap_version);
    assert_eq!(after.interface_version, before.interface_version);
    // Repeated reads are identical and require no authorization.
    env.set_auths(&[]);
    assert_eq!(client.get_capabilities(), after);
}

#[test]
fn uninitialized_contract_reports_version_zero_with_the_same_bitmap() {
    let env = Env::default();
    let id = env.register(ProofRegistryContract, ());
    let client = ProofRegistryContractClient::new(&env, &id);
    let report = client.get_capabilities();
    assert_eq!(report.contract_version, 0);
    assert_eq!(report.capabilities, PROOF_REGISTRY_CAPABILITIES);
}
