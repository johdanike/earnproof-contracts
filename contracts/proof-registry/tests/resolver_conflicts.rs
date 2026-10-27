//! A dispute resolver must not decide a case it is a party to.

mod common;

use common::{hash, setup, Fixture};
use earnproof_shared::{ConflictRole, DisputeStatus, ProofError};
use soroban_sdk::{testutils::Address as _, Address, BytesN};

fn open(f: &Fixture, n: u8, disputant: &Address) -> BytesN<32> {
    let proof_id = f.register(n);
    f.client
        .open_dispute(&proof_id, disputant, &hash(&f.env, 0x40));
    proof_id
}

fn resolver(f: &Fixture, commitment: u8) -> Address {
    let resolver = Address::generate(&f.env);
    f.client
        .register_dispute_resolver(&resolver, &hash(&f.env, commitment));
    resolver
}

#[test]
fn an_unconflicted_registered_resolver_can_resolve_and_reject() {
    let f = setup();
    let disputant = Address::generate(&f.env);
    let resolver = resolver(&f, 0x50);
    let resolved = open(&f, 1, &disputant);
    let rejected = open(&f, 2, &disputant);

    assert_eq!(f.client.get_resolver_conflict(&resolved, &resolver), None);
    f.client.resolve_dispute_as(&resolved, &resolver);
    f.client.reject_dispute_as(&rejected, &resolver);
    assert_eq!(
        f.client.get_dispute(&resolved).status,
        DisputeStatus::Resolved
    );
    assert_eq!(
        f.client.get_dispute(&rejected).status,
        DisputeStatus::Rejected
    );
    assert_eq!(f.client.get_dispute(&resolved).updated_by, resolver);
}

#[test]
fn an_unregistered_address_cannot_act_as_resolver() {
    let f = setup();
    let disputant = Address::generate(&f.env);
    let proof_id = open(&f, 1, &disputant);
    let stranger = Address::generate(&f.env);
    assert_eq!(
        f.client.try_resolve_dispute_as(&proof_id, &stranger),
        Err(Ok(ProofError::ResolverNotRegistered))
    );
    assert_eq!(f.client.get_dispute(&proof_id).status, DisputeStatus::Open);
}

#[test]
fn the_issuer_cannot_resolve_its_own_proof_dispute() {
    let f = setup();
    let disputant = Address::generate(&f.env);
    let proof_id = open(&f, 1, &disputant);
    f.client
        .register_dispute_resolver(&f.issuer, &hash(&f.env, 0x51));

    assert_eq!(
        f.client.get_resolver_conflict(&proof_id, &f.issuer),
        Some(ConflictRole::Issuer)
    );
    assert_eq!(
        f.client.try_resolve_dispute_as(&proof_id, &f.issuer),
        Err(Ok(ProofError::ResolverConflictOfInterest))
    );
    assert_eq!(f.client.get_dispute(&proof_id).status, DisputeStatus::Open);
}

#[test]
fn the_evidence_submitter_cannot_resolve_its_own_dispute() {
    let f = setup();
    let submitter = resolver(&f, 0x52);
    let proof_id = open(&f, 1, &submitter);
    assert_eq!(
        f.client.get_resolver_conflict(&proof_id, &submitter),
        Some(ConflictRole::EvidenceSubmitter)
    );
    assert_eq!(
        f.client.try_reject_dispute_as(&proof_id, &submitter),
        Err(Ok(ProofError::ResolverConflictOfInterest))
    );
}

#[test]
fn a_subject_authorized_actor_cannot_resolve() {
    let f = setup();
    let disputant = Address::generate(&f.env);
    let proof_id = open(&f, 1, &disputant);
    let actor = resolver(&f, 0x53);
    assert_eq!(f.client.get_resolver_conflict(&proof_id, &actor), None);

    f.client.authorize_subject_actor(&proof_id, &actor);
    assert_eq!(
        f.client.get_resolver_conflict(&proof_id, &actor),
        Some(ConflictRole::SubjectAuthorizedActor)
    );
    assert_eq!(
        f.client.try_resolve_dispute_as(&proof_id, &actor),
        Err(Ok(ProofError::ResolverConflictOfInterest))
    );
}

#[test]
fn rotating_the_issuer_address_does_not_clear_the_conflict() {
    let f = setup();
    let disputant = Address::generate(&f.env);
    let proof_id = open(&f, 1, &disputant);
    let new_issuer = Address::generate(&f.env);
    f.issuers.rotate_issuer_address(&f.issuer_id, &new_issuer);
    f.issuers.accept_issuer_address_rotation(&f.issuer_id);

    // The proof still records the old address, but the issuer is the same.
    f.client
        .register_dispute_resolver(&new_issuer, &hash(&f.env, 0x54));
    assert_eq!(
        f.client.get_resolver_conflict(&proof_id, &new_issuer),
        Some(ConflictRole::Issuer)
    );
    assert_eq!(
        f.client.try_resolve_dispute_as(&proof_id, &new_issuer),
        Err(Ok(ProofError::ResolverConflictOfInterest))
    );
    // The former address is still the issuer by address.
    f.client
        .register_dispute_resolver(&f.issuer, &hash(&f.env, 0x55));
    assert_eq!(
        f.client.get_resolver_conflict(&proof_id, &f.issuer),
        Some(ConflictRole::Issuer)
    );
}

#[test]
fn rotating_the_resolver_address_keeps_the_submitter_conflict() {
    let f = setup();
    let submitter = Address::generate(&f.env);
    let shared = hash(&f.env, 0x60);
    f.client.bind_participant_commitment(&submitter, &shared);
    let proof_id = open(&f, 1, &submitter);

    // A new address carrying the same commitment is the same participant.
    let rotated = Address::generate(&f.env);
    f.client.register_dispute_resolver(&rotated, &shared);
    assert_eq!(
        f.client.get_resolver_conflict(&proof_id, &rotated),
        Some(ConflictRole::EvidenceSubmitter)
    );
    assert_eq!(
        f.client.try_resolve_dispute_as(&proof_id, &rotated),
        Err(Ok(ProofError::ResolverConflictOfInterest))
    );
}

#[test]
fn a_delegate_sharing_a_commitment_with_the_resolver_is_a_conflict() {
    let f = setup();
    let disputant = Address::generate(&f.env);
    let proof_id = open(&f, 1, &disputant);
    let shared = hash(&f.env, 0x61);
    let delegate = Address::generate(&f.env);
    f.client.bind_participant_commitment(&delegate, &shared);
    f.client.authorize_subject_actor(&proof_id, &delegate);

    let resolver = Address::generate(&f.env);
    f.client.register_dispute_resolver(&resolver, &shared);
    assert_eq!(
        f.client.get_resolver_conflict(&proof_id, &resolver),
        Some(ConflictRole::SubjectAuthorizedActor)
    );
}

#[test]
fn the_admin_path_applies_the_same_check() {
    let f = setup();
    let proof_id = open(&f, 1, &f.admin);
    assert_eq!(
        f.client.try_resolve_dispute(&proof_id),
        Err(Ok(ProofError::ResolverConflictOfInterest))
    );
    assert_eq!(
        f.client.try_reject_dispute(&proof_id),
        Err(Ok(ProofError::ResolverConflictOfInterest))
    );
    assert_eq!(f.client.get_dispute(&proof_id).status, DisputeStatus::Open);

    // A dispute opened by someone else is still decided by the admin.
    let other = open(&f, 2, &Address::generate(&f.env));
    f.client.resolve_dispute(&other);
    assert_eq!(f.client.get_dispute(&other).status, DisputeStatus::Resolved);
}

#[test]
fn removing_a_resolver_revokes_its_ability_to_decide() {
    let f = setup();
    let disputant = Address::generate(&f.env);
    let proof_id = open(&f, 1, &disputant);
    let resolver = resolver(&f, 0x56);
    f.client.remove_dispute_resolver(&resolver);
    assert_eq!(f.client.get_dispute_resolver_commitment(&resolver), None);
    assert_eq!(
        f.client.try_resolve_dispute_as(&proof_id, &resolver),
        Err(Ok(ProofError::ResolverNotRegistered))
    );
}

#[test]
fn a_decided_dispute_cannot_be_decided_again() {
    let f = setup();
    let disputant = Address::generate(&f.env);
    let proof_id = open(&f, 1, &disputant);
    let resolver = resolver(&f, 0x57);
    f.client.resolve_dispute_as(&proof_id, &resolver);
    assert_eq!(
        f.client.try_resolve_dispute_as(&proof_id, &resolver),
        Err(Ok(ProofError::DisputeNotOpen))
    );
    assert_eq!(
        f.client.try_reject_dispute_as(&proof_id, &resolver),
        Err(Ok(ProofError::DisputeNotOpen))
    );
}

#[test]
fn participant_commitments_are_write_once_and_never_zero() {
    let f = setup();
    let participant = Address::generate(&f.env);
    let first = hash(&f.env, 0x70);
    f.client.bind_participant_commitment(&participant, &first);
    // Binding the same value again is idempotent.
    f.client.bind_participant_commitment(&participant, &first);
    assert_eq!(
        f.client
            .try_bind_participant_commitment(&participant, &hash(&f.env, 0x71)),
        Err(Ok(ProofError::ParticipantCommitmentLocked))
    );
    assert_eq!(
        f.client
            .try_bind_participant_commitment(&Address::generate(&f.env), &hash(&f.env, 0)),
        Err(Ok(ProofError::MalformedInput))
    );
    assert_eq!(
        f.client
            .try_register_dispute_resolver(&Address::generate(&f.env), &hash(&f.env, 0)),
        Err(Ok(ProofError::MalformedInput))
    );
}

#[test]
fn the_conflict_view_requires_a_proof_and_a_dispute() {
    let f = setup();
    let someone = Address::generate(&f.env);
    assert_eq!(
        f.client
            .try_get_resolver_conflict(&hash(&f.env, 0x99), &someone),
        Err(Ok(ProofError::ProofNotFound))
    );
    let proof_id = f.register(1);
    assert_eq!(
        f.client.try_get_resolver_conflict(&proof_id, &someone),
        Err(Ok(ProofError::DisputeNotFound))
    );
}
