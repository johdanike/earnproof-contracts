//! Deployment identity commitments for cross-contract dependencies.
//!
//! An interface version says *what* a dependency implements. It does not say
//! *which deployment* the dependency belongs to: a testnet `issuer-registry`
//! and a mainnet `issuer-registry` built from the same code report the same
//! interface version. A consumer that only checks the interface could be bound
//! to a dependency from the wrong network or the wrong deployment domain.
//!
//! Every contract therefore exposes `get_deployment_identity`, returning a
//! [`DeploymentIdentity`] built from:
//!
//! * the ledger network id (`env.ledger().network_id()`, the SHA-256 of the
//!   network passphrase), which the contract cannot choose; and
//! * a governed, write-once 32-byte deployment domain set by the admin with
//!   `set_deployment_domain`. The domain is an opaque synthetic label (for
//!   example the SHA-256 of `"earnproof:testnet:blue"`); it is never a secret.
//!
//! The `commitment` field is the SHA-256 of the XDR encoding of
//! `(identity_version, network_id, deployment_domain)`, so two identities are
//! equal exactly when their commitments are equal.
//!
//! A contract whose domain has never been set reports `None` and is treated as
//! a legacy deployment. See `docs/deployment-identity.md` for the consumer
//! rules.

use soroban_sdk::{contracttype, xdr::ToXdr, BytesN, Env};

/// Encoding version of [`DeploymentIdentity`] and of its commitment preimage.
pub const DEPLOYMENT_IDENTITY_VERSION: u32 = 1;

/// The deployment identity a contract reports to its consumers.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeploymentIdentity {
    pub identity_version: u32,
    pub network_id: BytesN<32>,
    pub deployment_domain: BytesN<32>,
    pub commitment: BytesN<32>,
}

/// Computes the identity commitment for a network id and deployment domain.
pub fn deployment_identity_commitment(
    env: &Env,
    network_id: &BytesN<32>,
    deployment_domain: &BytesN<32>,
) -> BytesN<32> {
    let preimage = (
        DEPLOYMENT_IDENTITY_VERSION,
        network_id.clone(),
        deployment_domain.clone(),
    )
        .to_xdr(env);
    env.crypto().sha256(&preimage).into()
}

/// Builds the identity for `deployment_domain` on the current network.
pub fn deployment_identity(env: &Env, deployment_domain: &BytesN<32>) -> DeploymentIdentity {
    let network_id = env.ledger().network_id();
    let commitment = deployment_identity_commitment(env, &network_id, deployment_domain);
    DeploymentIdentity {
        identity_version: DEPLOYMENT_IDENTITY_VERSION,
        network_id,
        deployment_domain: deployment_domain.clone(),
        commitment,
    }
}

/// A deployment domain must be a non-zero 32-byte value.
pub fn is_valid_deployment_domain(env: &Env, deployment_domain: &BytesN<32>) -> bool {
    deployment_domain != &BytesN::from_array(env, &[0u8; 32])
}

/// Consumer-side check: a dependency matches when it reports an identity of a
/// known version whose commitment recomputes and equals `expected`.
///
/// A dependency that reports no identity (legacy, or domain never set) does
/// not match a consumer that has pinned a domain.
pub fn dependency_identity_matches(
    env: &Env,
    expected: &DeploymentIdentity,
    actual: &Option<DeploymentIdentity>,
) -> bool {
    match actual {
        Some(actual) => {
            actual.identity_version == DEPLOYMENT_IDENTITY_VERSION
                && actual.commitment
                    == deployment_identity_commitment(
                        env,
                        &actual.network_id,
                        &actual.deployment_domain,
                    )
                && actual.commitment == expected.commitment
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes(env: &Env, value: u8) -> BytesN<32> {
        BytesN::from_array(env, &[value; 32])
    }

    #[test]
    fn identity_commitment_is_deterministic_and_domain_separated() {
        let env = Env::default();
        let a = deployment_identity(&env, &bytes(&env, 1));
        let again = deployment_identity(&env, &bytes(&env, 1));
        let b = deployment_identity(&env, &bytes(&env, 2));
        assert_eq!(a, again);
        assert_ne!(a.commitment, b.commitment);
        assert_eq!(a.identity_version, DEPLOYMENT_IDENTITY_VERSION);
        assert_eq!(a.network_id, env.ledger().network_id());
    }

    #[test]
    fn network_id_is_bound_into_the_commitment() {
        let env = Env::default();
        let domain = bytes(&env, 1);
        let here = deployment_identity_commitment(&env, &bytes(&env, 9), &domain);
        let there = deployment_identity_commitment(&env, &bytes(&env, 8), &domain);
        assert_ne!(here, there);
    }

    #[test]
    fn matching_rejects_missing_forged_and_foreign_identities() {
        let env = Env::default();
        let expected = deployment_identity(&env, &bytes(&env, 1));
        assert!(dependency_identity_matches(
            &env,
            &expected,
            &Some(expected.clone())
        ));
        assert!(!dependency_identity_matches(&env, &expected, &None));

        let foreign = deployment_identity(&env, &bytes(&env, 2));
        assert!(!dependency_identity_matches(
            &env,
            &expected,
            &Some(foreign)
        ));

        // Copying the expected commitment onto a different domain is detected.
        let mut forged = deployment_identity(&env, &bytes(&env, 2));
        forged.commitment = expected.commitment.clone();
        assert!(!dependency_identity_matches(&env, &expected, &Some(forged)));

        let mut future = expected.clone();
        future.identity_version = DEPLOYMENT_IDENTITY_VERSION + 1;
        assert!(!dependency_identity_matches(&env, &expected, &Some(future)));
    }

    #[test]
    fn zero_domain_is_invalid() {
        let env = Env::default();
        assert!(!is_valid_deployment_domain(&env, &bytes(&env, 0)));
        assert!(is_valid_deployment_domain(&env, &bytes(&env, 1)));
    }
}
