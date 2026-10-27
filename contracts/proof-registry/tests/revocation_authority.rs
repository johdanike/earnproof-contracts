//! Revocations are bound to the authority policy version that authorized them.

mod common;

use common::{hash, setup};
use earnproof_shared::{ProofError, RevocationActorRole};
use soroban_sdk::{testutils::Ledger as _, vec};

#[test]
fn the_default_policy_version_is_one() {
    let f = setup();
    assert_eq!(f.client.get_authority_policy_version(), 1);
}

#[test]
fn issuer_revocation_commits_version_role_actor_and_domain() {
    let f = setup();
    f.env.ledger().set_timestamp(500);
    f.env.ledger().set_sequence_number(42);
    let proof_id = f.register(1);
    f.client.revoke_proof(&proof_id);

    let evidence = f.client.get_revocation_authority(&proof_id);
    assert_eq!(evidence.proof_id_hash, proof_id);
    assert_eq!(evidence.policy_version, 1);
    assert_eq!(evidence.actor_role, RevocationActorRole::Issuer);
    assert_eq!(evidence.actor, f.issuer);
    assert_eq!(evidence.revoked_at, 500);
    assert_eq!(evidence.revoked_ledger, 42);
    let record = f.client.get_proof(&proof_id);
    assert_eq!(evidence.revoked_at, record.revoked_at);
    assert_eq!(evidence.revoked_ledger, record.revoked_ledger);
}

#[test]
fn admin_revocation_commits_the_admin_role_and_a_distinct_domain() {
    let f = setup();
    let by_issuer = f.register(1);
    let by_admin = f.register(2);
    f.client.revoke_proof(&by_issuer);
    f.client.admin_revoke_proof(&by_admin);

    let issuer_evidence = f.client.get_revocation_authority(&by_issuer);
    let admin_evidence = f.client.get_revocation_authority(&by_admin);
    assert_eq!(admin_evidence.actor_role, RevocationActorRole::Admin);
    assert_eq!(admin_evidence.actor, f.admin);
    assert_ne!(
        issuer_evidence.authorization_domain,
        admin_evidence.authorization_domain
    );
}

#[test]
fn a_later_policy_change_cannot_alter_recorded_evidence() {
    let f = setup();
    let first = f.register(1);
    f.client.revoke_proof(&first);
    let before = f.client.get_revocation_authority(&first);

    f.client.set_authority_policy_version(&2);
    assert_eq!(f.client.get_authority_policy_version(), 2);
    assert_eq!(f.client.get_revocation_authority(&first), before);

    let second = f.register(2);
    f.client.revoke_proof(&second);
    let after = f.client.get_revocation_authority(&second);
    assert_eq!(after.policy_version, 2);
    assert_ne!(after.authorization_domain, before.authorization_domain);
    // The earlier record is still the one written at its own revocation.
    assert_eq!(f.client.get_revocation_authority(&first), before);
}

#[test]
fn a_pinned_revocation_fails_when_the_policy_moved_on() {
    let f = setup();
    let proof_id = f.register(1);
    f.client.set_authority_policy_version(&2);

    assert_eq!(
        f.client.try_revoke_proof_at_policy(&proof_id, &1),
        Err(Ok(ProofError::StalePolicyVersion))
    );
    assert_eq!(
        f.client.try_admin_revoke_proof_at_policy(&proof_id, &1),
        Err(Ok(ProofError::StalePolicyVersion))
    );
    // A version ahead of the one in force fails the same way.
    assert_eq!(
        f.client.try_revoke_proof_at_policy(&proof_id, &3),
        Err(Ok(ProofError::StalePolicyVersion))
    );
    // Rejected attempts change nothing.
    assert!(!f.client.is_revoked(&proof_id));
    assert_eq!(
        f.client.try_get_revocation_authority(&proof_id),
        Err(Ok(ProofError::ProofNotFound))
    );

    f.client.revoke_proof_at_policy(&proof_id, &2);
    assert_eq!(
        f.client.get_revocation_authority(&proof_id).policy_version,
        2
    );
}

#[test]
fn already_revoked_is_reported_before_a_stale_version() {
    let f = setup();
    let proof_id = f.register(1);
    f.client.revoke_proof(&proof_id);
    let evidence = f.client.get_revocation_authority(&proof_id);
    f.client.set_authority_policy_version(&2);

    assert_eq!(
        f.client.try_revoke_proof_at_policy(&proof_id, &1),
        Err(Ok(ProofError::ProofAlreadyRevoked))
    );
    assert_eq!(
        f.client.try_revoke_proof(&proof_id),
        Err(Ok(ProofError::ProofAlreadyRevoked))
    );
    assert_eq!(f.client.get_revocation_authority(&proof_id), evidence);
}

#[test]
fn the_policy_version_only_increases() {
    let f = setup();
    assert_eq!(
        f.client.try_set_authority_policy_version(&1),
        Err(Ok(ProofError::InvalidPolicyVersion))
    );
    assert_eq!(
        f.client.try_set_authority_policy_version(&0),
        Err(Ok(ProofError::InvalidPolicyVersion))
    );
    f.client.set_authority_policy_version(&2);
    // Replaying the same governance call is rejected.
    assert_eq!(
        f.client.try_set_authority_policy_version(&2),
        Err(Ok(ProofError::InvalidPolicyVersion))
    );
    f.client.set_authority_policy_version(&10);
    assert_eq!(f.client.get_authority_policy_version(), 10);
}

#[test]
fn an_unrevoked_proof_has_no_authority_record() {
    let f = setup();
    let proof_id = f.register(1);
    assert_eq!(
        f.client.try_get_revocation_authority(&proof_id),
        Err(Ok(ProofError::ProofNotFound))
    );
    assert_eq!(
        f.client.try_get_revocation_authority(&hash(&f.env, 0x77)),
        Err(Ok(ProofError::ProofNotFound))
    );
}

#[test]
fn batch_revocation_binds_every_entry_to_the_same_version() {
    let f = setup();
    let a = f.register(1);
    let b = f.register(2);
    f.client.set_authority_policy_version(&3);
    f.client
        .revoke_proofs_batch(&vec![&f.env, a.clone(), b.clone()]);

    for id in [a, b] {
        let evidence = f.client.get_revocation_authority(&id);
        assert_eq!(evidence.policy_version, 3);
        assert_eq!(evidence.actor_role, RevocationActorRole::Issuer);
    }

    let c = f.register(3);
    f.client.admin_revoke_proofs_batch(&vec![&f.env, c.clone()]);
    assert_eq!(
        f.client.get_revocation_authority(&c).actor_role,
        RevocationActorRole::Admin
    );
}
