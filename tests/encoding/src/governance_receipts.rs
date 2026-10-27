//! Tests for Issue #287: Commit governance execution receipt hashes.
//!
//! Executed proposals require canonical, versioned receipt hashes binding proposal state,
//! action commitments, approval thresholds/counts, and resulting configuration versions.
//! Receipts must contain zero private data and can never be created for unapproved, failed,
//! or cancelled executions.

use earnproof_shared::{
    compute_governance_receipt_hash, CriticalAction, CriticalActionPolicy,
    GovernanceExecutionReceipt, GOVERNANCE_RECEIPT_VERSION,
};
use protocol_config::{ProtocolConfigContract, ProtocolConfigContractClient};
use soroban_sdk::testutils::{Address as _, Events, Ledger as _};
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{Address, BytesN, Env, Symbol, Vec};

fn make_bytes(env: &Env, val: u8) -> BytesN<32> {
    BytesN::from_array(env, &[val; 32])
}

fn setup_policy(env: &Env, client: &ProtocolConfigContractClient) -> Vec<Address> {
    let signers = Vec::from_array(
        env,
        [
            Address::generate(env),
            Address::generate(env),
            Address::generate(env),
        ],
    );
    let policy = CriticalActionPolicy {
        enabled: true,
        threshold: 1,
        signers: signers.clone(),
    };
    client.set_critical_action_policy(&policy);
    signers
}

#[test]
fn receipt_hash_canonical_encoding_stability() {
    let env = Env::default();
    let proposal_id = make_bytes(&env, 0x10);
    let action_commitment = make_bytes(&env, 0x20);
    let approval_count = 2_u32;
    let resulting_config_version = 5_u32;
    let execution_ledger = 10_000_u32;
    let execution_timestamp = 1_700_000_000_u64;

    let computed = compute_governance_receipt_hash(
        &env,
        GOVERNANCE_RECEIPT_VERSION,
        &proposal_id,
        &action_commitment,
        approval_count,
        resulting_config_version,
        execution_ledger,
        execution_timestamp,
    );

    // Independent reference computation using raw tuple XDR serialization
    let payload = (
        GOVERNANCE_RECEIPT_VERSION,
        Symbol::new(&env, "gov_exec_receipt"),
        proposal_id.clone(),
        action_commitment.clone(),
        approval_count,
        resulting_config_version,
        execution_ledger,
        execution_timestamp,
    )
        .to_xdr(&env);
    let expected = env.crypto().sha256(&payload).to_bytes();

    assert_eq!(computed, expected);
}

#[test]
fn receipt_hash_field_sensitivity_and_collision_resistance() {
    let env = Env::default();
    let proposal_id = make_bytes(&env, 0x01);
    let action_commitment = make_bytes(&env, 0x02);
    let approval_count = 1_u32;
    let resulting_config_version = 2_u32;
    let execution_ledger = 100_u32;
    let execution_timestamp = 1_000_u64;

    let baseline = compute_governance_receipt_hash(
        &env,
        GOVERNANCE_RECEIPT_VERSION,
        &proposal_id,
        &action_commitment,
        approval_count,
        resulting_config_version,
        execution_ledger,
        execution_timestamp,
    );

    // Version mutation
    assert_ne!(
        baseline,
        compute_governance_receipt_hash(
            &env,
            GOVERNANCE_RECEIPT_VERSION + 1,
            &proposal_id,
            &action_commitment,
            approval_count,
            resulting_config_version,
            execution_ledger,
            execution_timestamp,
        )
    );

    // Proposal ID mutation
    assert_ne!(
        baseline,
        compute_governance_receipt_hash(
            &env,
            GOVERNANCE_RECEIPT_VERSION,
            &make_bytes(&env, 0xFF),
            &action_commitment,
            approval_count,
            resulting_config_version,
            execution_ledger,
            execution_timestamp,
        )
    );

    // Action commitment mutation
    assert_ne!(
        baseline,
        compute_governance_receipt_hash(
            &env,
            GOVERNANCE_RECEIPT_VERSION,
            &proposal_id,
            &make_bytes(&env, 0xAA),
            approval_count,
            resulting_config_version,
            execution_ledger,
            execution_timestamp,
        )
    );

    // Approval count mutation
    assert_ne!(
        baseline,
        compute_governance_receipt_hash(
            &env,
            GOVERNANCE_RECEIPT_VERSION,
            &proposal_id,
            &action_commitment,
            approval_count + 1,
            resulting_config_version,
            execution_ledger,
            execution_timestamp,
        )
    );

    // Config version mutation
    assert_ne!(
        baseline,
        compute_governance_receipt_hash(
            &env,
            GOVERNANCE_RECEIPT_VERSION,
            &proposal_id,
            &action_commitment,
            approval_count,
            resulting_config_version + 1,
            execution_ledger,
            execution_timestamp,
        )
    );

    // Ledger sequence mutation
    assert_ne!(
        baseline,
        compute_governance_receipt_hash(
            &env,
            GOVERNANCE_RECEIPT_VERSION,
            &proposal_id,
            &action_commitment,
            approval_count,
            resulting_config_version,
            execution_ledger + 1,
            execution_timestamp,
        )
    );

    // Timestamp mutation
    assert_ne!(
        baseline,
        compute_governance_receipt_hash(
            &env,
            GOVERNANCE_RECEIPT_VERSION,
            &proposal_id,
            &action_commitment,
            approval_count,
            resulting_config_version,
            execution_ledger,
            execution_timestamp + 1,
        )
    );
}

#[test]
fn receipt_hash_boundary_values() {
    let env = Env::default();
    let zero_bytes = make_bytes(&env, 0);
    let max_bytes = make_bytes(&env, 0xFF);

    // Boundary at minimum values
    let min_hash = compute_governance_receipt_hash(&env, 0, &zero_bytes, &zero_bytes, 0, 0, 0, 0);
    assert_ne!(min_hash, zero_bytes);

    // Boundary at maximum values
    let max_hash = compute_governance_receipt_hash(
        &env,
        u32::MAX,
        &max_bytes,
        &max_bytes,
        u32::MAX,
        u32::MAX,
        u32::MAX,
        u64::MAX,
    );
    assert_ne!(max_hash, min_hash);
    assert_ne!(max_hash, zero_bytes);
}

#[test]
fn protocol_config_governance_execution_generates_receipt_and_emits_event() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let config_id = env.register(ProtocolConfigContract, ());
    let client = ProtocolConfigContractClient::new(&env, &config_id);
    client.initialize(&admin);

    let signers = setup_policy(&env, &client);
    let approver = signers.get(0).unwrap();

    let action = CriticalAction::SchemaApproval(22);
    let proposal_id = client.propose_critical_action(&action);

    // Before approval/execution, receipt queries return None
    assert_eq!(client.get_governance_receipt(&proposal_id), None);
    assert_eq!(client.get_latest_governance_receipt(), None);

    // Approve the critical action
    client.approve_critical_action(&proposal_id, &approver);

    // Advance ledger
    env.ledger().with_mut(|li| {
        li.sequence_number = 500;
        li.timestamp = 1_700_000_500;
    });

    // Execute critical action
    client.execute_critical_action(&proposal_id);
    let events = env.events().all();

    // Receipt must now exist and be queryable
    let receipt: GovernanceExecutionReceipt = client
        .get_governance_receipt(&proposal_id)
        .expect("receipt must exist for executed proposal");

    assert_eq!(receipt.receipt_version, GOVERNANCE_RECEIPT_VERSION);
    assert_eq!(receipt.proposal_id, proposal_id);
    assert_eq!(receipt.approval_count, 1);
    assert_eq!(receipt.execution_ledger, 500);
    assert_eq!(receipt.execution_timestamp, 1_700_000_500);
    assert_eq!(receipt.resulting_config_version, 3);
    assert_eq!(
        receipt.resulting_config_version,
        client.get_config_version()
    );

    let expected_hash = compute_governance_receipt_hash(
        &env,
        GOVERNANCE_RECEIPT_VERSION,
        &proposal_id,
        &receipt.action_commitment,
        receipt.approval_count,
        receipt.resulting_config_version,
        receipt.execution_ledger,
        receipt.execution_timestamp,
    );
    assert_eq!(receipt.receipt_hash, expected_hash);

    // Latest receipt matches
    assert_eq!(
        client.get_latest_governance_receipt(),
        Some(receipt.clone())
    );

    // Verify contract event was emitted
    let receipt_event_found = events.events().iter().any(|e| {
        use soroban_sdk::xdr::{ContractEventBody, ScVal};
        let ContractEventBody::V0(body) = &e.body;
        if let Some(ScVal::Symbol(sym)) = body.topics.first() {
            return sym.as_slice() == b"governance_receipt_committed";
        }
        false
    });
    assert!(
        receipt_event_found,
        "governance_receipt_committed event must be emitted"
    );
}

#[test]
fn unapproved_or_cancelled_proposals_never_commit_receipts() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let config_id = env.register(ProtocolConfigContract, ());
    let client = ProtocolConfigContractClient::new(&env, &config_id);
    client.initialize(&admin);

    let _signers = setup_policy(&env, &client);

    let proposal_id = client.propose_critical_action(&CriticalAction::SchemaApproval(30));

    // Attempting execution before approval fails
    let res = client.try_execute_critical_action(&proposal_id);
    assert!(res.is_err());
    assert_eq!(client.get_governance_receipt(&proposal_id), None);

    // Cancel proposal
    client.cancel_critical_action(&proposal_id);

    // Cannot execute cancelled proposal
    let res = client.try_execute_critical_action(&proposal_id);
    assert!(res.is_err());
    assert_eq!(client.get_governance_receipt(&proposal_id), None);
}

#[test]
fn governance_receipt_replay_prevention() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let config_id = env.register(ProtocolConfigContract, ());
    let client = ProtocolConfigContractClient::new(&env, &config_id);
    client.initialize(&admin);

    let signers = setup_policy(&env, &client);
    let approver = signers.get(0).unwrap();

    let proposal_id = client.propose_critical_action(&CriticalAction::SchemaApproval(31));
    client.approve_critical_action(&proposal_id, &approver);
    client.execute_critical_action(&proposal_id);

    let first_receipt = client
        .get_governance_receipt(&proposal_id)
        .expect("first receipt exists");

    // Second execution must fail (already executed)
    let re_exec = client.try_execute_critical_action(&proposal_id);
    assert!(re_exec.is_err());

    // Receipt unchanged
    assert_eq!(
        client.get_governance_receipt(&proposal_id),
        Some(first_receipt)
    );
}
