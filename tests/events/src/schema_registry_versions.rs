use crate::harness::Deployment;
use earnproof_shared::{
    EventSchemaBounds, ISSUER_REGISTRY_EVENT_SCHEMA_VERSION, PROOF_REGISTRY_EVENT_SCHEMA_VERSION,
    PROTOCOL_CONFIG_EVENT_SCHEMA_VERSION,
};
use soroban_sdk::testutils::Ledger as _;

#[test]
fn contracts_expose_canonical_event_schema_registry_versions() {
    let deployment = Deployment::new();

    assert_eq!(
        deployment.config.event_schema_version(),
        PROTOCOL_CONFIG_EVENT_SCHEMA_VERSION
    );
    assert_eq!(
        deployment.issuers.event_schema_version(),
        ISSUER_REGISTRY_EVENT_SCHEMA_VERSION
    );
    assert_eq!(
        deployment.proofs.event_schema_version(),
        PROOF_REGISTRY_EVENT_SCHEMA_VERSION
    );
}

#[test]
fn contracts_expose_supported_event_schema_bounds() {
    let deployment = Deployment::new();

    let expected_config_bounds = EventSchemaBounds {
        min_version: 1,
        max_version: PROTOCOL_CONFIG_EVENT_SCHEMA_VERSION,
        current_version: PROTOCOL_CONFIG_EVENT_SCHEMA_VERSION,
    };
    let expected_issuers_bounds = EventSchemaBounds {
        min_version: 1,
        max_version: ISSUER_REGISTRY_EVENT_SCHEMA_VERSION,
        current_version: ISSUER_REGISTRY_EVENT_SCHEMA_VERSION,
    };
    let expected_proofs_bounds = EventSchemaBounds {
        min_version: 1,
        max_version: PROOF_REGISTRY_EVENT_SCHEMA_VERSION,
        current_version: PROOF_REGISTRY_EVENT_SCHEMA_VERSION,
    };

    assert_eq!(
        deployment.config.event_schema_bounds(),
        expected_config_bounds
    );
    assert_eq!(
        deployment.issuers.event_schema_bounds(),
        expected_issuers_bounds
    );
    assert_eq!(
        deployment.proofs.event_schema_bounds(),
        expected_proofs_bounds
    );
}

#[test]
fn exact_boundary_checks_for_supported_versions() {
    let deployment = Deployment::new();

    for client in [
        &deployment.config.event_schema_bounds(),
        &deployment.issuers.event_schema_bounds(),
        &deployment.proofs.event_schema_bounds(),
    ] {
        assert_eq!(client.min_version, 1);
        assert_eq!(client.max_version, 1);
    }

    // Supported version (exact match)
    assert!(deployment.config.is_event_schema_supported(&1));
    assert!(deployment.issuers.is_event_schema_supported(&1));
    assert!(deployment.proofs.is_event_schema_supported(&1));

    // Below lower bound (exact boundary - 1)
    assert!(!deployment.config.is_event_schema_supported(&0));
    assert!(!deployment.issuers.is_event_schema_supported(&0));
    assert!(!deployment.proofs.is_event_schema_supported(&0));

    // Above upper bound (exact boundary + 1)
    assert!(!deployment.config.is_event_schema_supported(&2));
    assert!(!deployment.issuers.is_event_schema_supported(&2));
    assert!(!deployment.proofs.is_event_schema_supported(&2));

    // Max possible boundary
    assert!(!deployment.config.is_event_schema_supported(&u32::MAX));
    assert!(!deployment.issuers.is_event_schema_supported(&u32::MAX));
    assert!(!deployment.proofs.is_event_schema_supported(&u32::MAX));
}

#[test]
fn queries_remain_stable_across_mutations_and_additive_events() {
    let deployment = Deployment::new();

    let initial_config_bounds = deployment.config.event_schema_bounds();
    let initial_issuers_bounds = deployment.issuers.event_schema_bounds();
    let initial_proofs_bounds = deployment.proofs.event_schema_bounds();

    // Perform operational mutations that emit events
    deployment.config.pause();
    deployment.config.unpause();
    deployment.config.approve_schema_version(&7);
    deployment.register_proof(0x42);

    assert_eq!(
        deployment.config.event_schema_bounds(),
        initial_config_bounds
    );
    assert_eq!(
        deployment.issuers.event_schema_bounds(),
        initial_issuers_bounds
    );
    assert_eq!(
        deployment.proofs.event_schema_bounds(),
        initial_proofs_bounds
    );
}

#[test]
fn unauthenticated_indexer_can_query_schema_registry() {
    let deployment = Deployment::new();

    // Disable mock authorizations to confirm queries do not require admin auth
    deployment.env.set_auths(&[]);

    assert_eq!(deployment.config.event_schema_version(), 1);
    assert_eq!(deployment.issuers.event_schema_version(), 1);
    assert_eq!(deployment.proofs.event_schema_version(), 1);
    assert!(deployment.config.is_event_schema_supported(&1));
}

#[test]
fn schema_registry_queries_are_deterministic_across_ledgers() {
    let deployment = Deployment::new();

    let v1 = deployment.config.event_schema_version();
    let b1 = deployment.config.event_schema_bounds();

    // Advance ledger far into the future
    deployment.env.ledger().set_sequence_number(999_999);
    deployment.env.ledger().set_timestamp(5_000_000);

    let v2 = deployment.config.event_schema_version();
    let b2 = deployment.config.event_schema_bounds();

    assert_eq!(v1, v2);
    assert_eq!(b1, b2);
}
