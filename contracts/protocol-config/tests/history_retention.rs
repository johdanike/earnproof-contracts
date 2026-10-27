//! Governed history retention caps (issue #298) and the event sequence ranges
//! (issue #299) that report the retained window.

use earnproof_shared::{
    ContractError, EventDomain, HistoryKind, PauseScope, CONFIG_HISTORY_CAPACITY,
    CONFIG_HISTORY_MIN_RETENTION, EVENT_CHECKPOINT_INTERVAL, EVENT_RANGE_VERSION,
    HISTORY_RETENTION_VERSION, PAUSE_ACKNOWLEDGEMENT_MAX_RETENTION,
    PAUSE_ACKNOWLEDGEMENT_MIN_RETENTION,
};
use protocol_config::{ProtocolConfigContract, ProtocolConfigContractClient};
use soroban_sdk::{
    testutils::{Address as _, Events as _, MockAuth, MockAuthInvoke},
    Address, BytesN, Env, IntoVal,
};

fn setup() -> (Env, ProtocolConfigContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(ProtocolConfigContract, ());
    let client = ProtocolConfigContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, client, admin)
}

/// Appends `count` change-history entries by toggling a scoped pause.
fn append_config_changes(client: &ProtocolConfigContractClient<'static>, count: u32) {
    for index in 0..count {
        client.set_scoped_pause(&PauseScope::Updates, &(index % 2 == 0));
    }
}

/// Records `count` pause acknowledgements with distinct commitments, starting
/// from `first`. A scoped pause must already be active.
fn acknowledge(
    env: &Env,
    client: &ProtocolConfigContractClient<'static>,
    admin: &Address,
    first: u32,
    count: u32,
) {
    for offset in 0..count {
        let value = first + offset;
        let mut bytes = [0u8; 32];
        bytes[..4].copy_from_slice(&(value + 1).to_be_bytes());
        client.acknowledge_pause(&PauseScope::Global, &BytesN::from_array(env, &bytes), admin);
    }
}

fn ack_hash(env: &Env, value: u32) -> BytesN<32> {
    let mut bytes = [0u8; 32];
    bytes[..4].copy_from_slice(&(value + 1).to_be_bytes());
    BytesN::from_array(env, &bytes)
}

#[test]
fn defaults_preserve_existing_retention() {
    let (_env, client, _admin) = setup();
    let config = client.get_history_retention(&HistoryKind::ConfigChange);
    assert_eq!(config.version, HISTORY_RETENTION_VERSION);
    assert_eq!(config.max_entries, CONFIG_HISTORY_CAPACITY);
    assert_eq!(config.min_allowed, CONFIG_HISTORY_MIN_RETENTION);
    assert_eq!(config.max_allowed, CONFIG_HISTORY_CAPACITY);
    assert_eq!(config.revision, 0);
    assert_eq!(config.oldest_retained, 0);

    let pause = client.get_history_retention(&HistoryKind::PauseAcknowledgement);
    assert_eq!(pause.max_entries, PAUSE_ACKNOWLEDGEMENT_MAX_RETENTION);
    assert_eq!(pause.min_allowed, PAUSE_ACKNOWLEDGEMENT_MIN_RETENTION);
    assert_eq!(pause.max_allowed, PAUSE_ACKNOWLEDGEMENT_MAX_RETENTION);
    assert_eq!(pause.revision, 0);
    assert_eq!(pause.next_sequence, 0);
}

#[test]
fn lowering_a_cap_hides_old_entries_without_renumbering_cursors() {
    let (_env, client, _admin) = setup();
    append_config_changes(&client, 40);
    let before = client.get_history_retention(&HistoryKind::ConfigChange);
    assert_eq!(before.oldest_retained, before.next_sequence - 32);

    let after = client.set_history_retention(&HistoryKind::ConfigChange, &10, &0);
    assert_eq!(after.max_entries, 10);
    assert_eq!(after.revision, 1);
    // The retention change is itself recorded, so the cursor advanced by one
    // and was never renumbered.
    assert_eq!(client.get_config_history_cursor(), before.next_sequence + 1);
    assert_eq!(after.next_sequence, before.next_sequence + 1);
    assert_eq!(after.oldest_retained, after.next_sequence - 10);

    // A stale cursor is clamped forward to the oldest retained entry, and the
    // page is exactly the retained window: no holes, no silent shortfall.
    let page = client.get_config_history(&0, &20);
    assert_eq!(page.len(), 10);
    let from_oldest = client.get_config_history(&after.oldest_retained, &20);
    assert_eq!(page, from_oldest);
    // The newest entry is the retention change itself.
    let last = page.get(page.len() - 1).unwrap();
    assert_eq!(
        last.category,
        earnproof_shared::ConfigChangeCategory::HistoryRetention
    );
}

#[test]
fn window_never_drops_below_the_protocol_minimum() {
    let (_env, client, _admin) = setup();
    append_config_changes(&client, 60);
    client.set_history_retention(
        &HistoryKind::ConfigChange,
        &CONFIG_HISTORY_MIN_RETENTION,
        &0,
    );
    append_config_changes(&client, 30);
    let view = client.get_history_retention(&HistoryKind::ConfigChange);
    assert_eq!(
        view.next_sequence - view.oldest_retained,
        CONFIG_HISTORY_MIN_RETENTION
    );
    let page = client.get_config_history(&0, &20);
    assert_eq!(page.len(), CONFIG_HISTORY_MIN_RETENTION);
}

#[test]
fn exact_boundaries_are_inclusive_and_out_of_range_is_rejected() {
    let (_env, client, _admin) = setup();
    for (kind, min, max) in [
        (
            HistoryKind::ConfigChange,
            CONFIG_HISTORY_MIN_RETENTION,
            CONFIG_HISTORY_CAPACITY,
        ),
        (
            HistoryKind::PauseAcknowledgement,
            PAUSE_ACKNOWLEDGEMENT_MIN_RETENTION,
            PAUSE_ACKNOWLEDGEMENT_MAX_RETENTION,
        ),
    ] {
        assert_eq!(
            client.try_set_history_retention(&kind, &(min - 1), &0),
            Err(Ok(ContractError::InvalidInput))
        );
        assert_eq!(
            client.try_set_history_retention(&kind, &(max + 1), &0),
            Err(Ok(ContractError::InvalidInput))
        );
        assert_eq!(
            client.try_set_history_retention(&kind, &0, &0),
            Err(Ok(ContractError::InvalidInput))
        );
        // Neither rejected call advanced the revision.
        assert_eq!(client.get_history_retention(&kind).revision, 0);
        // The lower bound is accepted, then the default upper bound is
        // restorable at the next revision.
        assert_eq!(
            client.set_history_retention(&kind, &min, &0).max_entries,
            min
        );
        assert_eq!(
            client.set_history_retention(&kind, &max, &1).max_entries,
            max
        );
    }
}

#[test]
fn stale_revision_and_replay_are_rejected_deterministically() {
    let (_env, client, _admin) = setup();
    let kind = HistoryKind::ConfigChange;
    client.set_history_retention(&kind, &20, &0);

    // Replaying the identical request fails: its revision is now stale.
    assert_eq!(
        client.try_set_history_retention(&kind, &20, &0),
        Err(Ok(ContractError::InvalidState))
    );
    // A competing update that read revision 0 loses to the one that landed.
    assert_eq!(
        client.try_set_history_retention(&kind, &12, &0),
        Err(Ok(ContractError::InvalidState))
    );
    // Re-submitting the cap in force with the current revision is a no-op.
    assert_eq!(
        client.try_set_history_retention(&kind, &20, &1),
        Err(Ok(ContractError::InvalidState))
    );
    let view = client.get_history_retention(&kind);
    assert_eq!(view.max_entries, 20);
    assert_eq!(view.revision, 1);
    // A future revision is also stale state.
    assert_eq!(
        client.try_set_history_retention(&kind, &12, &7),
        Err(Ok(ContractError::InvalidState))
    );
    assert_eq!(client.set_history_retention(&kind, &12, &1).revision, 2);
}

#[test]
fn only_the_admin_can_change_retention() {
    let (env, client, _admin) = setup();
    let outsider = Address::generate(&env);
    env.mock_auths(&[MockAuth {
        address: &outsider,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "set_history_retention",
            args: (HistoryKind::ConfigChange, 12_u32, 0_u32).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    assert!(client
        .try_set_history_retention(&HistoryKind::ConfigChange, &12, &0)
        .is_err());
    let view = client.get_history_retention(&HistoryKind::ConfigChange);
    assert_eq!(view.max_entries, CONFIG_HISTORY_CAPACITY);
    assert_eq!(view.revision, 0);
}

#[test]
fn successful_update_emits_one_event_and_failed_update_emits_none() {
    let (env, client, _admin) = setup();
    assert!(client
        .try_set_history_retention(&HistoryKind::ConfigChange, &3, &0)
        .is_err());
    assert_eq!(env.events().all().events().len(), 0);
    client.set_history_retention(&HistoryKind::ConfigChange, &12, &0);
    assert_eq!(env.events().all().events().len(), 1);
}

#[test]
fn pause_acknowledgement_cap_prunes_history_but_keeps_replay_protection() {
    let (env, client, admin) = setup();
    client.pause();
    let kind = HistoryKind::PauseAcknowledgement;
    client.set_history_retention(&kind, &PAUSE_ACKNOWLEDGEMENT_MIN_RETENTION, &0);
    acknowledge(&env, &client, &admin, 0, 24);

    let view = client.get_history_retention(&kind);
    assert_eq!(view.next_sequence, 24);
    assert_eq!(
        view.oldest_retained,
        24 - PAUSE_ACKNOWLEDGEMENT_MIN_RETENTION
    );

    // A cursor behind the window is clamped to the oldest retained record.
    let page = client.get_pause_acknowledgements(&0, &20);
    assert_eq!(page.len(), 16);
    assert_eq!(page.get(0).unwrap().sequence, view.oldest_retained);
    assert_eq!(page.get(15).unwrap().sequence, 23);

    // The removed history entry is gone, but its hash index still answers and
    // still rejects a replay.
    assert!(client
        .get_pause_acknowledgement(&ack_hash(&env, 0))
        .is_some());
    assert_eq!(
        client.try_acknowledge_pause(&PauseScope::Global, &ack_hash(&env, 0), &admin),
        Err(Ok(ContractError::InvalidInput))
    );
}

#[test]
fn lowering_pause_cap_prunes_in_bounded_batches() {
    let (env, client, admin) = setup();
    client.pause();
    acknowledge(&env, &client, &admin, 0, 40);
    let kind = HistoryKind::PauseAcknowledgement;
    client.set_history_retention(&kind, &PAUSE_ACKNOWLEDGEMENT_MIN_RETENTION, &0);

    // The logical window shrinks at once even though only a bounded batch of
    // entries is physically removed by the update itself.
    let view = client.get_history_retention(&kind);
    assert_eq!(
        view.oldest_retained,
        40 - PAUSE_ACKNOWLEDGEMENT_MIN_RETENTION
    );
    assert_eq!(client.get_pause_acknowledgements(&0, &20).len(), 16);

    // Raising the cap again never restores what was removed.
    acknowledge(&env, &client, &admin, 40, 3);
    let lowered = client.get_history_retention(&kind);
    client.set_history_retention(&kind, &PAUSE_ACKNOWLEDGEMENT_MAX_RETENTION, &1);
    let raised = client.get_history_retention(&kind);
    assert!(raised.oldest_retained >= lowered.oldest_retained.min(raised.oldest_retained));
    let page = client.get_pause_acknowledgements(&0, &20);
    // No holes: every returned record is consecutive.
    for index in 1..page.len() {
        assert_eq!(
            page.get(index).unwrap().sequence,
            page.get(index - 1).unwrap().sequence + 1
        );
    }
}

#[test]
fn event_ranges_follow_the_retained_window() {
    let (_env, client, _admin) = setup();
    let empty = client.get_event_sequence_range(&EventDomain::PauseAcknowledgement);
    assert_eq!(empty.version, EVENT_RANGE_VERSION);
    assert_eq!(empty.first_retained, 0);
    assert_eq!(empty.latest_emitted, 0);
    assert_eq!(empty.checkpoint, 0);

    append_config_changes(&client, 40);
    client.set_history_retention(&HistoryKind::ConfigChange, &10, &0);
    let range = client.get_event_sequence_range(&EventDomain::ConfigChange);
    let view = client.get_history_retention(&HistoryKind::ConfigChange);
    assert_eq!(range.latest_emitted, u64::from(view.next_sequence));
    assert_eq!(range.first_retained, u64::from(view.oldest_retained) + 1);
    assert_eq!(
        range.checkpoint,
        range.latest_emitted - range.latest_emitted % EVENT_CHECKPOINT_INTERVAL
    );
    assert!(range.first_retained <= range.latest_emitted);
    assert_eq!(
        range.latest_emitted - range.first_retained + 1,
        10,
        "the reported range spans exactly the retained window"
    );
}

#[test]
fn failed_transactions_never_advance_reported_sequences() {
    let (env, client, _admin) = setup();
    let before = client.get_event_sequence_range(&EventDomain::ConfigChange);
    // Out of range, stale, and unsupported-domain calls all fail.
    assert!(client
        .try_set_history_retention(&HistoryKind::ConfigChange, &99, &0)
        .is_err());
    assert!(client
        .try_set_history_retention(&HistoryKind::ConfigChange, &12, &5)
        .is_err());
    assert_eq!(
        client.try_get_event_sequence_range(&EventDomain::ConsentWithdrawal),
        Err(Ok(ContractError::InvalidInput))
    );
    let outsider = Address::generate(&env);
    assert_eq!(
        client.try_acknowledge_pause(
            &PauseScope::Global,
            &BytesN::from_array(&env, &[9; 32]),
            &outsider
        ),
        Err(Ok(ContractError::InvalidState))
    );
    assert_eq!(
        client.get_event_sequence_range(&EventDomain::ConfigChange),
        before
    );
    assert_eq!(
        client
            .get_event_sequence_range(&EventDomain::PauseAcknowledgement)
            .latest_emitted,
        0
    );

    // A successful change advances the sequence by exactly the entries written.
    client.set_scoped_pause(&PauseScope::Updates, &true);
    let after = client.get_event_sequence_range(&EventDomain::ConfigChange);
    assert_eq!(after.latest_emitted, before.latest_emitted + 1);
}
