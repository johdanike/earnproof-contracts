//! Auditable acknowledgement that a delayed proof became eligible.

mod common;

use common::{hash, setup};
use earnproof_shared::{ActivationActorRole, ActivationBlocker, ProofError, ProofValidity};
use soroban_sdk::{testutils::Address as _, testutils::Ledger as _, Address};

const ACTIVATES_AT: u64 = 100;
const EXPIRES_AT: u64 = 200;

#[test]
fn acknowledgement_is_rejected_one_second_early_and_accepted_at_the_boundary() {
    let f = setup();
    let proof_id = f.register_delayed(1, EXPIRES_AT, ACTIVATES_AT);

    f.env.ledger().set_timestamp(ACTIVATES_AT - 1);
    assert_eq!(
        f.client.get_activation_blocker(&proof_id),
        Some(ActivationBlocker::NotYetActive)
    );
    assert_eq!(
        f.client.try_acknowledge_activation(&proof_id, &f.issuer),
        Err(Ok(ProofError::ActivationNotEligible))
    );
    assert_eq!(
        f.client.try_get_activation_acknowledgement(&proof_id),
        Err(Ok(ProofError::ProofNotFound))
    );

    f.env.ledger().set_timestamp(ACTIVATES_AT);
    f.env.ledger().set_sequence_number(77);
    assert_eq!(f.client.get_activation_blocker(&proof_id), None);
    f.client.acknowledge_activation(&proof_id, &f.issuer);

    let record = f.client.get_activation_acknowledgement(&proof_id);
    assert_eq!(record.proof_id_hash, proof_id);
    assert_eq!(record.acknowledged_by, f.issuer);
    assert_eq!(record.actor_role, ActivationActorRole::Issuer);
    assert_eq!(record.ledger, 77);
    assert_eq!(record.timestamp, ACTIVATES_AT);
    assert_eq!(record.sequence, 1);
    assert_eq!(record.activates_at, ACTIVATES_AT);
}

#[test]
fn the_last_valid_second_is_still_eligible_and_the_next_is_expired() {
    let f = setup();
    let proof_id = f.register_delayed(1, EXPIRES_AT, ACTIVATES_AT);

    f.env.ledger().set_timestamp(EXPIRES_AT + 1);
    assert_eq!(
        f.client.get_activation_blocker(&proof_id),
        Some(ActivationBlocker::Expired)
    );
    assert_eq!(
        f.client.try_acknowledge_activation(&proof_id, &f.issuer),
        Err(Ok(ProofError::ActivationNotEligible))
    );

    f.env.ledger().set_timestamp(EXPIRES_AT);
    f.client.acknowledge_activation(&proof_id, &f.issuer);
}

#[test]
fn a_repeated_acknowledgement_is_rejected_and_leaves_the_record_unchanged() {
    let f = setup();
    let proof_id = f.register_delayed(1, EXPIRES_AT, ACTIVATES_AT);
    f.env.ledger().set_timestamp(ACTIVATES_AT);
    f.client.acknowledge_activation(&proof_id, &f.issuer);
    let first = f.client.get_activation_acknowledgement(&proof_id);

    // Same ledger, then a later one: both are replays.
    assert_eq!(
        f.client.try_acknowledge_activation(&proof_id, &f.issuer),
        Err(Ok(ProofError::ActivationAlreadyAcknowledged))
    );
    f.env.ledger().set_timestamp(ACTIVATES_AT + 5);
    f.env.ledger().set_sequence_number(500);
    assert_eq!(
        f.client.try_acknowledge_activation(&proof_id, &f.issuer),
        Err(Ok(ProofError::ActivationAlreadyAcknowledged))
    );
    assert_eq!(f.client.get_activation_acknowledgement(&proof_id), first);
}

#[test]
fn sequence_numbers_are_registry_wide_and_gapless() {
    let f = setup();
    let a = f.register_delayed(1, EXPIRES_AT, ACTIVATES_AT);
    let b = f.register_delayed(2, EXPIRES_AT, ACTIVATES_AT);
    f.env.ledger().set_timestamp(ACTIVATES_AT);

    f.client.acknowledge_activation(&b, &f.issuer);
    // A rejected attempt must not consume a sequence number.
    assert_eq!(
        f.client.try_acknowledge_activation(&b, &f.issuer),
        Err(Ok(ProofError::ActivationAlreadyAcknowledged))
    );
    f.client.acknowledge_activation(&a, &f.issuer);

    assert_eq!(f.client.get_activation_acknowledgement(&b).sequence, 1);
    assert_eq!(f.client.get_activation_acknowledgement(&a).sequence, 2);
}

#[test]
fn an_authorized_delegate_may_acknowledge_and_an_outsider_may_not() {
    let f = setup();
    let proof_id = f.register_delayed(1, EXPIRES_AT, ACTIVATES_AT);
    f.env.ledger().set_timestamp(ACTIVATES_AT);
    let delegate = Address::generate(&f.env);
    let outsider = Address::generate(&f.env);

    assert_eq!(
        f.client.try_acknowledge_activation(&proof_id, &outsider),
        Err(Ok(ProofError::UnauthorizedActivator))
    );
    assert_eq!(
        f.client.try_acknowledge_activation(&proof_id, &delegate),
        Err(Ok(ProofError::UnauthorizedActivator))
    );

    f.client
        .set_activation_delegate(&f.issuer, &delegate, &true);
    assert!(f.client.is_activation_delegate(&f.issuer, &delegate));
    f.client.acknowledge_activation(&proof_id, &delegate);
    assert_eq!(
        f.client
            .get_activation_acknowledgement(&proof_id)
            .actor_role,
        ActivationActorRole::Delegate
    );
}

#[test]
fn a_removed_delegate_loses_the_ability_to_acknowledge() {
    let f = setup();
    let proof_id = f.register_delayed(1, EXPIRES_AT, ACTIVATES_AT);
    f.env.ledger().set_timestamp(ACTIVATES_AT);
    let delegate = Address::generate(&f.env);
    f.client
        .set_activation_delegate(&f.issuer, &delegate, &true);
    f.client
        .set_activation_delegate(&f.issuer, &delegate, &false);
    assert!(!f.client.is_activation_delegate(&f.issuer, &delegate));
    assert_eq!(
        f.client.try_acknowledge_activation(&proof_id, &delegate),
        Err(Ok(ProofError::UnauthorizedActivator))
    );
}

#[test]
fn a_revoked_proof_cannot_be_acknowledged() {
    let f = setup();
    let proof_id = f.register_delayed(1, EXPIRES_AT, ACTIVATES_AT);
    f.env.ledger().set_timestamp(ACTIVATES_AT);
    f.client.revoke_proof(&proof_id);
    assert_eq!(
        f.client.get_activation_blocker(&proof_id),
        Some(ActivationBlocker::Revoked)
    );
    assert_eq!(
        f.client.try_acknowledge_activation(&proof_id, &f.issuer),
        Err(Ok(ProofError::ActivationNotEligible))
    );
}

#[test]
fn a_proof_registered_without_a_delay_cannot_be_acknowledged() {
    let f = setup();
    let proof_id = f.register(1);
    assert_eq!(
        f.client.get_activation_blocker(&proof_id),
        Some(ActivationBlocker::NotDelayed)
    );
    assert_eq!(
        f.client.try_acknowledge_activation(&proof_id, &f.issuer),
        Err(Ok(ProofError::ActivationNotEligible))
    );
}

#[test]
fn an_open_or_upheld_dispute_blocks_but_a_withdrawn_one_does_not() {
    let f = setup();
    let proof_id = f.register_delayed(1, EXPIRES_AT, ACTIVATES_AT);
    f.env.ledger().set_timestamp(ACTIVATES_AT);
    let disputant = Address::generate(&f.env);

    f.client
        .open_dispute(&proof_id, &disputant, &hash(&f.env, 0x40));
    assert_eq!(
        f.client.get_activation_blocker(&proof_id),
        Some(ActivationBlocker::Disputed)
    );
    assert_eq!(
        f.client.try_acknowledge_activation(&proof_id, &f.issuer),
        Err(Ok(ProofError::ActivationNotEligible))
    );

    f.client.withdraw_dispute(&proof_id);
    assert_eq!(f.client.get_activation_blocker(&proof_id), None);
    f.client.acknowledge_activation(&proof_id, &f.issuer);
}

#[test]
fn a_resolved_dispute_keeps_blocking_activation() {
    let f = setup();
    let proof_id = f.register_delayed(1, EXPIRES_AT, ACTIVATES_AT);
    f.env.ledger().set_timestamp(ACTIVATES_AT);
    let disputant = Address::generate(&f.env);
    f.client
        .open_dispute(&proof_id, &disputant, &hash(&f.env, 0x40));
    f.client.resolve_dispute(&proof_id);
    assert_eq!(
        f.client.try_acknowledge_activation(&proof_id, &f.issuer),
        Err(Ok(ProofError::ActivationNotEligible))
    );
}

#[test]
fn an_unknown_proof_reports_not_found() {
    let f = setup();
    let missing = hash(&f.env, 0x66);
    assert_eq!(
        f.client.try_acknowledge_activation(&missing, &f.issuer),
        Err(Ok(ProofError::ProofNotFound))
    );
    assert_eq!(
        f.client.try_get_activation_blocker(&missing),
        Err(Ok(ProofError::ProofNotFound))
    );
}

#[test]
fn acknowledging_does_not_change_proof_validity() {
    let f = setup();
    let proof_id = f.register_delayed(1, EXPIRES_AT, ACTIVATES_AT);
    f.env.ledger().set_timestamp(ACTIVATES_AT);
    let before = f.client.get_proof_validity(&proof_id);
    assert_eq!(before, ProofValidity::Active);
    f.client.acknowledge_activation(&proof_id, &f.issuer);
    assert_eq!(f.client.get_proof_validity(&proof_id), before);
}

#[test]
fn a_paused_registry_rejects_acknowledgements() {
    let f = setup();
    let proof_id = f.register_delayed(1, EXPIRES_AT, ACTIVATES_AT);
    f.env.ledger().set_timestamp(ACTIVATES_AT);
    f.config.pause();
    assert_eq!(
        f.client.try_acknowledge_activation(&proof_id, &f.issuer),
        Err(Ok(ProofError::ContractPaused))
    );
    f.config.unpause();
    f.client.acknowledge_activation(&proof_id, &f.issuer);
}
