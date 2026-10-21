use earnproof_shared::IssuerError;
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
    let contract_id = env.register(IssuerRegistryContract, ());
    let client = IssuerRegistryContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let issuer_id = hash(&env, 1);
    client.initialize(&admin);
    client.register_issuer(&issuer_id, &issuer, &hash(&env, 2), &hash(&env, 3));
    (env, client, admin, issuer_id)
}

#[test]
fn compromise_is_immutable_and_blocks_reactivation() {
    let (env, client, _admin, issuer_id) = setup();
    let key_hash = hash(&env, 4);
    client.propose_issuer_signing_key(&issuer_id, &key_hash, &1);
    client.activate_issuer_signing_key(&issuer_id);
    env.ledger().with_mut(|ledger| ledger.sequence_number = 700);

    let record = client.mark_key_compromised_with_reason(&issuer_id, &key_hash, &1, &hash(&env, 5));
    assert_eq!(record.record_version, 1);
    assert_eq!(record.effective_ledger, 700);
    assert_eq!(record.sequence, 0);
    assert!(client.get_active_issuer_signing_key(&issuer_id).is_none());
    assert_eq!(
        client.try_propose_issuer_signing_key(&issuer_id, &key_hash, &1),
        Err(Ok(IssuerError::InvalidTransition))
    );
    assert_eq!(
        client.try_mark_key_compromised_with_reason(&issuer_id, &key_hash, &1, &hash(&env, 6)),
        Err(Ok(IssuerError::InvalidTransition))
    );
}

#[test]
fn retired_key_history_is_bounded_and_unknown_key_is_atomic() {
    let (env, client, _admin, issuer_id) = setup();
    let retired = hash(&env, 7);
    let active = hash(&env, 8);
    client.propose_issuer_signing_key(&issuer_id, &retired, &1);
    client.activate_issuer_signing_key(&issuer_id);
    client.propose_issuer_signing_key(&issuer_id, &active, &1);
    client.activate_issuer_signing_key(&issuer_id);
    client.mark_key_compromised_with_reason(&issuer_id, &retired, &1, &hash(&env, 9));

    let page = client.get_compromised_key_history(&issuer_id, &0, &1);
    assert_eq!(page.len(), 1);
    assert_eq!(page.get(0).unwrap().key_hash, retired);
    assert_eq!(
        client.try_get_compromised_key_history(&issuer_id, &0, &21),
        Err(Ok(IssuerError::BatchTooLarge))
    );
    assert_eq!(
        client.try_mark_key_compromised_with_reason(&issuer_id, &hash(&env, 10), &1, &hash(&env, 11)),
        Err(Ok(IssuerError::InvalidTransition))
    );
    assert_eq!(
        client
            .get_active_issuer_signing_key(&issuer_id)
            .unwrap()
            .key_hash,
        active
    );
}

#[test]
fn non_admin_cannot_mark_a_key() {
    let (env, client, _admin, issuer_id) = setup();
    let key_hash = hash(&env, 12);
    client.propose_issuer_signing_key(&issuer_id, &key_hash, &1);
    let outsider = Address::generate(&env);
    env.mock_auths(&[MockAuth {
        address: &outsider,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "mark_key_compromised_with_reason",
            args: (issuer_id.clone(), key_hash.clone(), 1u32, hash(&env, 13)).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    assert!(client
        .try_mark_key_compromised_with_reason(&issuer_id, &key_hash, &1, &hash(&env, 13))
        .is_err());
    assert!(client
        .get_compromised_signing_key(&issuer_id, &key_hash, &1)
        .is_none());
}
