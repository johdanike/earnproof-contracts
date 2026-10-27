//! Contract-independent rules for four governance behaviours.
//!
//! The contracts stay thin: they read and write storage and publish events,
//! while every decision lives here as a pure function so it can be tested
//! without deploying a contract.
//!
//! * Revocation authority binding: the policy version, actor role and
//!   authorization domain that justified a revocation.
//! * Commitment hash algorithm registry: approve, activate and deprecate
//!   lifecycle, and which uses each status permits.
//! * Activation acknowledgement: when a delayed proof may be acknowledged.
//! * Dispute resolver conflicts: participant commitments and conflict
//!   detection that survives address rotation.

use soroban_sdk::{contracterror, contracttype, xdr::ToXdr, Address, Bytes, BytesN, Env};

use crate::{DisputeStatus, ProofError, ProofStatus};

// ── revocation authority binding ────────────────────────────────────────────

/// Authority policy version assigned at `initialize`. Governance can only
/// raise it afterwards.
pub const INITIAL_AUTHORITY_POLICY_VERSION: u32 = 1;

const REVOCATION_DOMAIN: &[u8] = b"earnproof.proof-registry.revocation.v1";
const PARTICIPANT_DOMAIN: &[u8] = b"earnproof.proof-registry.participant.v1";

/// Role under which a revocation was authorized.
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RevocationActorRole {
    /// The proof's own recorded issuer address.
    Issuer,
    /// The proof-registry administrator.
    Admin,
}

impl RevocationActorRole {
    fn tag(self) -> u8 {
        match self {
            RevocationActorRole::Issuer => 1,
            RevocationActorRole::Admin => 2,
        }
    }
}

/// Immutable evidence of how a revocation was authorized. Written exactly
/// once, in the same call that revokes the proof, and never rewritten: a later
/// policy change cannot alter it.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevocationAuthority {
    pub proof_id_hash: BytesN<32>,
    /// Authority policy version in force when the revocation committed.
    pub policy_version: u32,
    pub actor_role: RevocationActorRole,
    pub actor: Address,
    /// Hash binding registry, role and policy version together.
    pub authorization_domain: BytesN<32>,
    pub revoked_at: u64,
    pub revoked_ledger: u32,
}

/// Domain separating revocation authorizations by registry instance, role and
/// policy version. Two different versions or roles never share a domain.
pub fn revocation_authorization_domain(
    env: &Env,
    registry: &Address,
    role: RevocationActorRole,
    policy_version: u32,
) -> BytesN<32> {
    let mut payload = Bytes::from_slice(env, REVOCATION_DOMAIN);
    payload.append(&registry.clone().to_xdr(env));
    payload.push_back(role.tag());
    payload.append(&Bytes::from_slice(env, &policy_version.to_be_bytes()));
    env.crypto().sha256(&payload).to_bytes()
}

/// A caller that pins the policy version it expects is rejected when the
/// registry has moved on. Equality is exact: both older and newer values fail.
pub fn require_policy_version(expected: u32, current: u32) -> Result<(), ProofError> {
    if expected == current {
        Ok(())
    } else {
        Err(ProofError::StalePolicyVersion)
    }
}

/// A policy version may only increase, and zero is never valid.
pub fn next_policy_version_is_valid(current: u32, next: u32) -> bool {
    next > current
}

// ── commitment hash algorithm registry ──────────────────────────────────────

/// Errors specific to protocol-config (100-199).
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ProtocolConfigError {
    /// No algorithm is registered under this identifier.
    HashAlgorithmNotFound = 100,
    /// The requested lifecycle step is not allowed from the current status.
    InvalidHashAlgorithmTransition = 101,
    /// The algorithm is deprecated and cannot create new commitments.
    HashAlgorithmDeprecated = 102,
    /// Identifier zero is reserved and never valid.
    InvalidHashAlgorithmId = 103,
    /// The algorithm is approved but not yet activated.
    HashAlgorithmNotActive = 104,
    /// The contract has no administrator or has been decommissioned.
    RegistryUnavailable = 105,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum HashAlgorithmStatus {
    /// Approved by governance, not yet usable for new commitments.
    Approved,
    /// Usable for new commitments and for verification.
    Active,
    /// Verification only. Never usable for new commitments again.
    Deprecated,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HashAlgorithmRecord {
    pub id: u32,
    pub status: HashAlgorithmStatus,
    pub approved_at: u64,
    /// Zero until activated.
    pub activated_at: u64,
    /// Zero until deprecated.
    pub deprecated_at: u64,
    /// Number of lifecycle changes applied; starts at 1 on approval.
    pub revision: u32,
}

/// What a caller wants to do with an algorithm.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum HashAlgorithmUse {
    CreateCommitment,
    VerifyCommitment,
}

pub fn hash_algorithm_approve(
    existing: Option<HashAlgorithmRecord>,
    id: u32,
    now: u64,
) -> Result<HashAlgorithmRecord, ProtocolConfigError> {
    if id == 0 {
        return Err(ProtocolConfigError::InvalidHashAlgorithmId);
    }
    if existing.is_some() {
        return Err(ProtocolConfigError::InvalidHashAlgorithmTransition);
    }
    Ok(HashAlgorithmRecord {
        id,
        status: HashAlgorithmStatus::Approved,
        approved_at: now,
        activated_at: 0,
        deprecated_at: 0,
        revision: 1,
    })
}

pub fn hash_algorithm_activate(
    existing: Option<HashAlgorithmRecord>,
    now: u64,
) -> Result<HashAlgorithmRecord, ProtocolConfigError> {
    let mut record = existing.ok_or(ProtocolConfigError::HashAlgorithmNotFound)?;
    if record.status != HashAlgorithmStatus::Approved {
        return Err(ProtocolConfigError::InvalidHashAlgorithmTransition);
    }
    record.status = HashAlgorithmStatus::Active;
    record.activated_at = now;
    record.revision += 1;
    Ok(record)
}

/// Deprecation is allowed from `Approved` or `Active` and is terminal.
pub fn hash_algorithm_deprecate(
    existing: Option<HashAlgorithmRecord>,
    now: u64,
) -> Result<HashAlgorithmRecord, ProtocolConfigError> {
    let mut record = existing.ok_or(ProtocolConfigError::HashAlgorithmNotFound)?;
    if record.status == HashAlgorithmStatus::Deprecated {
        return Err(ProtocolConfigError::InvalidHashAlgorithmTransition);
    }
    record.status = HashAlgorithmStatus::Deprecated;
    record.deprecated_at = now;
    record.revision += 1;
    Ok(record)
}

/// Whether `record` permits `usage`. Deprecated algorithms stay verifiable so
/// historical records keep validating, but never create new commitments.
pub fn hash_algorithm_permits(
    record: Option<&HashAlgorithmRecord>,
    usage: HashAlgorithmUse,
) -> Result<(), ProtocolConfigError> {
    let record = record.ok_or(ProtocolConfigError::HashAlgorithmNotFound)?;
    match (record.status, usage) {
        (HashAlgorithmStatus::Active, _) => Ok(()),
        (HashAlgorithmStatus::Deprecated, HashAlgorithmUse::VerifyCommitment) => Ok(()),
        (HashAlgorithmStatus::Deprecated, HashAlgorithmUse::CreateCommitment) => {
            Err(ProtocolConfigError::HashAlgorithmDeprecated)
        }
        (HashAlgorithmStatus::Approved, _) => Err(ProtocolConfigError::HashAlgorithmNotActive),
    }
}

// ── activation acknowledgement ──────────────────────────────────────────────

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ActivationActorRole {
    Issuer,
    /// An address the issuer authorized to acknowledge on its behalf.
    Delegate,
}

/// Why a proof cannot be acknowledged as activated.
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ActivationBlocker {
    /// The proof was revoked.
    Revoked,
    /// The proof has been superseded by a successor, so it was cancelled.
    Superseded,
    /// A dispute is open, or was resolved in the disputant's favor.
    Disputed,
    /// The proof expired before the acknowledgement.
    Expired,
    /// The proof was registered without an activation delay.
    NotDelayed,
    /// The activation time has not been reached.
    NotYetActive,
}

/// Auditable record of a delayed proof becoming eligible. Written once.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivationAcknowledgement {
    pub proof_id_hash: BytesN<32>,
    pub acknowledged_by: Address,
    pub actor_role: ActivationActorRole,
    pub ledger: u32,
    pub timestamp: u64,
    /// Registry-wide acknowledgement number, starting at 1.
    pub sequence: u64,
    /// The proof's `activates_at`, copied so the record stands alone.
    pub activates_at: u64,
}

/// First reason that stops an acknowledgement, in a fixed order, or `None`
/// when the proof is eligible. Boundaries: `now == activates_at` is eligible
/// and `now == expires_at` is still eligible, matching proof validity.
pub fn activation_blocker(
    now: u64,
    status: &ProofStatus,
    activates_at: u64,
    expires_at: u64,
    dispute: Option<DisputeStatus>,
    has_successor: bool,
) -> Option<ActivationBlocker> {
    if *status == ProofStatus::Revoked {
        return Some(ActivationBlocker::Revoked);
    }
    if has_successor {
        return Some(ActivationBlocker::Superseded);
    }
    if matches!(
        dispute,
        Some(DisputeStatus::Open) | Some(DisputeStatus::Resolved)
    ) {
        return Some(ActivationBlocker::Disputed);
    }
    if now > expires_at {
        return Some(ActivationBlocker::Expired);
    }
    if activates_at == 0 {
        return Some(ActivationBlocker::NotDelayed);
    }
    if now < activates_at {
        return Some(ActivationBlocker::NotYetActive);
    }
    None
}

// ── dispute resolver conflicts of interest ──────────────────────────────────

/// Relationship that disqualifies a resolver.
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ConflictRole {
    Issuer,
    SubjectAuthorizedActor,
    EvidenceSubmitter,
}

/// A party to a dispute, identified both by address and by a stable
/// commitment that outlives any single address.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Participant {
    pub role: ConflictRole,
    pub address: Address,
    pub commitment: BytesN<32>,
}

/// Default commitment for an address that never bound one. It only matches the
/// same address, so rotation protection needs a bound or issuer commitment.
pub fn default_participant_commitment(env: &Env, address: &Address) -> BytesN<32> {
    let mut payload = Bytes::from_slice(env, PARTICIPANT_DOMAIN);
    payload.append(&address.clone().to_xdr(env));
    env.crypto().sha256(&payload).to_bytes()
}

/// First participant that conflicts with the resolver, if any. A conflict is
/// an equal address or an equal commitment, so a resolver cannot escape by
/// rotating to a new address that carries the same commitment.
pub fn find_resolver_conflict(
    resolver_address: &Address,
    resolver_commitment: &BytesN<32>,
    participants: &[Participant],
) -> Option<ConflictRole> {
    participants
        .iter()
        .find(|p| p.address == *resolver_address || p.commitment == *resolver_commitment)
        .map(|p| p.role)
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    fn bytes(env: &Env, fill: u8) -> BytesN<32> {
        BytesN::from_array(env, &[fill; 32])
    }

    #[test]
    fn domain_differs_by_role_version_and_registry() {
        let env = Env::default();
        let a = Address::generate(&env);
        let b = Address::generate(&env);
        let base = revocation_authorization_domain(&env, &a, RevocationActorRole::Issuer, 1);
        assert_eq!(
            base,
            revocation_authorization_domain(&env, &a, RevocationActorRole::Issuer, 1)
        );
        assert_ne!(
            base,
            revocation_authorization_domain(&env, &a, RevocationActorRole::Admin, 1)
        );
        assert_ne!(
            base,
            revocation_authorization_domain(&env, &a, RevocationActorRole::Issuer, 2)
        );
        assert_ne!(
            base,
            revocation_authorization_domain(&env, &b, RevocationActorRole::Issuer, 1)
        );
    }

    #[test]
    fn policy_version_pin_is_exact() {
        assert_eq!(require_policy_version(3, 3), Ok(()));
        assert_eq!(
            require_policy_version(2, 3),
            Err(ProofError::StalePolicyVersion)
        );
        assert_eq!(
            require_policy_version(4, 3),
            Err(ProofError::StalePolicyVersion)
        );
    }

    #[test]
    fn policy_version_only_increases() {
        assert!(next_policy_version_is_valid(1, 2));
        assert!(!next_policy_version_is_valid(2, 2));
        assert!(!next_policy_version_is_valid(2, 1));
        assert!(!next_policy_version_is_valid(0, 0));
    }

    #[test]
    fn algorithm_lifecycle_and_permissions() {
        let approved = hash_algorithm_approve(None, 7, 10).unwrap();
        assert_eq!(approved.status, HashAlgorithmStatus::Approved);
        assert_eq!(approved.revision, 1);
        assert_eq!(
            hash_algorithm_permits(Some(&approved), HashAlgorithmUse::CreateCommitment),
            Err(ProtocolConfigError::HashAlgorithmNotActive)
        );

        let active = hash_algorithm_activate(Some(approved), 20).unwrap();
        assert_eq!(active.activated_at, 20);
        assert_eq!(
            hash_algorithm_permits(Some(&active), HashAlgorithmUse::CreateCommitment),
            Ok(())
        );
        assert_eq!(
            hash_algorithm_permits(Some(&active), HashAlgorithmUse::VerifyCommitment),
            Ok(())
        );

        let deprecated = hash_algorithm_deprecate(Some(active), 30).unwrap();
        assert_eq!(deprecated.deprecated_at, 30);
        assert_eq!(deprecated.revision, 3);
        assert_eq!(
            hash_algorithm_permits(Some(&deprecated), HashAlgorithmUse::VerifyCommitment),
            Ok(())
        );
        assert_eq!(
            hash_algorithm_permits(Some(&deprecated), HashAlgorithmUse::CreateCommitment),
            Err(ProtocolConfigError::HashAlgorithmDeprecated)
        );
    }

    #[test]
    fn algorithm_transitions_are_strict() {
        assert_eq!(
            hash_algorithm_approve(None, 0, 1),
            Err(ProtocolConfigError::InvalidHashAlgorithmId)
        );
        let approved = hash_algorithm_approve(None, 1, 1).unwrap();
        assert_eq!(
            hash_algorithm_approve(Some(approved.clone()), 1, 2),
            Err(ProtocolConfigError::InvalidHashAlgorithmTransition)
        );
        assert_eq!(
            hash_algorithm_activate(None, 1),
            Err(ProtocolConfigError::HashAlgorithmNotFound)
        );
        let active = hash_algorithm_activate(Some(approved.clone()), 2).unwrap();
        assert_eq!(
            hash_algorithm_activate(Some(active.clone()), 3),
            Err(ProtocolConfigError::InvalidHashAlgorithmTransition)
        );
        // An approved algorithm may be withdrawn before it ever activates.
        let withdrawn = hash_algorithm_deprecate(Some(approved), 4).unwrap();
        assert_eq!(
            hash_algorithm_activate(Some(withdrawn.clone()), 5),
            Err(ProtocolConfigError::InvalidHashAlgorithmTransition)
        );
        assert_eq!(
            hash_algorithm_deprecate(Some(withdrawn), 6),
            Err(ProtocolConfigError::InvalidHashAlgorithmTransition)
        );
        assert_eq!(
            hash_algorithm_permits(None, HashAlgorithmUse::VerifyCommitment),
            Err(ProtocolConfigError::HashAlgorithmNotFound)
        );
    }

    #[test]
    fn activation_boundaries_are_exact() {
        let st = ProofStatus::Active;
        let check = |now| activation_blocker(now, &st, 100, 200, None, false);
        assert_eq!(check(99), Some(ActivationBlocker::NotYetActive));
        assert_eq!(check(100), None);
        assert_eq!(check(200), None);
        assert_eq!(check(201), Some(ActivationBlocker::Expired));
    }

    #[test]
    fn activation_blockers_have_fixed_precedence() {
        let active = ProofStatus::Active;
        let revoked = ProofStatus::Revoked;
        let open = Some(DisputeStatus::Open);
        assert_eq!(
            activation_blocker(150, &revoked, 100, 200, open, true),
            Some(ActivationBlocker::Revoked)
        );
        assert_eq!(
            activation_blocker(150, &active, 100, 200, open, true),
            Some(ActivationBlocker::Superseded)
        );
        assert_eq!(
            activation_blocker(150, &active, 100, 200, open, false),
            Some(ActivationBlocker::Disputed)
        );
        assert_eq!(
            activation_blocker(150, &active, 100, 200, Some(DisputeStatus::Resolved), false),
            Some(ActivationBlocker::Disputed)
        );
        // Withdrawn and rejected disputes no longer block.
        assert_eq!(
            activation_blocker(
                150,
                &active,
                100,
                200,
                Some(DisputeStatus::Withdrawn),
                false
            ),
            None
        );
        assert_eq!(
            activation_blocker(150, &active, 100, 200, Some(DisputeStatus::Rejected), false),
            None
        );
        assert_eq!(
            activation_blocker(150, &active, 0, 200, None, false),
            Some(ActivationBlocker::NotDelayed)
        );
    }

    #[test]
    fn conflict_detected_by_address_and_by_commitment() {
        let env = Env::default();
        let resolver = Address::generate(&env);
        let rotated = Address::generate(&env);
        let other = Address::generate(&env);
        let shared_commitment = bytes(&env, 9);
        let participants = [
            Participant {
                role: ConflictRole::Issuer,
                address: other.clone(),
                commitment: bytes(&env, 1),
            },
            Participant {
                role: ConflictRole::EvidenceSubmitter,
                address: resolver.clone(),
                commitment: bytes(&env, 2),
            },
        ];
        assert_eq!(
            find_resolver_conflict(&resolver, &bytes(&env, 5), &participants),
            Some(ConflictRole::EvidenceSubmitter)
        );

        // Rotated resolver address, same commitment as the issuer: still blocked.
        let participants = [Participant {
            role: ConflictRole::Issuer,
            address: other,
            commitment: shared_commitment.clone(),
        }];
        assert_eq!(
            find_resolver_conflict(&rotated, &shared_commitment, &participants),
            Some(ConflictRole::Issuer)
        );
        assert_eq!(
            find_resolver_conflict(&rotated, &bytes(&env, 6), &participants),
            None
        );
        assert_eq!(find_resolver_conflict(&rotated, &bytes(&env, 6), &[]), None);
    }

    #[test]
    fn default_commitment_is_stable_and_address_specific() {
        let env = Env::default();
        let a = Address::generate(&env);
        let b = Address::generate(&env);
        assert_eq!(
            default_participant_commitment(&env, &a),
            default_participant_commitment(&env, &a)
        );
        assert_ne!(
            default_participant_commitment(&env, &a),
            default_participant_commitment(&env, &b)
        );
    }
}
