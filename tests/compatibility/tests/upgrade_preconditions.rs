use earnproof_shared::{
    compute_precondition_digest, ContractError, PauseScope, UPGRADE_TIMELOCK_LEDGERS,
};
use issuer_registry::{IssuerRegistryContract, IssuerRegistryContractClient};
use proof_registry::{ProofRegistryContract, ProofRegistryContractClient};
use protocol_config::{ProtocolConfigContract, ProtocolConfigContractClient};
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::{Address, BytesN, Env, Vec};

fn make_bytes(env: &Env, val: u8) -> BytesN<32> {
    BytesN::from_array(env, &[val; 32])
}

#[test]
fn protocol_config_commits_and_validates_upgrade_preconditions() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let config_id = env.register(ProtocolConfigContract, ());
    let client = ProtocolConfigContractClient::new(&env, &config_id);
    client.initialize(&admin);

    let wasm_hash = make_bytes(&env, 0x11);
    client.approve_upgrade(&wasm_hash, &2);

    let digest = client
        .get_upgrade_precondition(&wasm_hash)
        .expect("precondition digest must be committed at approval");

    let expected_digest =
        compute_precondition_digest(&env, &admin, 1, &Vec::new(&env), false, true);
    assert_eq!(digest, expected_digest);

    // Precondition change: Admin is rotated before execution
    let new_admin = Address::generate(&env);
    client.nominate_admin(&new_admin);
    client.accept_admin();

    // Execution should now panic with precondition mismatch
    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.upgrade_contract(&wasm_hash);
    }));
    assert!(
        res.is_err(),
        "upgrade must reject altered admin precondition"
    );

    // Creating a fresh approval under the new admin restores validity
    client.approve_upgrade(&wasm_hash, &2);
    let new_digest = client
        .get_upgrade_precondition(&wasm_hash)
        .expect("new approval captures new digest");
    assert_ne!(new_digest, digest);
    assert_eq!(
        new_digest,
        compute_precondition_digest(&env, &new_admin, 1, &Vec::new(&env), false, true)
    );
}

#[test]
fn protocol_config_pause_precondition_mutation_invalidates_execution() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let config_id = env.register(ProtocolConfigContract, ());
    let client = ProtocolConfigContractClient::new(&env, &config_id);
    client.initialize(&admin);

    let wasm_hash = make_bytes(&env, 0x12);
    client.approve_upgrade(&wasm_hash, &2);

    // Pause upgrades before execution
    client.set_scoped_pause(&PauseScope::Upgrades, &true);

    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.upgrade_contract(&wasm_hash);
    }));
    assert!(res.is_err(), "upgrade must reject paused precondition");
}

#[test]
fn issuer_registry_commits_precondition_digest_in_approval() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(IssuerRegistryContract, ());
    let client = IssuerRegistryContractClient::new(&env, &contract_id);
    client.initialize(&admin);

    let proposal_id = make_bytes(&env, 0x21);
    let wasm_hash = make_bytes(&env, 0x22);
    client.approve_upgrade(&proposal_id, &wasm_hash, &2);

    let digest = client
        .get_upgrade_precondition()
        .expect("approval record must contain precondition digest");

    let expected = compute_precondition_digest(&env, &admin, 1, &Vec::new(&env), false, true);
    assert_eq!(digest, expected);

    // Timelock must elapse
    env.ledger()
        .set_sequence_number(env.ledger().sequence() + UPGRADE_TIMELOCK_LEDGERS);

    // Admin mutation: rotate admin
    let successor = Address::generate(&env);
    client.nominate_admin(&successor);
    client.accept_admin();

    // Execution must fail with InvalidState due to precondition mismatch
    let res = client.try_upgrade_contract(&wasm_hash, &2);
    assert_eq!(res, Err(Ok(ContractError::InvalidState)));
}

#[test]
fn proof_registry_binds_dependencies_in_precondition_snapshot() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let proto_cfg_id = env.register(ProtocolConfigContract, ());
    let proto_cfg_client = ProtocolConfigContractClient::new(&env, &proto_cfg_id);
    proto_cfg_client.initialize(&admin);

    let issuer_reg_id = env.register(IssuerRegistryContract, ());
    let issuer_reg_client = IssuerRegistryContractClient::new(&env, &issuer_reg_id);
    issuer_reg_client.initialize(&admin);

    let contract_id = env.register(ProofRegistryContract, ());
    let client = ProofRegistryContractClient::new(&env, &contract_id);
    client.initialize(&admin, &issuer_reg_id, &proto_cfg_id);

    let wasm_hash = make_bytes(&env, 0x31);
    client.approve_upgrade(&wasm_hash, &2);

    let digest = client
        .get_upgrade_precondition()
        .expect("precondition digest must exist in approval");

    let mut expected_deps = Vec::new(&env);
    expected_deps.push_back(issuer_reg_id.clone());
    expected_deps.push_back(proto_cfg_id.clone());

    let expected = compute_precondition_digest(&env, &admin, 1, &expected_deps, false, true);
    assert_eq!(digest, expected);

    // Precondition change: paused
    proto_cfg_client.set_scoped_pause(&PauseScope::Upgrades, &true);

    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.upgrade_contract(&wasm_hash);
    }));
    assert!(res.is_err(), "must reject when paused");
}
