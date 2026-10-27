//! Disclosure consent withdrawal records (issue #301) and the consent
//! withdrawal event sequence range (issue #299).

mod support;

use earnproof_shared::{
    consent_withdrawal_commitment, EventDomain, ProofError, CONSENT_WITHDRAWAL_VERSION,
    EVENT_CHECKPOINT_INTERVAL, MAX_CONSENT_WITHDRAWAL_PAGE,
};
use soroban_sdk::testutils::{Events as _, Ledger as _, MockAuth, MockAuthInvoke};
use soroban_sdk::{Address, IntoVal};
use support::{deploy, hash, register};

#[test]
fn withdrawal_is_committed_queryable_and_blocks_new_receipts() {
    let d = deploy();
    let proof = register(&d, 10);
    let policy = hash(&d.env, 20);
    d.proofs
        .commit_disclosure_consent(&proof, &policy, &1, &hash(&d.env, 30));

    d.env.ledger().set_sequence_number(77);
    let record = d
        .proofs
        .withdraw_disclosure_consent(&proof, &policy, &1, &1, &hash(&d.env, 31));
    assert_eq!(record.record_version, CONSENT_WITHDRAWAL_VERSION);
    assert_eq!(record.sequence, 1);
    assert_eq!(record.reason_code, 1);
    assert_eq!(record.effective_ledger, 77);
    let expected = d.env.as_contract(&d.proofs.address, || {
        consent_withdrawal_commitment(
            &d.env,
            &d.env.ledger().network_id(),
            &d.proofs.address,
            &proof,
            &policy,
            1,
            1,
            &hash(&d.env, 31),
        )
    });
    assert_eq!(record.commitment_hash, expected);

    assert!(d.proofs.is_consent_withdrawn(&proof, &policy));
    assert_eq!(d.proofs.get_consent_withdrawal_count(), 1);
    assert_eq!(d.proofs.get_consent_withdrawal(&1), Some(record.clone()));
    // Other policies on the same proof are unaffected.
    assert!(!d.proofs.is_consent_withdrawn(&proof, &hash(&d.env, 21)));
    d.proofs
        .commit_disclosure_consent(&proof, &hash(&d.env, 21), &1, &hash(&d.env, 30));

    // A new receipt for the withdrawn pair is refused.
    assert_eq!(
        d.proofs
            .try_commit_disclosure_consent(&proof, &policy, &2, &hash(&d.env, 32)),
        Err(Ok(ProofError::ConsentWithdrawn))
    );
    // The earlier receipt stays on record: history is not rewritten.
    assert!(d
        .proofs
        .has_consent_receipt_commitment(&hash_receipt(&d, &proof, &policy, 30)));
}

fn hash_receipt(
    d: &support::Deployment,
    proof: &soroban_sdk::BytesN<32>,
    policy: &soroban_sdk::BytesN<32>,
    receipt: u8,
) -> soroban_sdk::BytesN<32> {
    d.env.as_contract(&d.proofs.address, || {
        earnproof_shared::disclosure_consent_commitment(
            &d.env,
            &d.env.ledger().network_id(),
            &d.proofs.address,
            proof,
            policy,
            1,
            &hash(&d.env, receipt),
        )
    })
}

#[test]
fn duplicate_stale_and_invalid_requests_are_rejected_without_state_change() {
    let d = deploy();
    let proof = register(&d, 10);
    let policy = hash(&d.env, 20);
    let salt = hash(&d.env, 31);

    // Reason codes 0 and 5 are outside the published range.
    for bad in [0_u32, 5, u32::MAX] {
        assert_eq!(
            d.proofs
                .try_withdraw_disclosure_consent(&proof, &policy, &1, &bad, &salt),
            Err(Ok(ProofError::InvalidConsentWithdrawalReason))
        );
    }
    // Wrong expected sequence: too early and too late.
    for stale in [0_u64, 2, 99] {
        assert_eq!(
            d.proofs
                .try_withdraw_disclosure_consent(&proof, &policy, &stale, &1, &salt),
            Err(Ok(ProofError::StaleConsentWithdrawalSequence))
        );
    }
    assert_eq!(
        d.proofs
            .try_withdraw_disclosure_consent(&hash(&d.env, 99), &policy, &1, &1, &salt),
        Err(Ok(ProofError::ProofNotFound))
    );
    assert_eq!(d.proofs.get_consent_withdrawal_count(), 0);
    assert!(!d.proofs.is_consent_withdrawn(&proof, &policy));

    d.proofs
        .withdraw_disclosure_consent(&proof, &policy, &1, &2, &salt);
    // Replaying the identical request is stale; a fresh sequence for the same
    // pair is reported as already withdrawn.
    assert_eq!(
        d.proofs
            .try_withdraw_disclosure_consent(&proof, &policy, &1, &2, &salt),
        Err(Ok(ProofError::StaleConsentWithdrawalSequence))
    );
    assert_eq!(
        d.proofs
            .try_withdraw_disclosure_consent(&proof, &policy, &2, &2, &salt),
        Err(Ok(ProofError::ConsentAlreadyWithdrawn))
    );
    assert_eq!(d.proofs.get_consent_withdrawal_count(), 1);
}

#[test]
fn only_the_proof_issuer_can_withdraw() {
    let d = deploy();
    let proof = register(&d, 10);
    let policy = hash(&d.env, 20);
    let outsider = <Address as soroban_sdk::testutils::Address>::generate(&d.env);
    d.env.mock_auths(&[MockAuth {
        address: &outsider,
        invoke: &MockAuthInvoke {
            contract: &d.proofs.address,
            fn_name: "withdraw_disclosure_consent",
            args: (
                proof.clone(),
                policy.clone(),
                1_u64,
                1_u32,
                hash(&d.env, 31),
            )
                .into_val(&d.env),
            sub_invokes: &[],
        },
    }]);
    assert!(d
        .proofs
        .try_withdraw_disclosure_consent(&proof, &policy, &1, &1, &hash(&d.env, 31))
        .is_err());
    assert_eq!(d.proofs.get_consent_withdrawal_count(), 0);
}

#[test]
fn withdrawal_is_allowed_for_revoked_proofs_and_emits_one_event() {
    let d = deploy();
    let proof = register(&d, 10);
    d.proofs.revoke_proof(&proof);
    let policy = hash(&d.env, 20);
    d.proofs
        .withdraw_disclosure_consent(&proof, &policy, &1, &3, &hash(&d.env, 31));
    assert_eq!(d.env.events().all().events().len(), 1);
    // A failed request emits nothing.
    let _ = d
        .proofs
        .try_withdraw_disclosure_consent(&proof, &policy, &2, &3, &hash(&d.env, 31));
    assert_eq!(d.env.events().all().events().len(), 0);
}

#[test]
fn pages_are_bounded_ordered_and_start_from_any_sequence() {
    let d = deploy();
    let proof = register(&d, 10);
    for index in 0..25_u8 {
        d.proofs.withdraw_disclosure_consent(
            &proof,
            &hash(&d.env, 100 + index),
            &(u64::from(index) + 1),
            &1,
            &hash(&d.env, 31),
        );
    }
    assert_eq!(d.proofs.get_consent_withdrawal_count(), 25);
    let first = d
        .proofs
        .get_consent_withdrawals(&0, &MAX_CONSENT_WITHDRAWAL_PAGE);
    assert_eq!(first.len(), MAX_CONSENT_WITHDRAWAL_PAGE);
    assert_eq!(first.get(0).unwrap().sequence, 1);
    let rest = d
        .proofs
        .get_consent_withdrawals(&21, &MAX_CONSENT_WITHDRAWAL_PAGE);
    assert_eq!(rest.len(), 5);
    assert_eq!(rest.get(4).unwrap().sequence, 25);
    assert!(d.proofs.get_consent_withdrawals(&26, &5).is_empty());
    for bad in [0_u32, MAX_CONSENT_WITHDRAWAL_PAGE + 1] {
        assert_eq!(
            d.proofs.try_get_consent_withdrawals(&1, &bad),
            Err(Ok(ProofError::InvalidBatchSize))
        );
    }
}

#[test]
fn event_range_tracks_withdrawals_and_rejects_foreign_domains() {
    let d = deploy();
    let proof = register(&d, 10);
    let empty = d
        .proofs
        .get_event_sequence_range(&EventDomain::ConsentWithdrawal);
    assert_eq!(
        (empty.first_retained, empty.latest_emitted, empty.checkpoint),
        (0, 0, 0)
    );

    for index in 0..17_u8 {
        d.proofs.withdraw_disclosure_consent(
            &proof,
            &hash(&d.env, 100 + index),
            &(u64::from(index) + 1),
            &1,
            &hash(&d.env, 31),
        );
    }
    let range = d
        .proofs
        .get_event_sequence_range(&EventDomain::ConsentWithdrawal);
    assert_eq!(range.first_retained, 1);
    assert_eq!(range.latest_emitted, 17);
    assert_eq!(range.checkpoint, EVENT_CHECKPOINT_INTERVAL);

    // A rejected withdrawal does not move the range.
    let _ = d.proofs.try_withdraw_disclosure_consent(
        &proof,
        &hash(&d.env, 100),
        &18,
        &1,
        &hash(&d.env, 31),
    );
    assert_eq!(
        d.proofs
            .get_event_sequence_range(&EventDomain::ConsentWithdrawal),
        range
    );
    assert_eq!(
        d.proofs
            .try_get_event_sequence_range(&EventDomain::ConfigChange),
        Err(Ok(ProofError::UnsupportedEventDomain))
    );
}
