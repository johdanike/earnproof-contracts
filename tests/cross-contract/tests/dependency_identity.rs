//! Dependency deployment identity (#292).
//!
//! `proof-registry` binds to `issuer-registry` and `protocol-config`. A
//! compatible interface version alone does not prove the dependency belongs to
//! the same network and deployment domain. These tests cover the identity
//! handshake on initialization, dependency replacement, domain pinning, and
//! registration, as documented in `docs/deployment-identity.md`.

use earnproof_shared::{
    deployment_identity, ContractError, DeploymentIdentity, InterfaceVersion, ProofError,
    ProofRegistrationInput, DEPLOYMENT_IDENTITY_VERSION, ISSUER_REGISTRY_INTERFACE_VERSION,
};
use issuer_registry::{IssuerRegistryContract, IssuerRegistryContractClient};
use proof_registry::{ProofRegistryContract, ProofRegistryContractClient};
use protocol_config::{ProtocolConfigContract, ProtocolConfigContractClient};
use soroban_sdk::{
    contract, contractimpl, contracttype, testutils::Address as _, vec, Address, BytesN, Env,
};

fn bytes(env: &Env, value: u8) -> BytesN<32> {
    BytesN::from_array(env, &[value; 32])
}

/// Local mirror of the dependencies' instance key, used only to simulate a
/// dependency whose reported identity changed after binding (for example a
/// redeployed or reconfigured instance at the same address).
#[contracttype]
enum IdentityKey {
    DeploymentDomain,
}

/// An `issuer-registry` stand-in built before deployment identity existed.
#[contract]
pub struct LegacyIssuerRegistry;

#[contractimpl]
impl LegacyIssuerRegistry {
    pub fn interface_version(_env: Env) -> InterfaceVersion {
        ISSUER_REGISTRY_INTERFACE_VERSION
    }

    pub fn is_active_address(_env: Env, _issuer_address: Address) -> bool {
        true
    }
}

#[contracttype]
enum ForgedKey {
    Identity,
}

/// An `issuer-registry` stand-in that reports an arbitrary identity.
#[contract]
pub struct ForgedIssuerRegistry;

#[contractimpl]
impl ForgedIssuerRegistry {
    pub fn set_reported(env: Env, identity: DeploymentIdentity) {
        env.storage()
            .instance()
            .set(&ForgedKey::Identity, &identity);
    }

    pub fn interface_version(_env: Env) -> InterfaceVersion {
        ISSUER_REGISTRY_INTERFACE_VERSION
    }

    pub fn is_active_address(_env: Env, _issuer_address: Address) -> bool {
        true
    }

    pub fn get_deployment_identity(env: Env) -> Option<DeploymentIdentity> {
        env.storage().instance().get(&ForgedKey::Identity)
    }
}

struct Deployment {
    env: Env,
    admin: Address,
}

impl Deployment {
    fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        Deployment { env, admin }
    }

    fn issuer_registry(&self, domain: Option<u8>) -> IssuerRegistryContractClient<'static> {
        let id = self.env.register(IssuerRegistryContract, ());
        let client = IssuerRegistryContractClient::new(&self.env, &id);
        client.initialize(&self.admin);
        if let Some(domain) = domain {
            client.set_deployment_domain(&bytes(&self.env, domain));
        }
        client
    }

    fn protocol_config(&self, domain: Option<u8>) -> ProtocolConfigContractClient<'static> {
        let id = self.env.register(ProtocolConfigContract, ());
        let client = ProtocolConfigContractClient::new(&self.env, &id);
        client.initialize(&self.admin);
        if let Some(domain) = domain {
            client.set_deployment_domain(&bytes(&self.env, domain));
        }
        client
    }

    fn proof_registry(&self) -> ProofRegistryContractClient<'static> {
        let id = self.env.register(ProofRegistryContract, ());
        ProofRegistryContractClient::new(&self.env, &id)
    }
}

#[test]
fn dependencies_expose_a_write_once_identity() {
    let d = Deployment::new();
    let env = &d.env;
    let issuers = d.issuer_registry(None);
    let config = d.protocol_config(None);
    assert_eq!(issuers.get_deployment_identity(), None);
    assert_eq!(config.get_deployment_identity(), None);

    let identity = issuers.set_deployment_domain(&bytes(env, 1));
    assert_eq!(identity.identity_version, DEPLOYMENT_IDENTITY_VERSION);
    assert_eq!(identity.network_id, env.ledger().network_id());
    assert_eq!(identity, deployment_identity(env, &bytes(env, 1)));
    assert_eq!(issuers.get_deployment_identity(), Some(identity.clone()));

    // Replaying the same domain is a no-op; changing it is rejected.
    assert_eq!(issuers.set_deployment_domain(&bytes(env, 1)), identity);
    assert_eq!(
        issuers.try_set_deployment_domain(&bytes(env, 2)),
        Err(Ok(ContractError::DeploymentIdentityMismatch))
    );
    assert_eq!(
        config.try_set_deployment_domain(&bytes(env, 0)),
        Err(Ok(ContractError::InvalidInput))
    );
    assert_eq!(issuers.get_deployment_identity(), Some(identity));
    assert_eq!(config.get_deployment_identity(), None);
}

#[test]
fn only_the_admin_can_set_a_deployment_domain() {
    let d = Deployment::new();
    let issuers = d.issuer_registry(None);
    let config = d.protocol_config(None);
    d.env.set_auths(&[]);
    assert!(issuers
        .try_set_deployment_domain(&bytes(&d.env, 1))
        .is_err());
    assert!(config.try_set_deployment_domain(&bytes(&d.env, 1)).is_err());
    assert_eq!(issuers.get_deployment_identity(), None);
    assert_eq!(config.get_deployment_identity(), None);
}

#[test]
fn initialization_pins_the_domain_when_dependencies_match() {
    let d = Deployment::new();
    let issuers = d.issuer_registry(Some(1));
    let config = d.protocol_config(Some(1));
    let proofs = d.proof_registry();
    let identity = proofs.initialize_in_domain(
        &d.admin,
        &issuers.address,
        &config.address,
        &bytes(&d.env, 1),
    );
    assert_eq!(identity, issuers.get_deployment_identity().unwrap());
    assert_eq!(proofs.get_deployment_identity(), Some(identity));
    assert_eq!(proofs.get_admin(), d.admin);
}

#[test]
fn initialization_with_a_foreign_dependency_is_rejected_atomically() {
    let d = Deployment::new();
    let env = &d.env;
    for (issuer_domain, config_domain) in [
        (Some(1), Some(2)),
        (Some(2), Some(1)),
        (None, Some(1)),
        (Some(1), None),
    ] {
        let issuers = d.issuer_registry(issuer_domain);
        let config = d.protocol_config(config_domain);
        let proofs = d.proof_registry();
        assert_eq!(
            proofs.try_initialize_in_domain(
                &d.admin,
                &issuers.address,
                &config.address,
                &bytes(env, 1)
            ),
            Err(Ok(ContractError::DeploymentIdentityMismatch))
        );
        // Nothing was written: the contract is still uninitialized.
        assert_eq!(
            proofs.try_get_admin(),
            Err(Ok(ContractError::NotInitialized))
        );
        assert_eq!(proofs.get_deployment_identity(), None);
    }

    let issuers = d.issuer_registry(Some(1));
    let config = d.protocol_config(Some(1));
    assert_eq!(
        d.proof_registry().try_initialize_in_domain(
            &d.admin,
            &issuers.address,
            &config.address,
            &bytes(env, 0)
        ),
        Err(Ok(ContractError::InvalidInput))
    );
}

#[test]
fn replacement_from_a_foreign_domain_is_rejected_atomically() {
    let d = Deployment::new();
    let issuers = d.issuer_registry(Some(1));
    let config = d.protocol_config(Some(1));
    let proofs = d.proof_registry();
    proofs.initialize_in_domain(
        &d.admin,
        &issuers.address,
        &config.address,
        &bytes(&d.env, 1),
    );

    let foreign_issuers = d.issuer_registry(Some(2));
    let unset_issuers = d.issuer_registry(None);
    for candidate in [&foreign_issuers, &unset_issuers] {
        assert_eq!(
            proofs.try_set_issuer_registry(&candidate.address),
            Err(Ok(ContractError::DeploymentIdentityMismatch))
        );
    }
    let foreign_config = d.protocol_config(Some(2));
    assert_eq!(
        proofs.try_set_protocol_config(&foreign_config.address),
        Err(Ok(ContractError::DeploymentIdentityMismatch))
    );
    assert_eq!(proofs.get_issuer_registry(), issuers.address);
    assert_eq!(proofs.get_protocol_config(), config.address);

    let same_domain_issuers = d.issuer_registry(Some(1));
    let same_domain_config = d.protocol_config(Some(1));
    proofs.set_issuer_registry(&same_domain_issuers.address);
    proofs.set_protocol_config(&same_domain_config.address);
    assert_eq!(proofs.get_issuer_registry(), same_domain_issuers.address);
    assert_eq!(proofs.get_protocol_config(), same_domain_config.address);
}

#[test]
fn legacy_and_forged_dependencies_never_satisfy_a_pinned_registry() {
    let d = Deployment::new();
    let env = &d.env;
    let issuers = d.issuer_registry(Some(1));
    let config = d.protocol_config(Some(1));
    let proofs = d.proof_registry();
    proofs.initialize_in_domain(&d.admin, &issuers.address, &config.address, &bytes(env, 1));

    // A dependency without `get_deployment_identity` is reported as `None`
    // instead of aborting the call.
    let legacy = env.register(LegacyIssuerRegistry, ());
    assert_eq!(
        proofs.try_set_issuer_registry(&legacy),
        Err(Ok(ContractError::DeploymentIdentityMismatch))
    );

    // Copying the pinned commitment onto another domain does not pass.
    let forged = env.register(ForgedIssuerRegistry, ());
    let forged_client = ForgedIssuerRegistryClient::new(env, &forged);
    let mut identity = deployment_identity(env, &bytes(env, 2));
    identity.commitment = issuers.get_deployment_identity().unwrap().commitment;
    forged_client.set_reported(&identity);
    assert_eq!(
        proofs.try_set_issuer_registry(&forged),
        Err(Ok(ContractError::DeploymentIdentityMismatch))
    );

    // An honest report of the pinned identity passes.
    forged_client.set_reported(&deployment_identity(env, &bytes(env, 1)));
    proofs.set_issuer_registry(&forged);
    assert_eq!(proofs.get_issuer_registry(), forged);
}

#[test]
fn unpinned_registries_keep_legacy_behaviour_until_pinned() {
    let d = Deployment::new();
    let env = &d.env;
    let issuers = d.issuer_registry(None);
    let config = d.protocol_config(None);
    let proofs = d.proof_registry();
    proofs.initialize(&d.admin, &issuers.address, &config.address);
    assert_eq!(proofs.get_deployment_identity(), None);

    // Legacy replacement still works with no identity at all.
    let legacy = env.register(LegacyIssuerRegistry, ());
    proofs.set_issuer_registry(&legacy);

    // Pinning validates the currently bound dependencies first.
    assert_eq!(
        proofs.try_set_deployment_domain(&bytes(env, 1)),
        Err(Ok(ContractError::DeploymentIdentityMismatch))
    );
    assert_eq!(proofs.get_deployment_identity(), None);

    proofs.set_issuer_registry(&issuers.address);
    issuers.set_deployment_domain(&bytes(env, 1));
    config.set_deployment_domain(&bytes(env, 1));
    let identity = proofs.set_deployment_domain(&bytes(env, 1));
    assert_eq!(proofs.get_deployment_identity(), Some(identity));
    assert_eq!(
        proofs.try_set_deployment_domain(&bytes(env, 2)),
        Err(Ok(ContractError::DeploymentIdentityMismatch))
    );
}

#[test]
fn registration_stops_when_a_bound_dependency_changes_identity() {
    let d = Deployment::new();
    let env = &d.env;
    let issuers = d.issuer_registry(Some(1));
    let config = d.protocol_config(Some(1));
    let proofs = d.proof_registry();
    proofs.initialize_in_domain(&d.admin, &issuers.address, &config.address, &bytes(env, 1));

    let issuer_address = Address::generate(env);
    issuers.register_issuer(
        &bytes(env, 10),
        &issuer_address,
        &bytes(env, 11),
        &bytes(env, 12),
    );
    config.approve_schema_version(&1);
    let proof_type = bytes(env, 13);
    config.approve_proof_type(&proof_type);
    let expires_at = env.ledger().timestamp() + 1_000;

    proofs.register_proof(
        &bytes(env, 20),
        &bytes(env, 21),
        &issuer_address,
        &1,
        &expires_at,
        &None,
        &proof_type,
    );

    // The issuer registry at the bound address now reports another domain.
    env.as_contract(&issuers.address, || {
        env.storage()
            .instance()
            .set(&IdentityKey::DeploymentDomain, &bytes(env, 2))
    });
    assert_eq!(
        proofs.try_register_proof(
            &bytes(env, 22),
            &bytes(env, 23),
            &issuer_address,
            &1,
            &expires_at,
            &None,
            &proof_type,
        ),
        Err(Ok(ProofError::DependencyIdentityMismatch))
    );
    let batch = vec![
        env,
        ProofRegistrationInput {
            proof_id_hash: bytes(env, 24),
            commitment_hash: bytes(env, 25),
            schema_version: 1,
            expires_at,
            proof_type: proof_type.clone(),
        },
    ];
    assert_eq!(
        proofs.try_register_proofs_batch(&batch, &issuer_address),
        Err(Ok(ProofError::DependencyIdentityMismatch))
    );
    assert!(proofs.try_get_proof(&bytes(env, 22)).is_err());
    assert!(proofs.try_get_proof(&bytes(env, 24)).is_err());

    env.as_contract(&issuers.address, || {
        env.storage()
            .instance()
            .set(&IdentityKey::DeploymentDomain, &bytes(env, 1))
    });
    proofs.register_proof(
        &bytes(env, 22),
        &bytes(env, 23),
        &issuer_address,
        &1,
        &expires_at,
        &None,
        &proof_type,
    );
    proofs.register_proofs_batch(&batch, &issuer_address);
    assert!(proofs.try_get_proof(&bytes(env, 24)).is_ok());
}
