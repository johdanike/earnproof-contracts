use earnproof_shared::{
    IssuerError, SigningKeyLifecycleTransition, MAX_SIGNING_KEY_LIFECYCLE_PAGE,
};
use issuer_registry::{IssuerRegistryContract, IssuerRegistryContractClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger as _, MockAuth, MockAuthInvoke},
    Address, BytesN, Env, IntoVal,
};

fn hash(env: &Env, value: u8) -> BytesN<32> {
    BytesN::from_array(env, &[value; 32])
}

fn setup() -> (
    Env,
    IssuerRegistryContractClient<'static>,
    Address,
    BytesN<32>,
) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|ledger| ledger.sequence_number = 50);
    let contract_id = env.register(IssuerRegistryContract, ());
    let client = IssuerRegistryContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let issuer_id = hash(&env, 1);
    client.initialize(&admin);
    client.register_issuer(
        &issuer_id,
        &Address::generate(&env),
        &hash(&env, 2),
        &hash(&env, 3),
    );
    (env, client, admin, issuer_id)
}

#[test]
fn rotation_sequences_proposal_retirement_and_activation_deterministically() {
    let (env, client, _admin, issuer_id) = setup();
    let first = hash(&env, 10);
    let second = hash(&env, 11);
    client.propose_issuer_signing_key(&issuer_id, &first, &1);
    client.activate_issuer_signing_key(&issuer_id);
    client.propose_issuer_signing_key(&issuer_id, &second, &1);
    client.activate_issuer_signing_key(&issuer_id);

    assert_eq!(client.get_signing_key_usage_sequence(&issuer_id), 5);
    let history = client.get_signing_key_lifecycle(&issuer_id, &1, &5);
    assert_eq!(history.len(), 5);
    assert_eq!(
        history.get(0).unwrap().transition,
        SigningKeyLifecycleTransition::Proposed
    );
    assert_eq!(
        history.get(1).unwrap().transition,
        SigningKeyLifecycleTransition::Activated
    );
    assert_eq!(
        history.get(2).unwrap().transition,
        SigningKeyLifecycleTransition::Proposed
    );
    assert_eq!(
        history.get(3).unwrap().transition,
        SigningKeyLifecycleTransition::Retired
    );
    assert_eq!(
        history.get(4).unwrap().transition,
        SigningKeyLifecycleTransition::Activated
    );
    assert_eq!(history.get(4).unwrap().effective_ledger, 50);

    let retired = client.retire_issuer_signing_key(&issuer_id);
    assert_eq!(retired.sequence, 6);
    assert_eq!(retired.transition, SigningKeyLifecycleTransition::Retired);
    assert!(client.get_active_issuer_signing_key(&issuer_id).is_none());
}

#[test]
fn compromise_is_terminal_and_failed_transitions_do_not_advance() {
    let (env, client, _admin, issuer_id) = setup();
    let key_hash = hash(&env, 20);
    assert_eq!(
        client.try_activate_issuer_signing_key(&issuer_id),
        Err(Ok(IssuerError::InvalidTransition))
    );
    assert_eq!(client.get_signing_key_usage_sequence(&issuer_id), 0);

    client.propose_issuer_signing_key(&issuer_id, &key_hash, &1);
    let compromised = client.record_signing_key_compromise(&issuer_id, &key_hash, &1);
    assert_eq!(compromised.sequence, 2);
    assert_eq!(
        lifecycle.transition,
        SigningKeyLifecycleTransition::Compromised
    );
    assert_eq!(
        client.try_propose_issuer_signing_key(&issuer_id, &key_hash, &1),
        Err(Ok(IssuerError::InvalidTransition))
    );
    assert_eq!(client.get_signing_key_usage_sequence(&issuer_id), 2);
    let state = client
        .get_issuer_signing_key_state(&issuer_id, &key_hash, &1)
        .unwrap();
    assert_eq!(state.sequence, 2);
    assert_eq!(state.transition, SigningKeyLifecycleTransition::Compromised);
}

#[test]
fn lifecycle_queries_are_bounded_and_retirement_is_admin_only() {
    let (env, client, _admin, issuer_id) = setup();
    let key_hash = hash(&env, 30);
    let reason = hash(&env, 31);
    assert_eq!(
        client.try_record_signing_key_compromise(&issuer_id, &key_hash, &1),
        Err(Ok(IssuerError::InvalidTransition))
    );
    assert_eq!(client.get_signing_key_usage_sequence(&issuer_id), 0);
    assert_eq!(
        client
            .try_get_signing_key_lifecycle(&issuer_id, &0, &(MAX_SIGNING_KEY_LIFECYCLE_PAGE + 1),),
        Err(Ok(IssuerError::BatchTooLarge))
    );

    client.propose_issuer_signing_key(&issuer_id, &key_hash, &1);
    client.activate_issuer_signing_key(&issuer_id);
    let outsider = Address::generate(&env);
    env.mock_auths(&[MockAuth {
        address: &outsider,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "retire_issuer_signing_key",
            args: (issuer_id.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    assert!(client.try_retire_issuer_signing_key(&issuer_id).is_err());
    assert_eq!(client.get_signing_key_usage_sequence(&issuer_id), 2);
    assert!(client.get_active_issuer_signing_key(&issuer_id).is_some());
}
