//! Proof policy version summaries (issue #300).

mod support;

use earnproof_shared::{
    ProofError, SubjectNullifier, CONFIG_DIGEST_VERSION, ISSUER_POLICY_ENCODING_VERSION,
    PROOF_POLICY_VERSIONS_VERSION, SHA256_COMMITMENT_ALGORITHM_V1,
};
use soroban_sdk::testutils::Ledger as _;
use support::{deploy, hash, register};

#[test]
fn typed_registration_binds_the_versions_in_force() {
    let d = deploy();
    d.env.ledger().set_sequence_number(42);
    let proof = register(&d, 10);
    let versions = d.proofs.get_proof_policy_versions(&proof);
    assert_eq!(versions.record_version, PROOF_POLICY_VERSIONS_VERSION);
    assert_eq!(versions.schema_version, 1);
    assert_eq!(versions.proof_type, Some(hash(&d.env, 1)));
    assert_eq!(
        versions.issuer_policy_version,
        ISSUER_POLICY_ENCODING_VERSION
    );
    assert_eq!(
        versions.commitment_algorithm,
        SHA256_COMMITMENT_ALGORITHM_V1
    );
    assert_eq!(versions.config_digest_version, CONFIG_DIGEST_VERSION);
    assert_eq!(versions.nullifier_domain_version, 0);
    assert_eq!(versions.bound_ledger, 42);
    assert!(!versions.legacy);
}

#[test]
fn summary_is_immutable_across_revocation_and_later_ledgers() {
    let d = deploy();
    let proof = register(&d, 10);
    let before = d.proofs.get_proof_policy_versions(&proof);
    d.proofs.revoke_proof(&proof);
    d.env.ledger().set_sequence_number(10_000);
    assert_eq!(d.proofs.get_proof_policy_versions(&proof), before);
}

#[test]
fn nullifier_registration_records_the_domain_version() {
    let d = deploy();
    let proof = hash(&d.env, 11);
    d.proofs.register_proof_with_nullifier(
        &proof,
        &hash(&d.env, 2),
        &d.issuer,
        &1,
        &1_000_000,
        &None,
        &hash(&d.env, 1),
        &SubjectNullifier {
            domain_version: 1,
            commitment: hash(&d.env, 5),
        },
    );
    let versions = d.proofs.get_proof_policy_versions(&proof);
    assert_eq!(versions.nullifier_domain_version, 1);
    assert!(!versions.legacy);
}

#[test]
fn proofs_without_a_stored_summary_are_reported_as_legacy() {
    let d = deploy();
    let proof = hash(&d.env, 12);
    d.proofs.register_proof_with_activation(
        &proof,
        &hash(&d.env, 2),
        &d.issuer,
        &1,
        &1_000_000,
        &10,
    );
    let versions = d.proofs.get_proof_policy_versions(&proof);
    assert!(versions.legacy);
    assert_eq!(versions.schema_version, 1);
    assert_eq!(versions.proof_type, None);
    assert_eq!(versions.issuer_policy_version, 0);
    assert_eq!(versions.commitment_algorithm, 0);
    assert_eq!(versions.config_digest_version, 0);
    assert_eq!(versions.nullifier_domain_version, 0);
}

#[test]
fn unknown_proof_is_not_found_and_failed_registration_stores_nothing() {
    let d = deploy();
    let unknown = hash(&d.env, 77);
    assert_eq!(
        d.proofs.try_get_proof_policy_versions(&unknown),
        Err(Ok(ProofError::ProofNotFound))
    );
    // Duplicate registration fails and leaves the original summary intact.
    let proof = register(&d, 10);
    let before = d.proofs.get_proof_policy_versions(&proof);
    assert!(d
        .proofs
        .try_register_proof_typed(
            &proof,
            &hash(&d.env, 3),
            &d.issuer,
            &1,
            &1_000_000,
            &hash(&d.env, 1),
        )
        .is_err());
    assert_eq!(d.proofs.get_proof_policy_versions(&proof), before);
}
