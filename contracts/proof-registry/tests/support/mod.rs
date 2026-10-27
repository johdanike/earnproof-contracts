//! Shared synthetic fixtures for the proof-registry integration tests.
#![allow(dead_code)]

use issuer_registry::{IssuerRegistryContract, IssuerRegistryContractClient};
use proof_registry::{ProofRegistryContract, ProofRegistryContractClient};
use protocol_config::{ProtocolConfigContract, ProtocolConfigContractClient};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

pub const ISSUER: &str = "GCATS5YOVB6ROX2WUNKGNQ2MP3GMXDMKSG2O4N5CLX3A6W4PZGZZI55U";

pub struct Deployment {
    pub env: Env,
    pub proofs: ProofRegistryContractClient<'static>,
    pub config: ProtocolConfigContractClient<'static>,
    pub issuer: Address,
}

pub fn hash(env: &Env, value: u8) -> BytesN<32> {
    BytesN::from_array(env, &[value; 32])
}

pub fn deploy() -> Deployment {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let config_id = env.register(ProtocolConfigContract, ());
    let config = ProtocolConfigContractClient::new(&env, &config_id);
    config.initialize(&admin);
    config.approve_schema_version(&1);
    config.approve_proof_type(&hash(&env, 1));
    let issuer_registry_id = env.register(IssuerRegistryContract, ());
    let issuer_registry = IssuerRegistryContractClient::new(&env, &issuer_registry_id);
    issuer_registry.initialize(&admin);
    let issuer = Address::from_str(&env, ISSUER);
    issuer_registry.register_issuer(&hash(&env, 9), &issuer, &hash(&env, 8), &hash(&env, 99));
    let proof_id = env.register(ProofRegistryContract, ());
    let proofs = ProofRegistryContractClient::new(&env, &proof_id);
    proofs.initialize(&admin, &issuer_registry_id, &config_id);
    Deployment {
        env,
        proofs,
        config,
        issuer,
    }
}

/// Registers a typed proof with the given id byte, valid for a long time.
pub fn register(d: &Deployment, id: u8) -> BytesN<32> {
    let proof_id = hash(&d.env, id);
    d.proofs.register_proof_typed(
        &proof_id,
        &hash(&d.env, 2),
        &d.issuer,
        &1,
        &1_000_000,
        &hash(&d.env, 1),
    );
    proof_id
}
