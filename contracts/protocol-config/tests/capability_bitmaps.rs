//! Capability bitmap tests (#293): every claimed bit maps to an entrypoint
//! that dispatches, the report is valid for its bitmap version, and the
//! bitmap does not change when the stored contract version changes.

use earnproof_shared::capabilities::*;
use earnproof_shared::PauseScope;
use earnproof_shared::{
    capabilities_are_valid, PROTOCOL_CONFIG_CAPABILITIES, PROTOCOL_CONFIG_INTERFACE_VERSION,
};
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

fn setup() -> (Env, ProtocolConfigContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register(ProtocolConfigContract, ());
    let client = ProtocolConfigContractClient::new(&env, &id);
    client.initialize(&Address::generate(&env));
    (env, client, id)
}

/// Calls a representative entrypoint for `capability`. Returns `false` when no
/// probe exists, which fails the test if the contract claims the bit.
fn probe(env: &Env, client: &ProtocolConfigContractClient<'static>, capability: u32) -> bool {
    let _ = env;
    match capability {
        CAP_INTERFACE_VERSION_QUERY => {
            client.interface_version();
            true
        }
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
        CAP_PAUSE_ACKNOWLEDGEMENTS => {
            let _ = client.try_get_pause_acknowledgements(&0, &1);
            true
        }
        CAP_SCOPED_PAUSE => {
            client.is_scope_paused(&PauseScope::Disputes);
            true
        }
        CAP_CRITICAL_ACTION_APPROVALS => {
            client.get_critical_action_policy();
            true
        }
        CAP_EMERGENCY_FAST_TRACK => {
            client.get_emergency_policy();
            true
        }
        CAP_SCHEMA_ACTIVATION_NOTICE => {
            client.get_min_schema_activation_notice();
            true
        }
        CAP_SCHEMA_REPLACEMENT_POINTERS => {
            client.get_schema_replacement(&1);
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
    assert_eq!(report.interface_version, PROTOCOL_CONFIG_INTERFACE_VERSION);
    assert_eq!(report.contract_version, 1);
    assert_eq!(report.capabilities, PROTOCOL_CONFIG_CAPABILITIES);
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
    let id = env.register(ProtocolConfigContract, ());
    let client = ProtocolConfigContractClient::new(&env, &id);
    let report = client.get_capabilities();
    assert_eq!(report.contract_version, 0);
    assert_eq!(report.capabilities, PROTOCOL_CONFIG_CAPABILITIES);
}
