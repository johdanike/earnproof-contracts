//! Governed registry of commitment hash algorithm identifiers.

use earnproof_shared::{HashAlgorithmStatus, ProtocolConfigError};
use protocol_config::{ProtocolConfigContract, ProtocolConfigContractClient};
use soroban_sdk::{testutils::Address as _, testutils::Ledger as _, Address, Env};

fn setup() -> (Env, ProtocolConfigContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1_000);
    let client = ProtocolConfigContractClient::new(&env, &env.register(ProtocolConfigContract, ()));
    client.initialize(&Address::generate(&env));
    (env, client)
}

#[test]
fn an_unknown_algorithm_can_neither_create_nor_verify() {
    let (_env, client) = setup();
    assert!(client.get_hash_algorithm(&7).is_none());
    assert!(!client.can_create_commitment(&7));
    assert!(!client.can_verify_commitment(&7));
    assert_eq!(
        client.try_check_hash_algorithm(&7, &true),
        Err(Ok(ProtocolConfigError::HashAlgorithmNotFound))
    );
    assert_eq!(
        client.try_activate_hash_algorithm(&7),
        Err(Ok(ProtocolConfigError::HashAlgorithmNotFound))
    );
    assert_eq!(
        client.try_deprecate_hash_algorithm(&7),
        Err(Ok(ProtocolConfigError::HashAlgorithmNotFound))
    );
}

#[test]
fn the_full_lifecycle_records_each_step() {
    let (env, client) = setup();
    client.approve_hash_algorithm(&7);
    let approved = client.get_hash_algorithm(&7).unwrap();
    assert_eq!(approved.status, HashAlgorithmStatus::Approved);
    assert_eq!(approved.approved_at, 1_000);
    assert_eq!(approved.revision, 1);
    // Approved is not yet usable.
    assert!(!client.can_create_commitment(&7));
    assert!(!client.can_verify_commitment(&7));
    assert_eq!(
        client.try_check_hash_algorithm(&7, &true),
        Err(Ok(ProtocolConfigError::HashAlgorithmNotActive))
    );

    env.ledger().set_timestamp(2_000);
    client.activate_hash_algorithm(&7);
    let active = client.get_hash_algorithm(&7).unwrap();
    assert_eq!(active.status, HashAlgorithmStatus::Active);
    assert_eq!(active.activated_at, 2_000);
    assert!(client.can_create_commitment(&7));
    assert!(client.can_verify_commitment(&7));

    env.ledger().set_timestamp(3_000);
    client.deprecate_hash_algorithm(&7);
    let deprecated = client.get_hash_algorithm(&7).unwrap();
    assert_eq!(deprecated.status, HashAlgorithmStatus::Deprecated);
    assert_eq!(deprecated.deprecated_at, 3_000);
    assert_eq!(deprecated.revision, 3);
    // Historical records stay verifiable; new commitments are refused.
    assert!(client.can_verify_commitment(&7));
    assert!(!client.can_create_commitment(&7));
    assert_eq!(
        client.try_check_hash_algorithm(&7, &true),
        Err(Ok(ProtocolConfigError::HashAlgorithmDeprecated))
    );
    client.check_hash_algorithm(&7, &false);
}

#[test]
fn lifecycle_steps_cannot_be_skipped_repeated_or_reversed() {
    let (_env, client) = setup();
    assert_eq!(
        client.try_approve_hash_algorithm(&0),
        Err(Ok(ProtocolConfigError::InvalidHashAlgorithmId))
    );
    client.approve_hash_algorithm(&7);
    // Replaying the approval is rejected and changes nothing.
    assert_eq!(
        client.try_approve_hash_algorithm(&7),
        Err(Ok(ProtocolConfigError::InvalidHashAlgorithmTransition))
    );
    assert_eq!(client.get_hash_algorithm(&7).unwrap().revision, 1);

    client.activate_hash_algorithm(&7);
    assert_eq!(
        client.try_activate_hash_algorithm(&7),
        Err(Ok(ProtocolConfigError::InvalidHashAlgorithmTransition))
    );
    client.deprecate_hash_algorithm(&7);
    assert_eq!(
        client.try_deprecate_hash_algorithm(&7),
        Err(Ok(ProtocolConfigError::InvalidHashAlgorithmTransition))
    );
    // Deprecation is terminal: no reactivation, no re-approval.
    assert_eq!(
        client.try_activate_hash_algorithm(&7),
        Err(Ok(ProtocolConfigError::InvalidHashAlgorithmTransition))
    );
    assert_eq!(
        client.try_approve_hash_algorithm(&7),
        Err(Ok(ProtocolConfigError::InvalidHashAlgorithmTransition))
    );
}

#[test]
fn an_approved_algorithm_can_be_withdrawn_before_it_ever_activates() {
    let (_env, client) = setup();
    client.approve_hash_algorithm(&9);
    client.deprecate_hash_algorithm(&9);
    assert_eq!(
        client.try_activate_hash_algorithm(&9),
        Err(Ok(ProtocolConfigError::InvalidHashAlgorithmTransition))
    );
    assert!(!client.can_create_commitment(&9));
    assert!(client.can_verify_commitment(&9));
}

#[test]
fn algorithms_are_independent_of_each_other() {
    let (_env, client) = setup();
    client.approve_hash_algorithm(&1);
    client.approve_hash_algorithm(&2);
    client.activate_hash_algorithm(&1);
    client.activate_hash_algorithm(&2);
    client.deprecate_hash_algorithm(&1);
    assert!(!client.can_create_commitment(&1));
    assert!(client.can_create_commitment(&2));
}

#[test]
fn each_lifecycle_change_advances_the_config_version() {
    let (_env, client) = setup();
    let start = client.get_config_version();
    client.approve_hash_algorithm(&7);
    client.activate_hash_algorithm(&7);
    client.deprecate_hash_algorithm(&7);
    assert_eq!(client.get_config_version(), start + 3);
    // A rejected change does not advance it.
    let _ = client.try_activate_hash_algorithm(&7);
    assert_eq!(client.get_config_version(), start + 3);
}
