//! Shared fixture for the proof-registry integration tests in this directory.
//! Synthetic addresses and hashes only.
#![allow(dead_code)]

use issuer_registry::{IssuerRegistryContract, IssuerRegistryContractClient};
use proof_registry::{ProofRegistryContract, ProofRegistryContractClient};
use protocol_config::{ProtocolConfigContract, ProtocolConfigContractClient};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

pub fn hash(env: &Env, value: u8) -> BytesN<32> {
    BytesN::from_array(env, &[value; 32])
}

pub struct Fixture {
    pub env: Env,
    pub client: ProofRegistryContractClient<'static>,
    pub config: ProtocolConfigContractClient<'static>,
    pub issuers: IssuerRegistryContractClient<'static>,
    pub admin: Address,
    pub issuer: Address,
    pub issuer_id: BytesN<32>,
}

pub fn setup() -> Fixture {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let config_id = env.register(ProtocolConfigContract, ());
    let config = ProtocolConfigContractClient::new(&env, &config_id);
    config.initialize(&admin);
    config.approve_schema_version(&1);
    config.approve_proof_type(&hash(&env, 1));
    let issuers_id = env.register(IssuerRegistryContract, ());
    let issuers = IssuerRegistryContractClient::new(&env, &issuers_id);
    issuers.initialize(&admin);
    let issuer_id = hash(&env, 9);
    issuers.register_issuer(&issuer_id, &issuer, &hash(&env, 8), &hash(&env, 99));
    let proofs_id = env.register(ProofRegistryContract, ());
    let client = ProofRegistryContractClient::new(&env, &proofs_id);
    client.initialize(&admin, &issuers_id, &config_id);
    Fixture {
        env,
        client,
        config,
        issuers,
        admin,
        issuer,
        issuer_id,
    }
}

impl Fixture {
    /// Registers an immediately active proof and returns its identifier.
    pub fn register(&self, n: u8) -> BytesN<32> {
        let proof_id = hash(&self.env, n);
        self.client.register_proof(
            &proof_id,
            &hash(&self.env, n ^ 0xFF),
            &self.issuer,
            &1,
            &100_000,
            &None,
            &hash(&self.env, 1),
        );
        proof_id
    }

    /// Registers a delayed proof that activates at `activates_at`.
    pub fn register_delayed(&self, n: u8, expires_at: u64, activates_at: u64) -> BytesN<32> {
        let proof_id = hash(&self.env, n);
        self.client.register_proof_with_activation(
            &proof_id,
            &hash(&self.env, n ^ 0xFF),
            &self.issuer,
            &1,
            &expires_at,
            &activates_at,
        );
        proof_id
    }
}
