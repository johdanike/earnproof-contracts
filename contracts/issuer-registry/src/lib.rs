#![no_std]

use earnproof_shared::{
    ContractError, GenesisRecord, GovernanceRole, GovernanceRoleAssignment, InterfaceVersion,
    IssuerAssuranceLevelCommitment, IssuerError, IssuerPolicyCommitments, IssuerQueryStatus,
    IssuerRecord, IssuerStatus, IssuerStatusResult, MigrationStatus, RoleAssignmentListResponse,
    RotationRecord, SigningKeyCommitment, SigningKeyLifecycleRecord, SigningKeyLifecycleTransition,
    TtlStatus, UpgradeApproval, UpgradeReceipt, ISSUER_ASSURANCE_COMMITMENT_VERSION,
    ISSUER_ASSURANCE_HISTORY_CAPACITY, ISSUER_REGISTRY_INTERFACE_VERSION,
    MAX_GOVERNANCE_ROLE_ASSIGNMENTS_PER_ROLE, MAX_GOVERNANCE_ROLE_ACTIVATION_DELAY_LEDGERS,
    MAX_GOVERNANCE_ROLES_PER_PRINCIPAL, MAX_ISSUER_ASSURANCE_HISTORY_PAGE,
    MAX_ISSUER_ASSURANCE_LEVEL, MAX_ISSUER_STATUS_BATCH, MAX_MIGRATION_BATCH,
    MAX_SIGNING_KEY_LIFECYCLE_PAGE, METADATA_REVISION_INITIAL, MIGRATION_STATUS_VERSION,
    SIGNING_KEY_LIFECYCLE_HISTORY_CAPACITY, TTL_EXTEND_TO_LEDGERS, TTL_THRESHOLD_LEDGERS,
    AdminTransferState, AdminTransferStatus, CompromisedSigningKeyRecord, ContractError,
    GenesisRecord, GovernanceAssignmentState, GovernanceRole, GovernanceRoleAssignment,
    InterfaceVersion, IssuerError, IssuerPolicyCommitments, IssuerQueryStatus, IssuerRecord,
    IssuerStatus, IssuerStatusResult, MigrationStatus, RotationRecord, SigningKeyCommitment,
    TtlStatus, UpgradeApproval, UpgradeReceipt, ADMIN_TRANSFER_EXPIRY_LEDGERS,
    ISSUER_REGISTRY_INTERFACE_VERSION, MAX_GOVERNANCE_ROLE_ACTIVATION_DELAY_LEDGERS,
    MAX_ISSUER_STATUS_BATCH, MAX_MIGRATION_BATCH, MAX_SIGNING_KEY_COMPROMISE_HISTORY_PAGE,
    METADATA_REVISION_INITIAL, MIGRATION_STATUS_VERSION, TTL_EXTEND_TO_LEDGERS,
    TTL_THRESHOLD_LEDGERS, UPGRADE_APPROVAL_EXPIRY_LEDGERS, UPGRADE_TIMELOCK_LEDGERS,
    ContractError, GenesisRecord, GovernanceRole, GovernanceRoleAssignment, InterfaceVersion,
    IssuerError, IssuerPolicyCommitments, IssuerQueryStatus, IssuerRecord, IssuerRevocationResult,
    IssuerRevocationState, IssuerRevocationTombstone, IssuerStatus, IssuerStatusResult,
    MigrationStatus, RotationRecord, SigningKeyCommitment, TtlStatus, UpgradeApproval,
    UpgradeReceipt, ISSUER_REGISTRY_INTERFACE_VERSION, ISSUER_REVOCATION_TOMBSTONE_VERSION,
    MAX_ISSUER_STATUS_BATCH, MAX_MIGRATION_BATCH, METADATA_REVISION_INITIAL,
    MIGRATION_STATUS_VERSION, TTL_EXTEND_TO_LEDGERS, TTL_THRESHOLD_LEDGERS,
    UPGRADE_APPROVAL_EXPIRY_LEDGERS, UPGRADE_TIMELOCK_LEDGERS,
};
use soroban_sdk::{contract, contractevent, contractimpl, contracttype, Address, BytesN, Env, Vec};

mod contract;
mod state;

const ISSUER_ROTATION_EXPIRY_LEDGERS: u32 = 518_400;

#[contract]
pub struct IssuerRegistryContract;

#[contracttype]
enum DataKey {
    Admin,
    Issuer(BytesN<32>),
    AddressIssuer(Address),
    IssuerTtl(BytesN<32>),
    AddressTtl(Address),
    InstanceLiveUntil,
    /// Allowlist entry: maps a WASM hash to the target contract version.
    AllowedWasm(BytesN<32>),
    MigrationStatus,
    /// Monotonically-increasing contract version.  Prevents downgrade.
    ContractVersion,
    Successor,
    Decommissioned,
    PendingAdmin,
    PendingAdminNominatedAt,
    PendingAdminExpiresAt,
    LatestUpgradeReceipt,
    /// Upgrade approval with temporal metadata (timelock and expiry).
    UpgradeApproval,
    /// Immutable deployment identity, written once at `initialize`.
    Genesis,
    ActiveSigningKey(BytesN<32>),
    PendingSigningKey(BytesN<32>),
    SigningKeyHistory(BytesN<32>, u32),
    SigningKeyHistoryCount(BytesN<32>),
    SigningKeyUsageSequence(BytesN<32>),
    SigningKeyLifecycleHistory(BytesN<32>, u32),
    SigningKeyLifecycleState(BytesN<32>, BytesN<32>, u32),
    CompromisedSigningKey(BytesN<32>, BytesN<32>, u32),
    CompromisedSigningKeyHistory(BytesN<32>, u32),
    CompromisedSigningKeyCount(BytesN<32>),
    RotationHistory(BytesN<32>, u32),
    RotationCount(BytesN<32>),
    ExecutedProposal(BytesN<32>),
    IssuerPolicy(BytesN<32>),
    IssuerAssurance(BytesN<32>),
    IssuerAssuranceHistory(BytesN<32>, u32),
    IssuerAssuranceSequence(BytesN<32>),
    /// Monotonic epoch, advanced once per externally visible issuer mutation.
    /// Off-chain consumers poll it as a cheap cache-invalidation signal.
    IssuerEpoch,
    /// Governed maximum number of issuers that may hold `Active` status at once.
    MaxActiveIssuers,
    /// Current number of issuers holding `Active` status.
    ActiveIssuerCount,
    /// Governed minimum ledger-time cooldown, in seconds, enforced before a
    /// suspended issuer may be reactivated.
    ReactivationCooldown,
    /// Per-issuer earliest ledger time at which reactivation is permitted.
    ReactivatableAt(BytesN<32>),
    PendingIssuerRotation(BytesN<32>),
    GovernanceAssignment(GovernanceRole, Address),
    GovernanceRoleDelay(GovernanceRole),
    /// Ordered index of the principals that hold one governance role, oldest
    /// grant first.
    RoleAssignmentIndex(GovernanceRole, u32),
    /// Number of principals indexed under one governance role.
    RoleAssignmentIndexCount(GovernanceRole),
    /// Ordered index of the governance roles one principal holds, oldest
    /// grant first.
    PrincipalRoleIndex(Address, u32),
    /// Number of governance roles indexed for one principal.
    PrincipalRoleIndexCount(Address),
    RevocationTombstone(BytesN<32>),
}

#[contractevent]
pub struct GovernanceRoleGranted {
    pub assignment: GovernanceRoleAssignment,
    pub granted_by: Address,
}

#[contractevent]
pub struct GovernanceRoleRemoved {
    pub role: GovernanceRole,
    pub address: Address,
    pub removed_by: Address,
}

#[contractevent]
pub struct GovernanceRoleDelayChanged {
    pub role: GovernanceRole,
    pub previous_delay_ledgers: u32,
    pub new_delay_ledgers: u32,
    pub changed_by: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingIssuerRotation {
    pub old_address: Address,
    pub new_address: Address,
    pub nominated_at_ledger: u32,
    pub expires_at_ledger: u32,
}

#[contractevent]
pub struct AdminTransferNominated {
    pub pending_admin: Address,
    pub nominated_by: Address,
}

#[contractevent]
pub struct AdminTransferAccepted {
    pub new_admin: Address,
}

#[contractevent]
pub struct AdminTransferCancelled {
    pub pending_admin: Address,
    pub cancelled_by: Address,
}

#[contractevent]
pub struct AdminTransferExpired {
    pub pending_admin: Address,
    pub expired_at_ledger: u32,
    pub cleaned_at_ledger: u32,
}

// ── upgrade events ────────────────────────────────────────────────────────────

/// Emitted when the admin adds a WASM hash to the upgrade allowlist.
#[contractevent]
pub struct UpgradeAllowlisted {
    pub proposal_id: BytesN<32>,
    pub wasm_hash: BytesN<32>,
    pub new_contract_version: u32,
    pub approved_by: Address,
}

/// Emitted when the admin removes a WASM hash from the allowlist without
/// applying it.
#[contractevent]
pub struct UpgradeRevoked {
    pub proposal_id: BytesN<32>,
    pub wasm_hash: BytesN<32>,
    pub revoked_by: Address,
}

/// Emitted when a WASM upgrade is successfully applied.
#[contractevent]
pub struct ContractUpgraded {
    pub new_wasm_hash: BytesN<32>,
    pub old_contract_version: u32,
    pub new_contract_version: u32,
    pub upgraded_by: Address,
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Emitted when an issuer is successfully registered.
///
/// Carries both metadata commitments: `metadata_hash` (content commitment) and
/// `metadata_uri_hash` (canonical-URI commitment). At registration the URI
/// commitment is the documented all-zero "no URI commitment recorded"
/// sentinel; a real URI commitment is set later via
/// `set_issuer_metadata_commitment`. `metadata_revision` starts at
/// [`METADATA_REVISION_INITIAL`].
/// `epoch` is the registry epoch after this mutation, so an indexer can order
/// lifecycle events and detect gaps without a separate read.
#[contractevent]
pub struct IssuerRegistered {
    pub issuer_id_hash: BytesN<32>,
    pub issuer_address: Address,
    pub metadata_hash: BytesN<32>,
    pub metadata_uri_hash: BytesN<32>,
    pub metadata_revision: u32,
    pub provenance_commitment: BytesN<32>,
    pub created_at: u64,
    pub epoch: u64,
}

/// Emitted when an issuer's metadata commitments are updated.
///
/// Carries both commitments and the post-update `metadata_revision` so
/// off-chain resolvers can distinguish a content change from a canonical-URI
/// change without re-reading storage.
#[contractevent]
pub struct IssuerMetadataUpdated {
    pub issuer_id_hash: BytesN<32>,
    pub metadata_hash: BytesN<32>,
    pub metadata_uri_hash: BytesN<32>,
    pub metadata_revision: u32,
    pub updated_at: u64,
    pub epoch: u64,
}

/// Emitted when an issuer is suspended.
///
/// `effective_ledger` and `effective_timestamp` record the ledger sequence and
/// timestamp at which the suspension became effective.
#[contractevent]
pub struct IssuerSuspended {
    pub issuer_id_hash: BytesN<32>,
    pub effective_ledger: u32,
    pub effective_timestamp: u64,
    pub reason_commitment: BytesN<32>,
    pub updated_at: u64,
    pub epoch: u64,
}

/// Emitted when a suspended issuer is reactivated.
///
/// `effective_ledger` and `effective_timestamp` record the ledger sequence and
/// timestamp at which the reactivation became effective.
#[contractevent]
pub struct IssuerReactivated {
    pub issuer_id_hash: BytesN<32>,
    pub effective_ledger: u32,
    pub effective_timestamp: u64,
    pub reason_commitment: BytesN<32>,
    pub updated_at: u64,
    pub epoch: u64,
}

/// Emitted when an issuer is permanently revoked.
///
/// `effective_ledger` and `effective_timestamp` record the ledger sequence and
/// timestamp at which the revocation became effective.
#[contractevent]
pub struct IssuerRevoked {
    pub issuer_id_hash: BytesN<32>,
    pub effective_ledger: u32,
    pub effective_timestamp: u64,
    pub reason_commitment: BytesN<32>,
    pub updated_at: u64,
    pub epoch: u64,
}

/// Emitted when an issuer's on-chain wallet address is rotated.
/// Both old and new addresses are included so indexers can update their mapping
/// without scanning storage.
#[contractevent]
pub struct IssuerAddressRotated {
    pub issuer_id_hash: BytesN<32>,
    pub old_address: Address,
    pub new_address: Address,
    pub updated_at: u64,
    pub epoch: u64,
}

#[contractevent]
pub struct IssuerRotationNominated {
    pub issuer_id_hash: BytesN<32>,
    pub old_address: Address,
    pub new_address: Address,
    pub expires_at_ledger: u32,
}

#[contractevent]
pub struct IssuerRotationCancelled {
    pub issuer_id_hash: BytesN<32>,
    pub cancelled_by: Address,
}

/// Emitted after a governed issuer assurance tier change commits.
#[contractevent]
pub struct IssuerAssuranceLevelChanged {
    pub issuer_id_hash: BytesN<32>,
    pub commitment_version: u32,
    pub assurance_level: u32,
    pub effective_ledger: u32,
    pub sequence: u32,
    pub changed_by: Address,
    pub epoch: u64,
}

#[contractevent]
pub struct SigningKeyLifecycleChanged {
    pub issuer_id: BytesN<32>,
    pub key_hash: BytesN<32>,
    pub algorithm: u32,
    pub transition: SigningKeyLifecycleTransition,
    pub sequence: u64,
    pub effective_ledger: u32,
/// Emitted once when an issuer signing-key commitment is declared compromised.
#[contractevent]
pub struct IssuerSigningKeyCompromised {
    pub record: CompromisedSigningKeyRecord,
    pub marked_by: Address,
}

// ── capacity and cooldown governance events ─────────────────────────────────

/// Emitted when the governed maximum active-issuer capacity is changed.
#[contractevent]
pub struct MaxActiveIssuersChanged {
    pub new_max: u32,
    pub active_count: u32,
    pub changed_by: Address,
}

/// Emitted when the governed reactivation cooldown is changed.
#[contractevent]
pub struct ReactivationCooldownChanged {
    pub new_cooldown_seconds: u64,
    pub changed_by: Address,
}

#[contractevent]
pub struct SuccessorNominated {
    pub proposal_id: BytesN<32>,
    pub successor: Address,
    pub nominated_by: Address,
}

#[contractevent]
pub struct ContractDecommissioned {
    pub proposal_id: BytesN<32>,
    pub old_instance: Address,
    pub successor_instance: Address,
    pub activated_by: Address,
}

// ---------------------------------------------------------------------------
// Contract implementation
// ---------------------------------------------------------------------------

#[contractimpl]
impl IssuerRegistryContract {
    pub fn grant_governance_role(
        env: Env,
        proposal_id: BytesN<32>,
        role: GovernanceRole,
        address: Address,
        activation_ledger: u32,
        expiration_ledger: Option<u32>,
    ) -> Result<(), ContractError> {
        let admin = Self::get_admin(env.clone()).map_err(|_| ContractError::NotInitialized)?;
        Self::require_auth(&admin);
        Self::require_valid_issuer_address(&address).map_err(|_| ContractError::InvalidAddress)?;
        let minimum_activation = env
            .ledger()
            .sequence()
            .checked_add(Self::get_role_activation_delay(env.clone(), role))
            .ok_or(ContractError::InvalidTimingConfig)?;
        let effective_activation = activation_ledger.max(minimum_activation);
        if role == GovernanceRole::Recovery
            || expiration_ledger
                .map(|expiration| expiration <= effective_activation)
                .unwrap_or(false)
        {
            return Err(ContractError::InvalidTimingConfig);
        }
        Self::consume_proposal(&env, &proposal_id).map_err(|_| ContractError::InvalidInput)?;
        // The index and the assignment are written in the same invocation, so
        // a grant that the index cannot hold leaves no assignment behind.
        state::index_assignment(&env, role, &address)?;
        let assignment = GovernanceRoleAssignment {
            role,
            address: address.clone(),
            activation_ledger: effective_activation,
            expiration_ledger,
            proposal_id,
        };
        let key = DataKey::GovernanceAssignment(role, address);
        env.storage().persistent().set(&key, &assignment);
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_THRESHOLD_LEDGERS, TTL_EXTEND_TO_LEDGERS);
        GovernanceRoleGranted {
            assignment,
            granted_by: admin,
        }
        .publish(&env);
        Ok(())
    }

    pub fn get_governance_assignment(
        env: Env,
        role: GovernanceRole,
        address: Address,
    ) -> Option<GovernanceRoleAssignment> {
        env.storage()
            .persistent()
            .get(&DataKey::GovernanceAssignment(role, address))
    }

    pub fn governance_role_state(
        env: Env,
        role: GovernanceRole,
        address: Address,
    ) -> GovernanceRoleState {
        let Some(assignment) = Self::get_governance_assignment(env.clone(), role, address) else {
            return GovernanceRoleState::Missing;
        };
        let ledger = env.ledger().sequence();
        if assignment.is_pending_at(ledger) {
            GovernanceRoleState::Pending
        } else if assignment.is_active_at(ledger) {
            GovernanceRoleState::Active
        } else {
            GovernanceRoleState::Expired
        }
    pub fn get_governance_assignment_state(
        env: Env,
        role: GovernanceRole,
        address: Address,
    ) -> Option<GovernanceAssignmentState> {
        let assignment = Self::get_governance_assignment(env.clone(), role, address)?;
        Some(assignment.state_at(env.ledger().sequence()))
    }

    /// Returns a bounded page of the assignments that currently hold `role`.
    ///
    /// The page is walked in grant order, oldest first, and `start_after`
    /// names the principal the page resumes after. The requested `limit` is
    /// clamped to the registry's maximum page size, so the work a page costs
    /// is bounded by the contract and not by the request.
    ///
    /// Only the administrator may enumerate. The listing names every address
    /// that holds the role, which is governance state rather than public
    /// data. Enumerating confers nothing: every operation that consults a
    /// role still consults the assignment itself.
    pub fn get_role_assignments(
        env: Env,
        role: GovernanceRole,
        start_after: Option<Address>,
        limit: Option<u32>,
    ) -> Result<RoleAssignmentListResponse, ContractError> {
        let admin = Self::get_admin(env.clone()).map_err(|_| ContractError::NotInitialized)?;
        Self::require_auth(&admin);
        contract::query_role_assignments(&env, role, start_after, limit)
    }

    /// Returns a bounded page of the roles `principal` currently holds.
    ///
    /// The page is walked in grant order, oldest first, and `start_after`
    /// names the role the page resumes after. The requested `limit` is
    /// clamped to the registry's maximum page size.
    pub fn get_principal_roles(
        env: Env,
        principal: Address,
        start_after: Option<GovernanceRole>,
        limit: Option<u32>,
    ) -> Result<RoleAssignmentListResponse, ContractError> {
        let admin = Self::get_admin(env.clone()).map_err(|_| ContractError::NotInitialized)?;
        Self::require_auth(&admin);
        contract::query_principal_roles(&env, &principal, start_after, limit)
    }

    /// Returns a bounded page of the most recently granted assignments for
    /// `role`.
    ///
    /// The page is walked in reverse grant order, newest first, so an
    /// administrator sees the latest grants rather than the oldest. A grant
    /// whose activation ledger is still in the future is included: it has
    /// been made but is not in force yet, and hiding it would hide a pending
    /// grant. An expired assignment is not.
    pub fn get_recent_role_assignments(
        env: Env,
        role: GovernanceRole,
        start_after: Option<Address>,
        limit: Option<u32>,
    ) -> Result<RoleAssignmentListResponse, ContractError> {
        let admin = Self::get_admin(env.clone()).map_err(|_| ContractError::NotInitialized)?;
        Self::require_auth(&admin);
        contract::query_recent_role_assignments(&env, role, start_after, limit)
    }

    /// Drops the assignments of `role` that have expired from the governance
    /// indexes.
    ///
    /// The assignment records are left in place: an expired assignment is a
    /// historical fact, and `governance_role_state` keeps reporting it as
    /// expired. What the sweep removes is the index entry, which is what
    /// occupies a slot in a bounded list.
    ///
    /// Returns the number of index entries dropped.
    pub fn expire_role_assignments(env: Env, role: GovernanceRole) -> Result<u32, ContractError> {
        let admin = Self::get_admin(env.clone()).map_err(|_| ContractError::NotInitialized)?;
        Self::require_auth(&admin);
        let ledger = env.ledger().sequence();
        let mut expired = 0u32;
        let mut slot = 0u32;
        // The index shrinks as entries are dropped, so the bound is read on
        // every pass and the cursor stays on the entry that took the place
        // of the one just removed.
        while slot < state::role_index_count(&env, role) {
            let Some(principal) = state::role_index_entry(&env, role, slot) else {
                slot += 1;
                continue;
            };
            let assignment: Option<GovernanceRoleAssignment> = env
                .storage()
                .persistent()
                .get(&DataKey::GovernanceAssignment(role, principal.clone()));
            let lapsed = match assignment {
                Some(record) => record
                    .expiration_ledger
                    .map(|expiry| ledger >= expiry)
                    .unwrap_or(false),
                // An index entry without a record cannot be acted on, so it is
                // dropped along with the expired ones.
                None => true,
            };
            if lapsed {
                state::unindex_assignment(&env, role, &principal);
                expired += 1;
                continue;
            }
            slot += 1;
        }
        Ok(expired)
    }

    /// Adds an existing assignment to the governance indexes.
    ///
    /// An assignment granted before the indexes existed is invisible to
    /// enumeration until it is indexed. This brings one such assignment into
    /// the index without regranting it: the activation ledger, the expiry,
    /// and the proposal that produced it are left exactly as they are.
    ///
    /// Returns `true` when the assignment was indexed by this call, and
    /// `false` when there was nothing to migrate or the assignment was
    /// already indexed.
    pub fn migrate_role_assignment(
        env: Env,
        role: GovernanceRole,
        address: Address,
    ) -> Result<bool, ContractError> {
        let admin = Self::get_admin(env.clone()).map_err(|_| ContractError::NotInitialized)?;
        Self::require_auth(&admin);
        let key = DataKey::GovernanceAssignment(role, address.clone());
        if !env.storage().persistent().has(&key) {
            return Ok(false);
        }
        if state::role_index_slot(&env, role, &address).is_some() {
            return Ok(false);
        }
        state::index_assignment(&env, role, &address)?;
        Ok(true)
    }

    /// Sets the minimum delay for future grants of a role. Existing
    /// assignments retain their snapshotted activation ledger.
    pub fn set_role_activation_delay(
        env: Env,
        role: GovernanceRole,
        delay_ledgers: u32,
    ) -> Result<(), ContractError> {
        let admin = Self::get_admin(env.clone()).map_err(|_| ContractError::NotInitialized)?;
        Self::require_auth(&admin);
        if role == GovernanceRole::Recovery
            || delay_ledgers > MAX_GOVERNANCE_ROLE_ACTIVATION_DELAY_LEDGERS
        {
            return Err(ContractError::InvalidTimingConfig);
        }
        let key = DataKey::GovernanceRoleDelay(role);
        let previous_delay_ledgers = env.storage().persistent().get(&key).unwrap_or(0);
        env.storage().persistent().set(&key, &delay_ledgers);
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_THRESHOLD_LEDGERS, TTL_EXTEND_TO_LEDGERS);
        GovernanceRoleDelayChanged {
            role,
            previous_delay_ledgers,
            new_delay_ledgers: delay_ledgers,
            changed_by: admin,
        }
        .publish(&env);
        Ok(())
    }

    pub fn get_role_activation_delay(env: Env, role: GovernanceRole) -> u32 {
        let key = DataKey::GovernanceRoleDelay(role);
        let delay = env.storage().persistent().get(&key);
        if delay.is_some() {
            env.storage().persistent().extend_ttl(
                &key,
                TTL_THRESHOLD_LEDGERS,
                TTL_EXTEND_TO_LEDGERS,
            );
        }
        delay.unwrap_or(0)
    }

    pub fn remove_governance_role(
        env: Env,
        role: GovernanceRole,
        address: Address,
    ) -> Result<(), ContractError> {
        let admin = Self::get_admin(env.clone()).map_err(|_| ContractError::NotInitialized)?;
        Self::require_auth(&admin);
        let key = DataKey::GovernanceAssignment(role, address.clone());
        if !env.storage().persistent().has(&key) {
            return Err(ContractError::NotFound);
        }
        env.storage().persistent().remove(&key);
        // The index entry goes with the assignment: a removed holder must not
        // keep occupying a slot in a bounded list.
        state::unindex_assignment(&env, role, &address);
        GovernanceRoleRemoved {
            role,
            address,
            removed_by: admin,
        }
        .publish(&env);
        Ok(())
    }

    pub fn suspend_issuer_by_role(
        env: Env,
        proposal_id: BytesN<32>,
        issuer_id_hash: BytesN<32>,
        reason_commitment: BytesN<32>,
        actor: Address,
    ) -> Result<(), IssuerError> {
        Self::set_status_as(
            env,
            proposal_id,
            issuer_id_hash,
            IssuerStatus::Suspended,
            reason_commitment,
            Some(actor),
        )
    }

    pub fn is_decommissioned(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Decommissioned)
            .unwrap_or(false)
    }

    fn ensure_not_decommissioned(env: &Env) -> Result<(), IssuerError> {
        if Self::is_decommissioned(env.clone()) {
            Err(IssuerError::InvalidTransition)
        } else {
            Ok(())
        }
    }

    pub fn is_proposal_executed(env: Env, proposal_id: BytesN<32>) -> bool {
        if proposal_id == BytesN::from_array(&env, &[0u8; 32]) {
            return false;
        }
        let domain_key = earnproof_shared::proposal_domain_key(
            &env,
            soroban_sdk::Symbol::new(&env, "issuer_registry"),
            &proposal_id,
        );
        env.storage()
            .persistent()
            .has(&DataKey::ExecutedProposal(domain_key))
    }

    fn consume_proposal(env: &Env, proposal_id: &BytesN<32>) -> Result<(), ContractError> {
        if proposal_id == &BytesN::from_array(env, &[0u8; 32]) {
            return Err(ContractError::InvalidInput);
        }
        let domain_key = earnproof_shared::proposal_domain_key(
            env,
            soroban_sdk::Symbol::new(env, "issuer_registry"),
            proposal_id,
        );
        let key = DataKey::ExecutedProposal(domain_key);
        if env.storage().persistent().has(&key) {
            return Err(ContractError::AlreadyExists);
        }
        env.storage().persistent().set(&key, &true);
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_THRESHOLD_LEDGERS, TTL_EXTEND_TO_LEDGERS);
        Ok(())
    }

    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(ContractError::AlreadyInitialized);
        }

        Self::require_valid_admin(&admin)?;
        Self::require_auth(&admin);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::ContractVersion, &1_u32);
        let genesis = GenesisRecord {
            genesis_id: earnproof_shared::compute_genesis_id(&env, "earnproof_issuer_registry"),
            initialized_at_ledger: env.ledger().sequence(),
        };
        env.storage().instance().set(&DataKey::Genesis, &genesis);
        // Deterministic starting state for the epoch, capacity, and cooldown
        // features. Capacity defaults to unlimited so pre-existing behaviour is
        // preserved until an admin sets a real bound.
        env.storage().instance().set(&DataKey::IssuerEpoch, &0_u64);
        env.storage()
            .instance()
            .set(&DataKey::MaxActiveIssuers, &u32::MAX);
        env.storage()
            .instance()
            .set(&DataKey::ActiveIssuerCount, &0_u32);
        env.storage()
            .instance()
            .set(&DataKey::ReactivationCooldown, &0_u64);
        Self::extend_instance_ttl(env);
        Ok(())
    }

    /// Initializes the epoch, capacity, and cooldown state on a contract that
    /// was deployed before these features existed.
    ///
    /// Idempotent for the keys that have a natural default (epoch, capacity
    /// limit, cooldown): they are only written when absent. The active-issuer
    /// count cannot be derived on-chain, so the caller supplies the known count
    /// once; it is written unconditionally. Admin-only.
    pub fn migrate(env: Env, active_issuer_count: u32) -> Result<(), IssuerError> {
        let admin = Self::get_admin(env.clone()).map_err(|_| IssuerError::IssuerNotFound)?;
        Self::require_auth(&admin);

        if !env.storage().instance().has(&DataKey::IssuerEpoch) {
            env.storage().instance().set(&DataKey::IssuerEpoch, &0_u64);
        }
        if !env.storage().instance().has(&DataKey::MaxActiveIssuers) {
            env.storage()
                .instance()
                .set(&DataKey::MaxActiveIssuers, &u32::MAX);
        }
        if !env.storage().instance().has(&DataKey::ReactivationCooldown) {
            env.storage()
                .instance()
                .set(&DataKey::ReactivationCooldown, &0_u64);
        }
        env.storage()
            .instance()
            .set(&DataKey::ActiveIssuerCount, &active_issuer_count);
        Self::extend_instance_ttl(env);
        Ok(())
    }

    /// Returns the immutable genesis identity recorded at `initialize`.
    /// Unchanged across upgrades and storage migrations.
    pub fn get_genesis(env: Env) -> Result<GenesisRecord, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::Genesis)
            .ok_or(ContractError::NotInitialized)
    }

    pub fn nominate_admin(env: Env, new_admin: Address) -> Result<(), ContractError> {
        Self::ensure_not_decommissioned(&env).map_err(|_| ContractError::InvalidState)?;
        let admin = Self::get_admin(env.clone())?;
        Self::require_valid_admin(&new_admin)?;
        Self::require_auth(&admin);

        let nominated_at = env.ledger().sequence();
        let expires_at = nominated_at
            .checked_add(ADMIN_TRANSFER_EXPIRY_LEDGERS)
            .ok_or(ContractError::InvalidTimingConfig)?;
        env.storage()
            .instance()
            .set(&DataKey::PendingAdmin, &new_admin);
        env.storage()
            .instance()
            .set(&DataKey::PendingAdminNominatedAt, &nominated_at);
        env.storage()
            .instance()
            .set(&DataKey::PendingAdminExpiresAt, &expires_at);
        AdminTransferNominated {
            pending_admin: new_admin.clone(),
            nominated_by: admin,
        }
        .publish(&env);
        Ok(())
    }

    pub fn accept_admin(env: Env) -> Result<(), ContractError> {
        Self::ensure_not_decommissioned(&env).map_err(|_| ContractError::InvalidState)?;
        let status = Self::get_admin_transfer_status(env.clone());
        if status.state == AdminTransferState::Expired {
            return Err(ContractError::AdminNominationExpired);
        }
        let pending_admin = status.pending_admin.ok_or(ContractError::NotFound)?;
        Self::require_auth(&pending_admin);

        env.storage()
            .instance()
            .set(&DataKey::Admin, &pending_admin);
        Self::clear_admin_nomination(&env);

        AdminTransferAccepted {
            new_admin: pending_admin,
        }
        .publish(&env);
        Ok(())
    }

    pub fn cancel_admin_transfer(env: Env) -> Result<(), ContractError> {
        Self::ensure_not_decommissioned(&env).map_err(|_| ContractError::InvalidState)?;
        let admin = Self::get_admin(env.clone())?;
        Self::require_auth(&admin);

        let pending_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::PendingAdmin)
            .ok_or(ContractError::NotFound)?;
        Self::clear_admin_nomination(&env);

        AdminTransferCancelled {
            pending_admin,
            cancelled_by: admin,
        }
        .publish(&env);
        Ok(())
    }

    pub fn get_admin_transfer_status(env: Env) -> AdminTransferStatus {
        let pending_admin: Option<Address> = env.storage().instance().get(&DataKey::PendingAdmin);
        let Some(pending_admin) = pending_admin else {
            return AdminTransferStatus {
                state: AdminTransferState::Absent,
                pending_admin: None,
                nominated_at_ledger: 0,
                expires_at_ledger: 0,
            };
        };
        let nominated_at_ledger = env
            .storage()
            .instance()
            .get(&DataKey::PendingAdminNominatedAt)
            .unwrap_or(0);
        let expires_at_ledger = env
            .storage()
            .instance()
            .get(&DataKey::PendingAdminExpiresAt)
            .unwrap_or(0);
        let state = if expires_at_ledger == 0 || env.ledger().sequence() >= expires_at_ledger {
            AdminTransferState::Expired
        } else {
            AdminTransferState::Pending
        };
        AdminTransferStatus {
            state,
            pending_admin: Some(pending_admin),
            nominated_at_ledger,
            expires_at_ledger,
        }
    }

    pub fn cleanup_expired_admin_transfer(env: Env) -> Result<(), ContractError> {
        let status = Self::get_admin_transfer_status(env.clone());
        match status.state {
            AdminTransferState::Absent => return Err(ContractError::NotFound),
            AdminTransferState::Pending => return Err(ContractError::InvalidState),
            AdminTransferState::Expired => {}
        }
        let pending_admin = status.pending_admin.ok_or(ContractError::NotFound)?;
        Self::clear_admin_nomination(&env);
        AdminTransferExpired {
            pending_admin,
            expired_at_ledger: status.expires_at_ledger,
            cleaned_at_ledger: env.ledger().sequence(),
        }
        .publish(&env);
        Ok(())
    }

    fn clear_admin_nomination(env: &Env) {
        env.storage().instance().remove(&DataKey::PendingAdmin);
        env.storage()
            .instance()
            .remove(&DataKey::PendingAdminNominatedAt);
        env.storage()
            .instance()
            .remove(&DataKey::PendingAdminExpiresAt);
    }

    pub fn get_admin(env: Env) -> Result<Address, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ContractError::NotInitialized)
    }

    pub fn nominate_successor(
        env: Env,
        proposal_id: BytesN<32>,
        successor: Address,
    ) -> Result<(), IssuerError> {
        Self::ensure_not_decommissioned(&env)?;
        let admin = Self::get_admin(env.clone()).map_err(|_| IssuerError::IssuerNotFound)?;
        Self::require_valid_issuer_address(&successor)?;
        Self::require_auth(&admin);
        Self::consume_proposal(&env, &proposal_id).map_err(|_| IssuerError::InvalidTransition)?;
        env.storage()
            .instance()
            .set(&DataKey::Successor, &successor);
        SuccessorNominated {
            proposal_id,
            successor: successor.clone(),
            nominated_by: admin,
        }
        .publish(&env);
        Ok(())
    }

    pub fn get_successor(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Successor)
    }

    pub fn activate_successor(env: Env, proposal_id: BytesN<32>) -> Result<(), IssuerError> {
        Self::ensure_not_decommissioned(&env)?;
        let admin = Self::get_admin(env.clone()).map_err(|_| IssuerError::IssuerNotFound)?;
        Self::require_auth(&admin);
        let successor: Address = env
            .storage()
            .instance()
            .get(&DataKey::Successor)
            .ok_or(IssuerError::IssuerNotFound)?;

        Self::consume_proposal(&env, &proposal_id).map_err(|_| IssuerError::InvalidTransition)?;
        env.storage()
            .instance()
            .set(&DataKey::Decommissioned, &true);
        ContractDecommissioned {
            proposal_id,
            old_instance: env.current_contract_address(),
            successor_instance: successor,
            activated_by: admin,
        }
        .publish(&env);
        Ok(())
    }

    pub fn keepalive_instance(env: Env) -> bool {
        if !env.storage().instance().has(&DataKey::Admin) {
            return false;
        }
        Self::extend_instance_ttl(env);
        true
    }

    pub fn keepalive_issuer(env: Env, issuer_id_hash: BytesN<32>) -> bool {
        let key = DataKey::Issuer(issuer_id_hash);
        if env.storage().persistent().has(&key) {
            Self::extend_issuer_key_ttl(env, &key);
            true
        } else {
            false
        }
    }

    pub fn keepalive_address_issuer(env: Env, issuer_address: Address) -> bool {
        let key = DataKey::AddressIssuer(issuer_address);
        if env.storage().persistent().has(&key) {
            env.storage().persistent().extend_ttl(
                &key,
                TTL_THRESHOLD_LEDGERS,
                TTL_EXTEND_TO_LEDGERS,
            );
            true
        } else {
            false
        }
    }

    pub fn register_issuer(
        env: Env,
        issuer_id_hash: BytesN<32>,
        issuer_address: Address,
        metadata_hash: BytesN<32>,
        provenance_commitment: BytesN<32>,
    ) -> Result<(), IssuerError> {
        Self::assert_operational(&env);
        if provenance_commitment == BytesN::from_array(&env, &[0u8; 32]) {
            return Err(IssuerError::InvalidAddress);
        }
        let admin = Self::get_admin(env.clone()).map_err(|_| IssuerError::IssuerNotFound)?;
        Self::require_valid_issuer_address(&issuer_address)?;
        Self::require_auth(&admin);

        let key = DataKey::Issuer(issuer_id_hash.clone());
        if env
            .storage()
            .persistent()
            .has(&DataKey::RevocationTombstone(issuer_id_hash.clone()))
        {
            return Err(IssuerError::IssuerRevoked);
        }
        if env.storage().persistent().has(&key) {
            return Err(IssuerError::IssuerAlreadyRegistered);
        }

        let address_key = DataKey::AddressIssuer(issuer_address.clone());
        if env.storage().persistent().has(&address_key) {
            return Err(IssuerError::IssuerAddressAlreadyRegistered);
        }

        // Status and its effective ledger metadata are written together in a
        // single persistent `set`, so the record's status is never stored
        // without the ledger/timestamp at which it became effective. Timing is
        // sourced only from the host ledger environment. The URI commitment
        // starts at the all-zero sentinel until an explicit commitment is set.
        // A new issuer starts Active, so it consumes one capacity slot. This is
        // checked and reserved before any state is written, so a rejected
        // registration mutates nothing.
        Self::reserve_active_capacity(&env)?;

        let now = env.ledger().timestamp();
        let effective_ledger = env.ledger().sequence();
        let metadata_uri_hash = Self::zero_hash(&env);
        let record = IssuerRecord {
            issuer_id_hash: issuer_id_hash.clone(),
            issuer_address: issuer_address.clone(),
            metadata_hash: metadata_hash.clone(),
            metadata_uri_hash: metadata_uri_hash.clone(),
            metadata_revision: METADATA_REVISION_INITIAL,
            provenance_commitment: provenance_commitment.clone(),
            status: IssuerStatus::Active,
            created_at: now,
            updated_at: now,
            status_effective_ledger: effective_ledger,
            status_effective_timestamp: now,
            reason_commitment: None,
        };

        env.storage().persistent().set(&key, &record);
        env.storage()
            .persistent()
            .set(&address_key, &issuer_id_hash);
        Self::extend_issuer_ttl(env.clone(), issuer_id_hash.clone());
        Self::extend_address_ttl(env.clone(), issuer_address.clone());

        let epoch = Self::bump_epoch(&env);
        IssuerRegistered {
            issuer_id_hash,
            issuer_address,
            metadata_hash,
            metadata_uri_hash,
            metadata_revision: METADATA_REVISION_INITIAL,
            provenance_commitment,
            created_at: now,
            epoch,
        }
        .publish(&env);
        Ok(())
    }

    pub fn get_provenance_commitment(
        env: Env,
        issuer_id_hash: BytesN<32>,
    ) -> Result<BytesN<32>, IssuerError> {
        let record = Self::get_issuer(env, issuer_id_hash)?;
        Ok(record.provenance_commitment)
    }

    pub fn update_issuer(
        env: Env,
        issuer_id_hash: BytesN<32>,
        metadata_hash: BytesN<32>,
    ) -> Result<(), IssuerError> {
        Self::assert_operational(&env);
        let admin = Self::get_admin(env.clone()).map_err(|_| IssuerError::IssuerNotFound)?;
        Self::require_auth(&admin);

        let key = DataKey::Issuer(issuer_id_hash.clone());
        let mut record: IssuerRecord = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(IssuerError::IssuerNotFound)?;

        if record.status == IssuerStatus::Revoked {
            return Err(IssuerError::IssuerRevoked);
        }

        let now = env.ledger().timestamp();
        record.metadata_hash = metadata_hash.clone();
        record.metadata_revision = record.metadata_revision.saturating_add(1);
        record.updated_at = now;
        let metadata_uri_hash = record.metadata_uri_hash.clone();
        let metadata_revision = record.metadata_revision;
        env.storage().persistent().set(&key, &record);
        Self::extend_issuer_key_ttl(env.clone(), &key);

        let epoch = Self::bump_epoch(&env);
        IssuerMetadataUpdated {
            issuer_id_hash,
            metadata_hash,
            metadata_uri_hash,
            metadata_revision,
            updated_at: now,
            epoch,
        }
        .publish(&env);
        Ok(())
    }

    /// Update both metadata commitments (content hash and canonical-URI hash)
    /// for an issuer in one atomic operation, incrementing the metadata
    /// revision.
    ///
    /// # Commitment rules
    ///
    /// Both commitments are opaque 32-byte digests computed off chain. No raw
    /// URI or private metadata is ever stored on chain. A commitment must be
    /// non-empty: the all-zero digest is rejected as
    /// [`IssuerError::InvalidMetadataCommitment`], since it is reserved as the
    /// "no URI commitment recorded" sentinel and is not a value any real
    /// SHA-256 digest collides with in practice.
    ///
    /// # Canonical bytes and domain separation
    ///
    /// A backend must compute the commitments with domain separation so a
    /// content digest can never be confused with a URI digest:
    ///
    /// - content:  `metadata_hash    = SHA-256("earnproof:issuer-metadata:v1"    || canonical_document_bytes)`
    /// - location: `metadata_uri_hash = SHA-256("earnproof:issuer-metadata-uri:v1" || uri_utf8_bytes)`
    ///
    /// The contract treats the resulting values as opaque `BytesN<32>` and
    /// stores and echoes them byte-for-byte; the golden vectors in the test
    /// suite pin this encoding parity between backend and contract.
    pub fn set_issuer_metadata_commitment(
        env: Env,
        issuer_id_hash: BytesN<32>,
        metadata_hash: BytesN<32>,
        metadata_uri_hash: BytesN<32>,
    ) -> Result<(), IssuerError> {
        Self::assert_operational(&env);
        let admin = Self::get_admin(env.clone()).map_err(|_| IssuerError::IssuerNotFound)?;
        Self::require_auth(&admin);

        if Self::is_zero_hash(&env, &metadata_hash) || Self::is_zero_hash(&env, &metadata_uri_hash)
        {
            return Err(IssuerError::InvalidMetadataCommitment);
        }

        let key = DataKey::Issuer(issuer_id_hash.clone());
        let mut record: IssuerRecord = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(IssuerError::IssuerNotFound)?;

        if record.status == IssuerStatus::Revoked {
            return Err(IssuerError::IssuerRevoked);
        }

        let now = env.ledger().timestamp();
        record.metadata_hash = metadata_hash.clone();
        record.metadata_uri_hash = metadata_uri_hash.clone();
        record.metadata_revision = record.metadata_revision.saturating_add(1);
        record.updated_at = now;
        let metadata_revision = record.metadata_revision;
        env.storage().persistent().set(&key, &record);
        Self::extend_issuer_key_ttl(env.clone(), &key);

        let epoch = Self::bump_epoch(&env);
        IssuerMetadataUpdated {
            issuer_id_hash,
            metadata_hash,
            metadata_uri_hash,
            metadata_revision,
            updated_at: now,
            epoch,
        }
        .publish(&env);
        Ok(())
    }

    /// Begins a governed two-step signing-key rotation. The pending commitment
    /// is not active until `activate_issuer_signing_key` commits it.
    pub fn propose_issuer_signing_key(
        env: Env,
        issuer_id: BytesN<32>,
        key_hash: BytesN<32>,
        algorithm: u32,
    ) -> Result<(), IssuerError> {
        Self::assert_operational(&env);
        let admin = Self::get_admin(env.clone()).map_err(|_| IssuerError::IssuerNotFound)?;
        Self::require_auth(&admin);
        if algorithm == 0 || Self::is_zero_hash(&env, &key_hash) {
            return Err(IssuerError::InvalidMetadataCommitment);
        }
        Self::get_issuer(env.clone(), issuer_id.clone())?;
        let pending_key = DataKey::PendingSigningKey(issuer_id.clone());
        if env.storage().persistent().has(&pending_key)
            || env
                .storage()
                .persistent()
                .get::<_, SigningKeyCommitment>(&DataKey::ActiveSigningKey(issuer_id.clone()))
                .map(|active| active.key_hash == key_hash && active.algorithm == algorithm)
                .unwrap_or(false)
            || Self::stored_signing_key_state(
                env.clone(),
                issuer_id.clone(),
                key_hash.clone(),
        if env
            .storage()
            .persistent()
            .has(&DataKey::CompromisedSigningKey(
                issuer_id.clone(),
                key_hash.clone(),
                algorithm,
            ))
        {
            return Err(IssuerError::InvalidTransition);
        }
        env.storage().persistent().set(
            &DataKey::PendingSigningKey(issuer_id),
            &SigningKeyCommitment {
                key_hash,
                algorithm,
            )
            .map(|state| state.transition == SigningKeyLifecycleTransition::Compromised)
            .unwrap_or(false)
        {
            return Err(IssuerError::InvalidTransition);
        }
        let sequence = Self::checked_signing_key_sequence(env.clone(), issuer_id.clone(), 1)?;
        let commitment = SigningKeyCommitment {
            key_hash: key_hash.clone(),
            algorithm,
            activated_ledger: 0,
        };
        env.storage().persistent().set(&pending_key, &commitment);
        Self::append_signing_key_transition(
            env.clone(),
            issuer_id.clone(),
            key_hash,
            algorithm,
            SigningKeyLifecycleTransition::Proposed,
            sequence,
        );
        Self::store_signing_key_sequence(env, issuer_id, sequence);
        Ok(())
    }

    /// Activates a previously proposed key once. The pending entry is removed,
    /// so a retired commitment cannot be replayed into the active position.
    pub fn activate_issuer_signing_key(env: Env, issuer_id: BytesN<32>) -> Result<(), IssuerError> {
        Self::assert_operational(&env);
        let admin = Self::get_admin(env.clone()).map_err(|_| IssuerError::IssuerNotFound)?;
        Self::require_auth(&admin);
        let pending_key = DataKey::PendingSigningKey(issuer_id.clone());
        let mut next: SigningKeyCommitment = env
            .storage()
            .persistent()
            .get(&pending_key)
            .ok_or(IssuerError::InvalidTransition)?;
        if env
            .storage()
            .persistent()
            .has(&DataKey::CompromisedSigningKey(
                issuer_id.clone(),
                next.key_hash.clone(),
                next.algorithm,
            ))
        {
            return Err(IssuerError::InvalidTransition);
        }
        next.activated_ledger = env.ledger().sequence();
        let active_key = DataKey::ActiveSigningKey(issuer_id.clone());
        let previous = env
            .storage()
            .persistent()
            .get::<_, SigningKeyCommitment>(&active_key);
        let steps = if previous.is_some() { 2 } else { 1 };
        let final_sequence =
            Self::checked_signing_key_sequence(env.clone(), issuer_id.clone(), steps)?;
        let mut sequence = final_sequence - u64::from(steps) + 1;
        if let Some(previous) = previous {
            let count: u32 = env
                .storage()
                .persistent()
                .get(&DataKey::SigningKeyHistoryCount(issuer_id.clone()))
                .unwrap_or(0);
            if count < 4 {
                env.storage().persistent().set(
                    &DataKey::SigningKeyHistory(issuer_id.clone(), count),
                    &previous,
                );
                env.storage().persistent().set(
                    &DataKey::SigningKeyHistoryCount(issuer_id.clone()),
                    &(count + 1),
                );
            }
            Self::append_signing_key_transition(
                env.clone(),
                issuer_id.clone(),
                previous.key_hash,
                previous.algorithm,
                SigningKeyLifecycleTransition::Retired,
                sequence,
            );
            sequence += 1;
        }
        env.storage().persistent().set(&active_key, &next);
        env.storage().persistent().remove(&pending_key);
        Self::append_signing_key_transition(
            env.clone(),
            issuer_id.clone(),
            next.key_hash,
            next.algorithm,
            SigningKeyLifecycleTransition::Activated,
            sequence,
        );
        Self::store_signing_key_sequence(env, issuer_id, final_sequence);
        Ok(())
    }

    pub fn get_active_issuer_signing_key(
        env: Env,
        issuer_id: BytesN<32>,
    ) -> Option<SigningKeyCommitment> {
        env.storage()
            .persistent()
            .get(&DataKey::ActiveSigningKey(issuer_id))
    }
    pub fn get_prior_issuer_signing_keys(
        env: Env,
        issuer_id: BytesN<32>,
    ) -> Vec<SigningKeyCommitment> {
        let mut result = Vec::new(&env);
        let count: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::SigningKeyHistoryCount(issuer_id.clone()))
            .unwrap_or(0);
        for index in 0..count {
            if let Some(item) = env
                .storage()
                .persistent()
                .get(&DataKey::SigningKeyHistory(issuer_id.clone(), index))
            {
                result.push_back(item);
            }
        }
        result
    }

    pub fn retire_issuer_signing_key(
        env: Env,
        issuer_id: BytesN<32>,
    ) -> Result<SigningKeyLifecycleRecord, IssuerError> {
        Self::assert_operational(&env);
        let admin = Self::get_admin(env.clone()).map_err(|_| IssuerError::IssuerNotFound)?;
        Self::require_auth(&admin);
        Self::get_issuer(env.clone(), issuer_id.clone())?;
        let active_key = DataKey::ActiveSigningKey(issuer_id.clone());
        let active: SigningKeyCommitment = env
            .storage()
            .persistent()
            .get(&active_key)
            .ok_or(IssuerError::InvalidTransition)?;
        let sequence = Self::checked_signing_key_sequence(env.clone(), issuer_id.clone(), 1)?;
        let count_key = DataKey::SigningKeyHistoryCount(issuer_id.clone());
        let count: u32 = env.storage().persistent().get(&count_key).unwrap_or(0);
        if count < 4 {
            env.storage().persistent().set(
                &DataKey::SigningKeyHistory(issuer_id.clone(), count),
                &active,
            );
            env.storage().persistent().set(&count_key, &(count + 1));
        }
        env.storage().persistent().remove(&active_key);
        let record = Self::append_signing_key_transition(
            env.clone(),
            issuer_id.clone(),
            active.key_hash,
            active.algorithm,
            SigningKeyLifecycleTransition::Retired,
            sequence,
        );
        Self::store_signing_key_sequence(env, issuer_id, sequence);
        Ok(record)
    }

    /// Records a terminal compromise transition for a known key commitment.
    /// No raw key material or free-form incident data is accepted.
    /// Permanently marks a known active, pending, or retired signing-key
    /// commitment as compromised. The immutable sidecar is deliberately
    /// separate from legacy signing-key records to preserve their encoding.
    pub fn mark_signing_key_compromised(
        env: Env,
        issuer_id: BytesN<32>,
        key_hash: BytesN<32>,
        algorithm: u32,
    ) -> Result<SigningKeyLifecycleRecord, IssuerError> {
        Self::assert_operational(&env);
        let admin = Self::get_admin(env.clone()).map_err(|_| IssuerError::IssuerNotFound)?;
        Self::require_auth(&admin);
        Self::get_issuer(env.clone(), issuer_id.clone())?;
        if algorithm == 0 || Self::is_zero_hash(&env, &key_hash) {
            return Err(IssuerError::InvalidMetadataCommitment);
        }
        if Self::stored_signing_key_state(
            env.clone(),
            issuer_id.clone(),
            key_hash.clone(),
            algorithm,
        )
        .map(|state| state.transition == SigningKeyLifecycleTransition::Compromised)
        .unwrap_or(false)
        {
            return Err(IssuerError::InvalidTransition);
        }

        let pending_key = DataKey::PendingSigningKey(issuer_id.clone());
        let active_key = DataKey::ActiveSigningKey(issuer_id.clone());
        let pending_match = env
            .storage()
            .persistent()
            .get::<_, SigningKeyCommitment>(&pending_key)
            .map(|pending| pending.key_hash == key_hash && pending.algorithm == algorithm)
            .unwrap_or(false);
        let active_match = env
            .storage()
            .persistent()
            .get::<_, SigningKeyCommitment>(&active_key)
            .map(|active| active.key_hash == key_hash && active.algorithm == algorithm)
            .unwrap_or(false);
        let prior_match = Self::get_prior_issuer_signing_keys(env.clone(), issuer_id.clone())
            .iter()
            .any(|prior| prior.key_hash == key_hash && prior.algorithm == algorithm);
        if !pending_match && !active_match && !prior_match {
            return Err(IssuerError::InvalidTransition);
        }
        let sequence = Self::checked_signing_key_sequence(env.clone(), issuer_id.clone(), 1)?;
        if pending_match {
            env.storage().persistent().remove(&pending_key);
        }
        if active_match {
            env.storage().persistent().remove(&active_key);
        }
        let record = Self::append_signing_key_transition(
            env.clone(),
            issuer_id.clone(),
            key_hash,
            algorithm,
            SigningKeyLifecycleTransition::Compromised,
            sequence,
        );
        Self::store_signing_key_sequence(env, issuer_id, sequence);
        Ok(record)
    }

    pub fn get_signing_key_usage_sequence(env: Env, issuer_id: BytesN<32>) -> u64 {
        let key = DataKey::SigningKeyUsageSequence(issuer_id);
        let sequence = env.storage().persistent().get(&key).unwrap_or(0);
        if env.storage().persistent().has(&key) {
            Self::extend_issuer_key_ttl(env, &key);
        }
        sequence
    }

    pub fn get_issuer_signing_key_state(
        reason_hash: BytesN<32>,
    ) -> Result<CompromisedSigningKeyRecord, IssuerError> {
        Self::assert_operational(&env);
        let admin = Self::get_admin(env.clone()).map_err(|_| IssuerError::IssuerNotFound)?;
        Self::require_auth(&admin);
        if algorithm == 0
            || Self::is_zero_hash(&env, &key_hash)
            || Self::is_zero_hash(&env, &reason_hash)
        {
            return Err(IssuerError::InvalidMetadataCommitment);
        }
        Self::get_issuer(env.clone(), issuer_id.clone())?;

        let marker = DataKey::CompromisedSigningKey(issuer_id.clone(), key_hash.clone(), algorithm);
        if env.storage().persistent().has(&marker) {
            return Err(IssuerError::InvalidTransition);
        }

        let active_key = DataKey::ActiveSigningKey(issuer_id.clone());
        let pending_key = DataKey::PendingSigningKey(issuer_id.clone());
        let active: Option<SigningKeyCommitment> = env.storage().persistent().get(&active_key);
        let pending: Option<SigningKeyCommitment> = env.storage().persistent().get(&pending_key);
        let active_matches = active
            .as_ref()
            .map(|key| key.key_hash == key_hash && key.algorithm == algorithm)
            .unwrap_or(false);
        let pending_matches = pending
            .as_ref()
            .map(|key| key.key_hash == key_hash && key.algorithm == algorithm)
            .unwrap_or(false);
        let history_count: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::SigningKeyHistoryCount(issuer_id.clone()))
            .unwrap_or(0);
        let mut known = active_matches || pending_matches;
        if !known {
            for index in 0..history_count {
                let historic: Option<SigningKeyCommitment> = env
                    .storage()
                    .persistent()
                    .get(&DataKey::SigningKeyHistory(issuer_id.clone(), index));
                if historic
                    .map(|key| key.key_hash == key_hash && key.algorithm == algorithm)
                    .unwrap_or(false)
                {
                    known = true;
                    break;
                }
            }
        }
        if !known {
            return Err(IssuerError::InvalidTransition);
        }

        let sequence: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::CompromisedSigningKeyCount(issuer_id.clone()))
            .unwrap_or(0);
        if sequence == u32::MAX {
            return Err(IssuerError::InvalidTransition);
        }
        let record = CompromisedSigningKeyRecord {
            record_version: 1,
            issuer_id_hash: issuer_id.clone(),
            key_hash,
            algorithm,
            effective_ledger: env.ledger().sequence(),
            reason_hash,
            sequence,
        };

        let history_key = DataKey::CompromisedSigningKeyHistory(issuer_id.clone(), sequence);
        let count_key = DataKey::CompromisedSigningKeyCount(issuer_id);
        env.storage().persistent().set(&marker, &record);
        env.storage().persistent().set(&history_key, &record);
        env.storage().persistent().set(&count_key, &(sequence + 1));
        for key in [&marker, &history_key, &count_key] {
            env.storage().persistent().extend_ttl(
                key,
                TTL_THRESHOLD_LEDGERS,
                TTL_EXTEND_TO_LEDGERS,
            );
        }
        if active_matches {
            env.storage().persistent().remove(&active_key);
        }
        if pending_matches {
            env.storage().persistent().remove(&pending_key);
        }
        IssuerSigningKeyCompromised {
            record: record.clone(),
            marked_by: admin,
        }
        .publish(&env);
        Ok(record)
    }

    pub fn get_compromised_signing_key(
        env: Env,
        issuer_id: BytesN<32>,
        key_hash: BytesN<32>,
        algorithm: u32,
    ) -> Option<SigningKeyLifecycleRecord> {
        Self::stored_signing_key_state(env, issuer_id, key_hash, algorithm)
    }

    /// Returns retained lifecycle entries in ascending sequence order.
    pub fn get_signing_key_lifecycle(
        env: Env,
        issuer_id: BytesN<32>,
        cursor: u64,
        limit: u32,
    ) -> Result<Vec<SigningKeyLifecycleRecord>, IssuerError> {
        if limit > MAX_SIGNING_KEY_LIFECYCLE_PAGE {
            return Err(IssuerError::BatchTooLarge);
        }
        let latest = Self::get_signing_key_usage_sequence(env.clone(), issuer_id.clone());
        let oldest = latest.saturating_sub(u64::from(
            SIGNING_KEY_LIFECYCLE_HISTORY_CAPACITY.saturating_sub(1),
        ));
        let mut sequence = cursor.max(1).max(oldest);
        let mut records = Vec::new(&env);
        while sequence <= latest && records.len() < limit {
            let slot = ((sequence - 1) % u64::from(SIGNING_KEY_LIFECYCLE_HISTORY_CAPACITY)) as u32;
            let key = DataKey::SigningKeyLifecycleHistory(issuer_id.clone(), slot);
            if let Some(record) = env
                .storage()
                .persistent()
                .get::<_, SigningKeyLifecycleRecord>(&key)
            {
                if record.sequence == sequence {
                    records.push_back(record);
                    Self::extend_issuer_key_ttl(env.clone(), &key);
                }
            }
            if sequence == latest {
                break;
            }
            sequence += 1;
    ) -> Option<CompromisedSigningKeyRecord> {
        env.storage()
            .persistent()
            .get(&DataKey::CompromisedSigningKey(
                issuer_id, key_hash, algorithm,
            ))
    }

    pub fn get_compromised_key_history(
        env: Env,
        issuer_id: BytesN<32>,
        cursor: u32,
        limit: u32,
    ) -> Result<Vec<CompromisedSigningKeyRecord>, IssuerError> {
        if limit == 0 || limit > MAX_SIGNING_KEY_COMPROMISE_HISTORY_PAGE {
            return Err(IssuerError::BatchTooLarge);
        }
        let count: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::CompromisedSigningKeyCount(issuer_id.clone()))
            .unwrap_or(0);
        let end = cursor.saturating_add(limit).min(count);
        let mut records = Vec::new(&env);
        for index in cursor..end {
            if let Some(record) =
                env.storage()
                    .persistent()
                    .get(&DataKey::CompromisedSigningKeyHistory(
                        issuer_id.clone(),
                        index,
                    ))
            {
                records.push_back(record);
            }
        }
        Ok(records)
    }

    pub fn set_issuer_policy_commitments(
        env: Env,
        issuer_id: BytesN<32>,
        commitments: IssuerPolicyCommitments,
    ) -> Result<(), IssuerError> {
        Self::assert_operational(&env);
        let admin = Self::get_admin(env.clone()).map_err(|_| IssuerError::IssuerNotFound)?;
        Self::require_auth(&admin);
        if commitments.encoding_version != 1
            || Self::is_zero_hash(&env, &commitments.category_commitment)
            || Self::is_zero_hash(&env, &commitments.jurisdiction_commitment)
        {
            return Err(IssuerError::InvalidMetadataCommitment);
        }
        Self::get_issuer(env.clone(), issuer_id.clone())?;
        env.storage()
            .persistent()
            .set(&DataKey::IssuerPolicy(issuer_id), &commitments);
        Ok(())
    }
    pub fn get_issuer_policy_commitments(
        env: Env,
        issuer_id: BytesN<32>,
    ) -> Option<IssuerPolicyCommitments> {
        env.storage()
            .persistent()
            .get(&DataKey::IssuerPolicy(issuer_id))
    }

    /// Commits a compact public assurance tier for an issuer.
    ///
    /// The current ledger is the exact effective boundary. All validation and
    /// checked sequence arithmetic complete before storage is changed.
    pub fn set_issuer_assurance_level(
        env: Env,
        issuer_id: BytesN<32>,
        commitment_version: u32,
        assurance_level: u32,
    ) -> Result<IssuerAssuranceLevelCommitment, IssuerError> {
        Self::assert_operational(&env);
        let admin = Self::get_admin(env.clone()).map_err(|_| IssuerError::IssuerNotFound)?;
        Self::require_auth(&admin);
        if commitment_version != ISSUER_ASSURANCE_COMMITMENT_VERSION
            || assurance_level == 0
            || assurance_level > MAX_ISSUER_ASSURANCE_LEVEL
        {
            return Err(IssuerError::InvalidAssuranceLevel);
        }
        let issuer = Self::get_issuer(env.clone(), issuer_id.clone())?;
        if issuer.status == IssuerStatus::Revoked {
            return Err(IssuerError::IssuerRevoked);
        }
        let current_key = DataKey::IssuerAssurance(issuer_id.clone());
        if env
            .storage()
            .persistent()
            .get::<_, IssuerAssuranceLevelCommitment>(&current_key)
            .map(|current| {
                current.commitment_version == commitment_version
                    && current.assurance_level == assurance_level
            })
            .unwrap_or(false)
        {
            return Err(IssuerError::InvalidTransition);
        }
        let sequence_key = DataKey::IssuerAssuranceSequence(issuer_id.clone());
        let sequence = env
            .storage()
            .persistent()
            .get::<_, u32>(&sequence_key)
            .unwrap_or(0)
            .checked_add(1)
            .ok_or(IssuerError::SequenceOverflow)?;
        let commitment = IssuerAssuranceLevelCommitment {
            commitment_version,
            assurance_level,
            effective_ledger: env.ledger().sequence(),
            sequence,
        };
        let slot = (sequence - 1) % ISSUER_ASSURANCE_HISTORY_CAPACITY;
        let history_key = DataKey::IssuerAssuranceHistory(issuer_id.clone(), slot);
        env.storage().persistent().set(&current_key, &commitment);
        env.storage().persistent().set(&history_key, &commitment);
        env.storage().persistent().set(&sequence_key, &sequence);
        Self::extend_issuer_key_ttl(env.clone(), &current_key);
        Self::extend_issuer_key_ttl(env.clone(), &history_key);
        Self::extend_issuer_key_ttl(env.clone(), &sequence_key);
        let epoch = Self::bump_epoch(&env);
        IssuerAssuranceLevelChanged {
            issuer_id_hash: issuer_id,
            commitment_version,
            assurance_level,
            effective_ledger: commitment.effective_ledger,
            sequence,
            changed_by: admin,
            epoch,
        }
        .publish(&env);
        Ok(commitment)
    }

    pub fn get_issuer_assurance_level(
        env: Env,
        issuer_id: BytesN<32>,
    ) -> Option<IssuerAssuranceLevelCommitment> {
        let key = DataKey::IssuerAssurance(issuer_id);
        let value = env.storage().persistent().get(&key);
        if value.is_some() {
            Self::extend_issuer_key_ttl(env, &key);
        }
        value
    }

    /// Returns a retained immutable assurance record by its binding sequence.
    pub fn get_issuer_assurance_at_sequence(
        env: Env,
        issuer_id: BytesN<32>,
        sequence: u32,
    ) -> Option<IssuerAssuranceLevelCommitment> {
        if sequence == 0 {
            return None;
        }
        let key = DataKey::IssuerAssuranceHistory(
            issuer_id,
            (sequence - 1) % ISSUER_ASSURANCE_HISTORY_CAPACITY,
        );
        let value: Option<IssuerAssuranceLevelCommitment> = env.storage().persistent().get(&key);
        match value {
            Some(record) if record.sequence == sequence => {
                Self::extend_issuer_key_ttl(env, &key);
                Some(record)
            }
            _ => None,
        }
    }

    /// Returns retained assurance history in ascending sequence order.
    pub fn get_issuer_assurance_history(
        env: Env,
        issuer_id: BytesN<32>,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<IssuerAssuranceLevelCommitment>, IssuerError> {
        if limit > MAX_ISSUER_ASSURANCE_HISTORY_PAGE {
            return Err(IssuerError::BatchTooLarge);
        }
        let latest: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::IssuerAssuranceSequence(issuer_id.clone()))
            .unwrap_or(0);
        let first = latest
            .saturating_sub(ISSUER_ASSURANCE_HISTORY_CAPACITY)
            .saturating_add(1);
        let mut sequence = first.saturating_add(offset);
        let mut records = Vec::new(&env);
        while sequence <= latest && records.len() < limit {
            if let Some(record) =
                Self::get_issuer_assurance_at_sequence(env.clone(), issuer_id.clone(), sequence)
            {
                records.push_back(record);
            }
            if sequence == latest {
                break;
            }
            sequence += 1;
        }
        Ok(records)
    }

    pub fn suspend_issuer(
        env: Env,
        proposal_id: BytesN<32>,
        issuer_id_hash: BytesN<32>,
        reason_commitment: BytesN<32>,
    ) -> Result<(), IssuerError> {
        Self::set_status(
            env,
            proposal_id,
            issuer_id_hash,
            IssuerStatus::Suspended,
            reason_commitment,
        )
    }

    pub fn reactivate_issuer(
        env: Env,
        proposal_id: BytesN<32>,
        issuer_id_hash: BytesN<32>,
        reason_commitment: BytesN<32>,
    ) -> Result<(), IssuerError> {
        Self::set_status(
            env,
            proposal_id,
            issuer_id_hash,
            IssuerStatus::Active,
            reason_commitment,
        )
    }

    pub fn revoke_issuer(
        env: Env,
        proposal_id: BytesN<32>,
        issuer_id_hash: BytesN<32>,
        reason_commitment: BytesN<32>,
    ) -> Result<(), IssuerError> {
        Self::set_status(
            env,
            proposal_id,
            issuer_id_hash,
            IssuerStatus::Revoked,
            reason_commitment,
        )
    }

    pub fn rotate_issuer_address(
        env: Env,
        issuer_id_hash: BytesN<32>,
        new_address: Address,
    ) -> Result<(), IssuerError> {
        Self::assert_operational(&env);
        let _admin = Self::get_admin(env.clone()).map_err(|_| IssuerError::IssuerNotFound)?;
        Self::require_valid_issuer_address(&new_address)?;
        let key = DataKey::Issuer(issuer_id_hash.clone());
        let record: IssuerRecord = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(IssuerError::IssuerNotFound)?;
        if record.status == IssuerStatus::Revoked {
            return Err(IssuerError::IssuerRevoked);
        }
        if record.status != IssuerStatus::Active {
            return Err(IssuerError::IssuerInactive);
        }
        if new_address == record.issuer_address {
            return Err(IssuerError::InvalidAddress);
        }
        let new_address_key = DataKey::AddressIssuer(new_address.clone());
        if env.storage().persistent().has(&new_address_key) {
            return Err(IssuerError::IssuerAddressAlreadyRegistered);
        }
        Self::require_auth(&record.issuer_address);
        let now_ledger = env.ledger().sequence();
        let expires_at_ledger = now_ledger.saturating_add(ISSUER_ROTATION_EXPIRY_LEDGERS);
        let pending = PendingIssuerRotation {
            old_address: record.issuer_address.clone(),
            new_address: new_address.clone(),
            nominated_at_ledger: now_ledger,
            expires_at_ledger,
        };
        env.storage().persistent().set(
            &DataKey::PendingIssuerRotation(issuer_id_hash.clone()),
            &pending,
        );
        Self::extend_issuer_key_ttl(
            env.clone(),
            &DataKey::PendingIssuerRotation(issuer_id_hash.clone()),
        );
        IssuerRotationNominated {
            issuer_id_hash,
            old_address: record.issuer_address,
            new_address,
            expires_at_ledger,
        }
        .publish(&env);
        Ok(())
    }

    pub fn get_pending_issuer_rotation(
        env: Env,
        issuer_id_hash: BytesN<32>,
    ) -> Option<PendingIssuerRotation> {
        env.storage()
            .persistent()
            .get(&DataKey::PendingIssuerRotation(issuer_id_hash))
    }

    pub fn cancel_issuer_address_rotation(
        env: Env,
        issuer_id_hash: BytesN<32>,
    ) -> Result<(), IssuerError> {
        let key = DataKey::PendingIssuerRotation(issuer_id_hash.clone());
        let pending: PendingIssuerRotation = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(IssuerError::InvalidTransition)?;
        Self::require_auth(&pending.old_address);
        env.storage().persistent().remove(&key);
        IssuerRotationCancelled {
            issuer_id_hash,
            cancelled_by: pending.old_address,
        }
        .publish(&env);
        Ok(())
    }

    pub fn accept_issuer_address_rotation(
        env: Env,
        issuer_id_hash: BytesN<32>,
    ) -> Result<(), IssuerError> {
        Self::assert_operational(&env);
        let pending_key = DataKey::PendingIssuerRotation(issuer_id_hash.clone());
        let pending: PendingIssuerRotation = env
            .storage()
            .persistent()
            .get(&pending_key)
            .ok_or(IssuerError::InvalidTransition)?;
        if env.ledger().sequence() >= pending.expires_at_ledger {
            return Err(IssuerError::InvalidTransition);
        }
        let issuer_key = DataKey::Issuer(issuer_id_hash.clone());
        let mut record: IssuerRecord = env
            .storage()
            .persistent()
            .get(&issuer_key)
            .ok_or(IssuerError::IssuerNotFound)?;
        if record.status == IssuerStatus::Revoked {
            return Err(IssuerError::IssuerRevoked);
        }
        if record.status != IssuerStatus::Active || record.issuer_address != pending.old_address {
            return Err(IssuerError::IssuerInactive);
        }
        Self::require_valid_issuer_address(&pending.new_address)?;
        let new_address_key = DataKey::AddressIssuer(pending.new_address.clone());
        if env.storage().persistent().has(&new_address_key) {
            return Err(IssuerError::IssuerAddressAlreadyRegistered);
        }
        Self::require_auth(&pending.new_address);

        let old_address = record.issuer_address.clone();
        env.storage()
            .persistent()
            .remove(&DataKey::AddressIssuer(old_address.clone()));
        env.storage()
            .persistent()
            .remove(&DataKey::AddressTtl(old_address.clone()));
        record.issuer_address = pending.new_address.clone();
        let now = env.ledger().timestamp();
        record.updated_at = now;
        env.storage().persistent().set(&issuer_key, &record);
        env.storage()
            .persistent()
            .set(&new_address_key, &issuer_id_hash.clone());
        env.storage().persistent().remove(&pending_key);
        Self::extend_issuer_key_ttl(env.clone(), &issuer_key);
        Self::extend_address_ttl(env.clone(), pending.new_address.clone());

        let count_key = DataKey::RotationCount(issuer_id_hash.clone());
        let rotation_count: u32 = env.storage().persistent().get(&count_key).unwrap_or(0);

        let rotation_record = RotationRecord {
            old_address: old_address.clone(),
            new_address: pending.new_address.clone(),
            rotated_at: now,
            ledger_sequence: env.ledger().sequence(),
        };

        let history_key = DataKey::RotationHistory(issuer_id_hash.clone(), rotation_count);
        env.storage()
            .persistent()
            .set(&history_key, &rotation_record);
        env.storage()
            .persistent()
            .set(&count_key, &(rotation_count + 1));
        Self::extend_issuer_key_ttl(env.clone(), &history_key);
        Self::extend_issuer_key_ttl(env.clone(), &count_key);

        let epoch = Self::bump_epoch(&env);
        IssuerAddressRotated {
            issuer_id_hash,
            old_address,
            new_address: pending.new_address,
            updated_at: now,
            epoch,
        }
        .publish(&env);
        Ok(())
    }

    pub fn get_rotation_count(env: Env, issuer_id_hash: BytesN<32>) -> Result<u32, IssuerError> {
        let _ = Self::get_issuer(env.clone(), issuer_id_hash.clone())?;
        let count_key = DataKey::RotationCount(issuer_id_hash);
        Ok(env.storage().persistent().get(&count_key).unwrap_or(0))
    }

    pub fn get_rotation_history(
        env: Env,
        issuer_id_hash: BytesN<32>,
        offset: u32,
        limit: u32,
    ) -> Result<soroban_sdk::Vec<RotationRecord>, IssuerError> {
        let _ = Self::get_issuer(env.clone(), issuer_id_hash.clone())?;
        if limit == 0 || limit > 50 {
            return Err(IssuerError::InvalidTransition);
        }

        let count = Self::get_rotation_count(env.clone(), issuer_id_hash.clone())?;
        if offset >= count {
            return Ok(soroban_sdk::Vec::new(&env));
        }

        let end = count.min(offset.saturating_add(limit));
        let mut history = soroban_sdk::Vec::new(&env);
        for i in offset..end {
            let history_key = DataKey::RotationHistory(issuer_id_hash.clone(), i);
            if let Some(record) = env
                .storage()
                .persistent()
                .get::<_, RotationRecord>(&history_key)
            {
                history.push_back(record);
            }
        }
        Ok(history)
    }

    pub fn get_issuer(env: Env, issuer_id_hash: BytesN<32>) -> Result<IssuerRecord, IssuerError> {
        let key = DataKey::Issuer(issuer_id_hash);
        let record = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(IssuerError::IssuerNotFound)?;
        Self::extend_issuer_key_ttl(env, &key);
        Ok(record)
    }

    pub fn is_active_issuer(env: Env, issuer_id_hash: BytesN<32>) -> bool {
        match Self::get_issuer(env, issuer_id_hash) {
            Ok(record) => record.status == IssuerStatus::Active,
            Err(_) => false,
        }
    }

    /// Returns the status of each supplied issuer identifier, in the same order
    /// as the request.
    ///
    /// This lets proof validation and indexer reconciliation inspect several
    /// issuers in one call instead of one cross-contract round trip per issuer.
    ///
    /// # Bounding
    /// The batch is rejected with [`IssuerError::BatchTooLarge`] when it carries
    /// more than [`MAX_ISSUER_STATUS_BATCH`] identifiers. The check runs before
    /// any storage read, so an oversized request cannot force unbounded host
    /// work. An empty batch is valid and yields an empty response.
    ///
    /// # Duplicates and unknown identifiers
    /// Input order is preserved and each occurrence produces its own entry, so a
    /// repeated identifier appears once per occurrence. An identifier with no
    /// registered issuer is reported as [`IssuerQueryStatus::NotFound`] rather
    /// than being omitted, keeping the response aligned with the request.
    ///
    /// # TTL
    /// A found issuer's record TTL is extended exactly as a single-item
    /// [`Self::get_issuer`] read would, so batching does not change the TTL
    /// behavior clients already rely on. Unknown identifiers touch no storage.
    pub fn get_issuer_statuses(
        env: Env,
        issuer_id_hashes: Vec<BytesN<32>>,
    ) -> Result<Vec<IssuerStatusResult>, IssuerError> {
        if issuer_id_hashes.len() > MAX_ISSUER_STATUS_BATCH {
            return Err(IssuerError::BatchTooLarge);
        }

        let mut results = Vec::new(&env);
        for issuer_id_hash in issuer_id_hashes.iter() {
            let key = DataKey::Issuer(issuer_id_hash.clone());
            let status = match env
                .storage()
                .persistent()
                .get::<DataKey, IssuerRecord>(&key)
            {
                Some(record) => {
                    Self::extend_issuer_key_ttl(env.clone(), &key);
                    match record.status {
                        IssuerStatus::Active => IssuerQueryStatus::Active,
                        IssuerStatus::Suspended => IssuerQueryStatus::Suspended,
                        IssuerStatus::Revoked => IssuerQueryStatus::Revoked,
                    }
                }
                None => IssuerQueryStatus::NotFound,
            };
            results.push_back(IssuerStatusResult {
                issuer_id_hash,
                status,
            });
        }
        Ok(results)
    }

    /// Returns a privacy-minimal revocation result even when the full issuer
    /// record no longer exists.
    pub fn get_issuer_revocation_tombstone(
        env: Env,
        issuer_id_hash: BytesN<32>,
    ) -> IssuerRevocationResult {
        let tombstone_key = DataKey::RevocationTombstone(issuer_id_hash.clone());
        if let Some(tombstone) = env
            .storage()
            .persistent()
            .get::<_, IssuerRevocationTombstone>(&tombstone_key)
        {
            Self::extend_tombstone_ttl(env, &tombstone_key);
            return IssuerRevocationResult {
                issuer_id_hash,
                state: IssuerRevocationState::Revoked,
                version: tombstone.version,
                revoked_at: tombstone.revoked_at,
                revoked_ledger: tombstone.revoked_ledger,
            };
        }
        let state = if env
            .storage()
            .persistent()
            .has(&DataKey::Issuer(issuer_id_hash.clone()))
        {
            IssuerRevocationState::NotRevoked
        } else {
            IssuerRevocationState::Unknown
        };
        IssuerRevocationResult {
            issuer_id_hash,
            state,
            version: 0,
            revoked_at: 0,
            revoked_ledger: 0,
        }
    }

    /// Bounded revocation lookup preserving input order and duplicates.
    pub fn get_issuer_revocation_statuses(
        env: Env,
        issuer_id_hashes: Vec<BytesN<32>>,
    ) -> Result<Vec<IssuerRevocationResult>, IssuerError> {
        if issuer_id_hashes.len() > MAX_ISSUER_STATUS_BATCH {
            return Err(IssuerError::BatchTooLarge);
        }
        let mut results = Vec::new(&env);
        for issuer_id_hash in issuer_id_hashes.iter() {
            results.push_back(Self::get_issuer_revocation_tombstone(
                env.clone(),
                issuer_id_hash,
            ));
        }
        Ok(results)
    }

    pub fn keepalive_revocation_tombstone(env: Env, issuer_id_hash: BytesN<32>) -> bool {
        let key = DataKey::RevocationTombstone(issuer_id_hash);
        if !env.storage().persistent().has(&key) {
            return false;
        }
        Self::extend_tombstone_ttl(env, &key);
        true
    }

    pub fn is_active_address(env: Env, issuer_address: Address) -> bool {
        let issuer_id_hash: Option<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::AddressIssuer(issuer_address.clone()));

        match issuer_id_hash {
            Some(id) => Self::is_active_issuer(env, id),
            None => false,
        }
    }

    // ── epoch, capacity, cooldown, interface version ──────────────────────────

    /// Machine-readable interface version this contract exposes to consumers.
    pub fn interface_version(_env: Env) -> InterfaceVersion {
        ISSUER_REGISTRY_INTERFACE_VERSION
    }

    /// Current registry epoch. Advances by one on every externally visible
    /// issuer mutation. A stable value means nothing has changed; consumers use
    /// it to skip refreshing cached issuer data. Starts at 0.
    pub fn get_issuer_epoch(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::IssuerEpoch)
            .unwrap_or(0)
    }

    /// Number of issuers currently in `Active` status.
    pub fn get_active_issuer_count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::ActiveIssuerCount)
            .unwrap_or(0)
    }

    /// Governed maximum number of simultaneously `Active` issuers. Defaults to
    /// `u32::MAX` (effectively unlimited) until an admin sets a bound.
    pub fn get_max_active_issuers(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::MaxActiveIssuers)
            .unwrap_or(u32::MAX)
    }

    /// Governed reactivation cooldown, in seconds. Defaults to 0 (no cooldown).
    pub fn get_reactivation_cooldown(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::ReactivationCooldown)
            .unwrap_or(0)
    }

    /// Earliest ledger time at which a suspended issuer may be reactivated.
    /// Returns 0 when the issuer has never been suspended or has no cooldown
    /// pending. Fixed at suspension time, so a later cooldown change does not
    /// move it.
    pub fn get_earliest_reactivation(env: Env, issuer_id_hash: BytesN<32>) -> u64 {
        Self::earliest_reactivation(&env, &issuer_id_hash)
    }

    /// Admin-only: set the maximum active-issuer capacity.
    ///
    /// A new limit below the current active usage is rejected with
    /// `MaxBelowActiveUsage` unless `allow_below_usage` is true, which lets an
    /// admin ratchet the ceiling down toward a target without first suspending
    /// issuers (no existing issuer is affected; only future reactivations and
    /// registrations see the tighter bound).
    pub fn set_max_active_issuers(
        env: Env,
        new_max: u32,
        allow_below_usage: bool,
    ) -> Result<(), IssuerError> {
        let admin = Self::get_admin(env.clone()).map_err(|_| IssuerError::IssuerNotFound)?;
        Self::require_auth(&admin);

        let active_count = Self::get_active_issuer_count(env.clone());
        if new_max < active_count && !allow_below_usage {
            return Err(IssuerError::MaxBelowActiveUsage);
        }

        env.storage()
            .instance()
            .set(&DataKey::MaxActiveIssuers, &new_max);
        Self::extend_instance_ttl(env.clone());

        MaxActiveIssuersChanged {
            new_max,
            active_count,
            changed_by: admin,
        }
        .publish(&env);
        Ok(())
    }

    /// Admin-only: set the reactivation cooldown, in seconds.
    ///
    /// The new value applies only to suspensions that happen after this call;
    /// the earliest reactivation time of an already-suspended issuer is fixed
    /// and is never retroactively shortened or lengthened.
    pub fn set_reactivation_cooldown(env: Env, cooldown_seconds: u64) -> Result<(), IssuerError> {
        let admin = Self::get_admin(env.clone()).map_err(|_| IssuerError::IssuerNotFound)?;
        Self::require_auth(&admin);

        env.storage()
            .instance()
            .set(&DataKey::ReactivationCooldown, &cooldown_seconds);
        Self::extend_instance_ttl(env.clone());

        ReactivationCooldownChanged {
            new_cooldown_seconds: cooldown_seconds,
            changed_by: admin,
        }
        .publish(&env);
        Ok(())
    }

    // ── upgrade governance ────────────────────────────────────────────────────

    /// Returns the stored monotonic contract version.  Starts at 1.
    pub fn get_contract_version(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::ContractVersion)
            .unwrap_or(0)
    }

    pub fn get_migration_status(env: Env) -> Option<MigrationStatus> {
        env.storage().instance().get(&DataKey::MigrationStatus)
    }

    pub fn begin_migration(
        env: Env,
        target_contract_version: u32,
        total_items: u32,
    ) -> Result<MigrationStatus, ContractError> {
        let admin = Self::get_admin(env.clone())?;
        Self::require_auth(&admin);
        if target_contract_version <= Self::get_contract_version(env.clone()) || total_items == 0 {
            return Err(ContractError::InvalidInput);
        }
        if let Some(status) = Self::get_migration_status(env.clone()) {
            return if status.target_contract_version == target_contract_version
                && status.total_items == total_items
            {
                Ok(status)
            } else {
                Err(ContractError::InvalidState)
            };
        }
        let status = MigrationStatus {
            status_version: MIGRATION_STATUS_VERSION,
            target_contract_version,
            cursor: 0,
            total_items,
            complete: false,
        };
        env.storage()
            .instance()
            .set(&DataKey::MigrationStatus, &status);
        Self::extend_instance_ttl(env);
        Ok(status)
    }

    pub fn advance_migration(
        env: Env,
        expected_cursor: u32,
        processed_items: u32,
    ) -> Result<MigrationStatus, ContractError> {
        let admin = Self::get_admin(env.clone())?;
        Self::require_auth(&admin);
        if processed_items == 0 || processed_items > MAX_MIGRATION_BATCH {
            return Err(ContractError::InvalidInput);
        }
        let mut status =
            Self::get_migration_status(env.clone()).ok_or(ContractError::InvalidState)?;
        if expected_cursor < status.cursor {
            return if expected_cursor.saturating_add(processed_items) <= status.cursor {
                Ok(status)
            } else {
                Err(ContractError::InvalidState)
            };
        }
        if expected_cursor != status.cursor || status.complete {
            return Err(ContractError::InvalidState);
        }
        let next = status
            .cursor
            .checked_add(processed_items)
            .ok_or(ContractError::InvalidInput)?;
        if next > status.total_items {
            return Err(ContractError::InvalidInput);
        }
        status.cursor = next;
        status.complete = next == status.total_items;
        env.storage()
            .instance()
            .set(&DataKey::MigrationStatus, &status);
        Self::extend_instance_ttl(env);
        Ok(status)
    }

    pub fn get_config_digest_version() -> u32 {
        earnproof_shared::CONFIG_DIGEST_VERSION
    }

    pub fn get_config_digest(env: Env) -> Result<BytesN<32>, ContractError> {
        let admin = Self::get_admin(env.clone())?;
        Ok(earnproof_shared::issuer_registry_digest(
            &env,
            &admin,
            Self::get_contract_version(env.clone()),
        ))
    }

    pub fn get_instance_ttl_status(env: Env) -> TtlStatus {
        earnproof_shared::ttl_status(
            env.ledger().sequence(),
            env.storage().instance().has(&DataKey::Admin),
            env.storage().instance().get(&DataKey::InstanceLiveUntil),
        )
    }

    pub fn get_issuer_ttl_status(env: Env, issuer_id_hash: BytesN<32>) -> TtlStatus {
        earnproof_shared::ttl_status(
            env.ledger().sequence(),
            env.storage()
                .persistent()
                .has(&DataKey::Issuer(issuer_id_hash.clone())),
            env.storage()
                .persistent()
                .get(&DataKey::IssuerTtl(issuer_id_hash)),
        )
    }

    pub fn get_address_ttl_status(env: Env, issuer_address: Address) -> TtlStatus {
        earnproof_shared::ttl_status(
            env.ledger().sequence(),
            env.storage()
                .persistent()
                .has(&DataKey::AddressIssuer(issuer_address.clone())),
            env.storage()
                .persistent()
                .get(&DataKey::AddressTtl(issuer_address)),
        )
    }

    pub fn refresh_instance_ttl(env: Env) -> Result<TtlStatus, ContractError> {
        let admin = Self::get_admin(env.clone())?;
        Self::require_auth(&admin);
        Self::extend_instance_ttl(env.clone());
        Ok(Self::get_instance_ttl_status(env))
    }

    /// Admin-only: add `wasm_hash` to the upgrade allowlist.
    ///
    /// Records an upgrade approval with timelock and expiry.
    ///
    /// # Timing
    /// - `earliest_execution` = current_ledger + UPGRADE_TIMELOCK_LEDGERS
    /// - `expires_at` = current_ledger + UPGRADE_APPROVAL_EXPIRY_LEDGERS
    ///
    /// # Re-approval
    /// Re-approval replaces ALL timing metadata. Old timing is
    /// never reused — prevents stale metadata from persisting.
    ///
    /// # Authorization
    /// Caller must be the authorized admin.
    pub fn approve_upgrade(
        env: Env,
        proposal_id: BytesN<32>,
        wasm_hash: BytesN<32>,
        new_version: u32,
    ) -> Result<(), ContractError> {
        Self::assert_operational(&env);
        let admin = Self::get_admin(env.clone()).map_err(|_| ContractError::NotInitialized)?;
        Self::require_auth(&admin);

        let current = Self::get_contract_version(env.clone());
        if new_version <= current {
            return Err(ContractError::InvalidInput);
        }

        let current_ledger = env.ledger().sequence();

        // Saturating arithmetic prevents overflow on boundary inputs
        let earliest_execution = current_ledger.saturating_add(UPGRADE_TIMELOCK_LEDGERS);
        let expires_at = current_ledger.saturating_add(UPGRADE_APPROVAL_EXPIRY_LEDGERS);

        // Validate timing invariants
        if earliest_execution > expires_at {
            return Err(ContractError::InvalidTimingConfig);
        }

        // Store approval — ALWAYS creates fresh timing metadata
        // Never reuses stale fields from a previous approval
        let approval = UpgradeApproval {
            wasm_hash: wasm_hash.clone(),
            created_at: current_ledger,
            earliest_execution,
            expires_at,
            approved_by: admin.clone(),
        };

        env.storage()
            .instance()
            .set(&DataKey::UpgradeApproval, &approval);
        Self::extend_instance_ttl(env.clone());

        UpgradeAllowlisted {
            proposal_id,
            wasm_hash,
            new_contract_version: new_version,
            approved_by: admin,
        }
        .publish(&env);
        Ok(())
    }

    /// Admin-only: remove a hash from the allowlist without applying it.
    pub fn revoke_upgrade(
        env: Env,
        proposal_id: BytesN<32>,
        wasm_hash: BytesN<32>,
    ) -> Result<(), ContractError> {
        Self::assert_operational(&env);
        let admin = Self::get_admin(env.clone()).map_err(|_| ContractError::NotInitialized)?;
        Self::require_auth(&admin);

        // Remove old-style allowlist entry if it exists (for backwards compatibility during transition)
        env.storage()
            .instance()
            .remove(&DataKey::AllowedWasm(wasm_hash.clone()));

        // Remove the approval
        env.storage().instance().remove(&DataKey::UpgradeApproval);

        UpgradeRevoked {
            proposal_id,
            wasm_hash,
            revoked_by: admin,
        }
        .publish(&env);
        Ok(())
    }

    /// Returns true when `wasm_hash` is on the allowlist.
    pub fn is_upgrade_allowed(env: Env, wasm_hash: BytesN<32>) -> bool {
        // Check new-style approval
        if let Some(approval) = env
            .storage()
            .instance()
            .get::<_, UpgradeApproval>(&DataKey::UpgradeApproval)
        {
            return approval.wasm_hash == wasm_hash;
        }
        // Fall back to old-style allowlist for backwards compatibility
        env.storage()
            .instance()
            .has(&DataKey::AllowedWasm(wasm_hash))
    }

    /// Admin-only: apply an in-place WASM upgrade.
    ///
    /// # Requirements
    /// 1. Caller is the admin.
    /// 2. An approval exists with matching wasm_hash.
    /// 3. current_ledger >= earliest_execution (timelock elapsed)
    /// 4. current_ledger < expires_at (approval not expired)
    /// 5. Target version is strictly greater than current (downgrade guard).
    ///
    /// On success the allowlist entry is consumed and `ContractVersion` is
    /// advanced.
    ///
    /// # Failed execution
    /// Approval state is left UNCHANGED on all rejection paths.
    /// Only successful execution removes the approval.
    pub fn upgrade_contract(
        env: Env,
        wasm_hash: BytesN<32>,
        new_version: u32,
    ) -> Result<(), ContractError> {
        Self::assert_operational(&env);
        let admin = Self::get_admin(env.clone()).map_err(|_| ContractError::NotInitialized)?;
        let old_version = Self::get_contract_version(env.clone());
        Self::require_auth(&admin);
        assert!(
            earnproof_shared::is_valid_principal_address(&admin),
            "invalid pre-upgrade admin address invariant"
        );

        // Load approval — error if none exists
        let approval: UpgradeApproval = env
            .storage()
            .instance()
            .get(&DataKey::UpgradeApproval)
            .ok_or(ContractError::NoUpgradeApproval)?;

        let current_ledger = env.ledger().sequence();

        // Check timelock: too early
        if current_ledger < approval.earliest_execution {
            return Err(ContractError::UpgradeTimelockNotElapsed);
        }

        assert!(old_version >= 1, "invalid pre-upgrade version invariant");

        // Check expiry: too late
        if current_ledger >= approval.expires_at {
            // Approval expired — leave state unchanged
            // Caller must re-approve
            return Err(ContractError::UpgradeApprovalExpired);
        }

        // Verify hash matches approved hash
        if wasm_hash != approval.wasm_hash {
            return Err(ContractError::WasmHashMismatch);
        }

        // Verify version is still valid
        if new_version <= old_version {
            return Err(ContractError::InvalidInput);
        }
        if let Some(status) = Self::get_migration_status(env.clone()) {
            if !status.complete || status.target_contract_version != new_version {
                panic!("required storage migration is incomplete");
            }
        }

        // All checks passed — execute upgrade
        // Consume allowlist entry before applying to prevent replay.
        env.storage()
            .instance()
            .remove(&DataKey::AllowedWasm(wasm_hash.clone()));

        // Remove approval after successful execution
        env.storage().instance().remove(&DataKey::UpgradeApproval);

        #[cfg(not(test))]
        env.deployer()
            .update_current_contract_wasm(wasm_hash.clone());

        let post_admin = Self::get_admin(env.clone()).expect("post-upgrade admin check failed");
        assert_eq!(
            admin, post_admin,
            "admin address invariant violated after upgrade"
        );

        env.storage()
            .instance()
            .set(&DataKey::ContractVersion, &new_version);

        let now = env.ledger().timestamp();
        let receipt = UpgradeReceipt {
            wasm_hash: wasm_hash.clone(),
            old_version,
            new_version,
            upgraded_at: now,
            upgraded_by: admin.clone(),
        };

        env.storage()
            .instance()
            .set(&DataKey::LatestUpgradeReceipt, &receipt);
        env.storage().instance().remove(&DataKey::MigrationStatus);
        Self::extend_instance_ttl(env.clone());

        ContractUpgraded {
            new_wasm_hash: wasm_hash,
            old_contract_version: old_version,
            new_contract_version: new_version,
            upgraded_by: admin,
        }
        .publish(&env);
        Ok(())
    }

    /// Admin-only: revoke an upgrade approval at any point in its lifetime.
    ///
    /// Revocation is allowed:
    /// - Before timelock elapses
    /// - During the valid execution window
    /// - Even after expiry (cleanup)
    ///
    /// # Authorization
    /// Only the admin can revoke.
    pub fn revoke_upgrade_approval(env: Env, proposal_id: BytesN<32>) -> Result<(), ContractError> {
        let admin = Self::get_admin(env.clone()).map_err(|_| ContractError::NotInitialized)?;
        Self::require_auth(&admin);

        Self::consume_proposal(&env, &proposal_id)?;

        // Allow revocation even if no approval exists (idempotent)
        env.storage().instance().remove(&DataKey::UpgradeApproval);

        Ok(())
    }

    pub fn get_latest_upgrade_receipt(env: Env) -> Option<UpgradeReceipt> {
        env.storage().instance().get(&DataKey::LatestUpgradeReceipt)
    }

    // ── private helpers ───────────────────────────────────────────────────────

    fn require_valid_admin(address: &Address) -> Result<(), ContractError> {
        if !earnproof_shared::is_valid_principal_address(address) {
            return Err(ContractError::InvalidInput);
        }
        Ok(())
    }

    fn assert_operational(env: &Env) {
        if Self::is_decommissioned(env.clone()) {
            panic!("contract is decommissioned");
        }
        if Self::get_migration_status(env.clone()).is_some_and(|status| !status.complete) {
            panic!("storage migration in progress");
        }
    }

    fn require_valid_issuer_address(address: &Address) -> Result<(), IssuerError> {
        if !earnproof_shared::is_valid_principal_address(address) {
            return Err(IssuerError::InvalidAddress);
        }
        Ok(())
    }

    /// The all-zero 32-byte digest, used as the "no URI commitment recorded"
    /// sentinel for `metadata_uri_hash`.
    fn zero_hash(env: &Env) -> BytesN<32> {
        BytesN::from_array(env, &[0u8; 32])
    }

    /// Returns true when `hash` is the all-zero digest (an empty commitment).
    fn is_zero_hash(env: &Env, hash: &BytesN<32>) -> bool {
        hash == &Self::zero_hash(env)
    }

    fn checked_signing_key_sequence(
        env: Env,
        issuer_id: BytesN<32>,
        steps: u32,
    ) -> Result<u64, IssuerError> {
        earnproof_shared::checked_signing_key_sequence(
            Self::get_signing_key_usage_sequence(env, issuer_id),
            steps,
        )
        .ok_or(IssuerError::SequenceOverflow)
    }

    fn store_signing_key_sequence(env: Env, issuer_id: BytesN<32>, sequence: u64) {
        let key = DataKey::SigningKeyUsageSequence(issuer_id);
        env.storage().persistent().set(&key, &sequence);
        Self::extend_issuer_key_ttl(env, &key);
    }

    fn stored_signing_key_state(
        env: Env,
        issuer_id: BytesN<32>,
        key_hash: BytesN<32>,
        algorithm: u32,
    ) -> Option<SigningKeyLifecycleRecord> {
        let key = DataKey::SigningKeyLifecycleState(issuer_id, key_hash, algorithm);
        let state = env.storage().persistent().get(&key);
        if state.is_some() {
            Self::extend_issuer_key_ttl(env, &key);
        }
        state
    }

    fn append_signing_key_transition(
        env: Env,
        issuer_id: BytesN<32>,
        key_hash: BytesN<32>,
        algorithm: u32,
        transition: SigningKeyLifecycleTransition,
        sequence: u64,
    ) -> SigningKeyLifecycleRecord {
        let record = SigningKeyLifecycleRecord {
            record_version: 1,
            key_hash: key_hash.clone(),
            algorithm,
            transition,
            sequence,
            effective_ledger: env.ledger().sequence(),
        };
        let slot = ((sequence - 1) % u64::from(SIGNING_KEY_LIFECYCLE_HISTORY_CAPACITY)) as u32;
        let history_key = DataKey::SigningKeyLifecycleHistory(issuer_id.clone(), slot);
        let state_key =
            DataKey::SigningKeyLifecycleState(issuer_id.clone(), key_hash.clone(), algorithm);
        env.storage().persistent().set(&history_key, &record);
        env.storage().persistent().set(&state_key, &record);
        Self::extend_issuer_key_ttl(env.clone(), &history_key);
        Self::extend_issuer_key_ttl(env.clone(), &state_key);
        SigningKeyLifecycleChanged {
            issuer_id,
            key_hash,
            algorithm,
            transition,
            sequence,
            effective_ledger: record.effective_ledger,
        }
        .publish(&env);
        record
    }

    fn set_status(
        env: Env,
        proposal_id: BytesN<32>,
        issuer_id_hash: BytesN<32>,
        status: IssuerStatus,
        reason_commitment: BytesN<32>,
    ) -> Result<(), IssuerError> {
        Self::set_status_as(
            env,
            proposal_id,
            issuer_id_hash,
            status,
            reason_commitment,
            None,
        )
    }

    fn set_status_as(
        env: Env,
        proposal_id: BytesN<32>,
        issuer_id_hash: BytesN<32>,
        status: IssuerStatus,
        reason_commitment: BytesN<32>,
        actor: Option<Address>,
    ) -> Result<(), IssuerError> {
        Self::assert_operational(&env);
        if reason_commitment == BytesN::from_array(&env, &[0u8; 32]) {
            return Err(IssuerError::InvalidAddress);
        }
        let admin = Self::get_admin(env.clone()).map_err(|_| IssuerError::IssuerNotFound)?;
        if let Some(actor) = actor {
            Self::require_active_role(&env, GovernanceRole::IssuerManagement, &actor)?;
        } else {
            Self::require_auth(&admin);
        }
        Self::consume_proposal(&env, &proposal_id).map_err(|_| IssuerError::InvalidTransition)?;

        let key = DataKey::Issuer(issuer_id_hash.clone());
        let mut record: IssuerRecord = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(IssuerError::IssuerNotFound)?;

        let previous = record.status.clone();
        if previous == IssuerStatus::Revoked {
            return if status == IssuerStatus::Revoked {
                Err(IssuerError::IssuerRevoked)
            } else {
                Err(IssuerError::InvalidTransition)
            };
        }

        // The new status and the ledger metadata marking when it became
        // effective are written together in a single persistent `set`, so a
        // status change is never stored without its effective ledger and
        // timestamp. Timing is sourced only from the host ledger environment.
        let now = env.ledger().timestamp();
        let effective_ledger = env.ledger().sequence();
        // Enforce cooldown and capacity, and adjust the active-issuer count, per
        // transition. All checks that can reject the call run before any state
        // is written, so a rejected transition mutates nothing.
        match status {
            IssuerStatus::Active => {
                if previous == IssuerStatus::Suspended {
                    let earliest = Self::earliest_reactivation(&env, &issuer_id_hash);
                    if now < earliest {
                        return Err(IssuerError::ReactivationCooldownActive);
                    }
                    // Reactivation returns the issuer to Active, reclaiming a slot.
                    Self::reserve_active_capacity(&env)?;
                    env.storage()
                        .persistent()
                        .remove(&DataKey::ReactivatableAt(issuer_id_hash.clone()));
                }
            }
            IssuerStatus::Suspended => {
                if previous == IssuerStatus::Active {
                    Self::release_active_capacity(&env);
                }
                if previous != IssuerStatus::Revoked {
                    // Fix the earliest reactivation time from the cooldown in
                    // force now. A later cooldown change does not move it.
                    let cooldown = Self::get_reactivation_cooldown(env.clone());
                    let earliest = now.saturating_add(cooldown);
                    env.storage()
                        .persistent()
                        .set(&DataKey::ReactivatableAt(issuer_id_hash.clone()), &earliest);
                    Self::extend_reactivatable_ttl(env.clone(), &issuer_id_hash);
                }
            }
            IssuerStatus::Revoked => {
                if previous == IssuerStatus::Active {
                    Self::release_active_capacity(&env);
                }
                let tombstone_key = DataKey::RevocationTombstone(issuer_id_hash.clone());
                let tombstone = IssuerRevocationTombstone {
                    version: ISSUER_REVOCATION_TOMBSTONE_VERSION,
                    issuer_id_hash: issuer_id_hash.clone(),
                    revoked_at: now,
                    revoked_ledger: effective_ledger,
                };
                env.storage().persistent().set(&tombstone_key, &tombstone);
                Self::extend_tombstone_ttl(env.clone(), &tombstone_key);
            }
        }

        env.storage()
            .persistent()
            .remove(&DataKey::PendingIssuerRotation(issuer_id_hash.clone()));

        record.status = status.clone();
        record.reason_commitment = Some(reason_commitment.clone());
        record.updated_at = now;
        record.status_effective_ledger = effective_ledger;
        record.status_effective_timestamp = now;
        env.storage().persistent().set(&key, &record);
        Self::extend_issuer_key_ttl(env.clone(), &key);

        let epoch = Self::bump_epoch(&env);
        match status {
            IssuerStatus::Active => IssuerReactivated {
                issuer_id_hash,
                effective_ledger,
                effective_timestamp: now,
                reason_commitment,
                updated_at: now,
                epoch,
            }
            .publish(&env),
            IssuerStatus::Suspended => IssuerSuspended {
                issuer_id_hash,
                effective_ledger,
                effective_timestamp: now,
                reason_commitment,
                updated_at: now,
                epoch,
            }
            .publish(&env),
            IssuerStatus::Revoked => IssuerRevoked {
                issuer_id_hash,
                effective_ledger,
                effective_timestamp: now,
                reason_commitment,
                updated_at: now,
                epoch,
            }
            .publish(&env),
        }
        Ok(())
    }

    fn require_active_role(
        env: &Env,
        role: GovernanceRole,
        actor: &Address,
    ) -> Result<(), IssuerError> {
        let admin = Self::get_admin(env.clone()).map_err(|_| IssuerError::IssuerNotFound)?;
        if actor == &admin {
            Self::require_auth(actor);
            return Ok(());
        }
        let assignment: GovernanceRoleAssignment = env
            .storage()
            .persistent()
            .get(&DataKey::GovernanceAssignment(role, actor.clone()))
            .ok_or(IssuerError::InvalidTransition)?;
        if !assignment.is_active_at(env.ledger().sequence()) {
            return Err(IssuerError::InvalidTransition);
        }
        Self::require_auth(actor);
        Ok(())
    }

    /// Advances the registry epoch by one and returns the new value.
    /// Overflow is explicit: at `u64::MAX` the call panics rather than wrapping,
    /// which is unreachable in practice (one bump per mutation).
    fn bump_epoch(env: &Env) -> u64 {
        let current = Self::get_issuer_epoch(env.clone());
        let next = current
            .checked_add(1)
            .unwrap_or_else(|| panic!("issuer epoch overflow: reached maximum"));
        env.storage().instance().set(&DataKey::IssuerEpoch, &next);
        Self::extend_instance_ttl(env.clone());
        next
    }

    /// Reserves one active-issuer slot, rejecting if the governed capacity is
    /// already full. Increments the active count on success.
    fn reserve_active_capacity(env: &Env) -> Result<(), IssuerError> {
        let count = Self::get_active_issuer_count(env.clone());
        let max = Self::get_max_active_issuers(env.clone());
        if count >= max {
            return Err(IssuerError::IssuerCapacityExceeded);
        }
        let next = count
            .checked_add(1)
            .ok_or(IssuerError::IssuerCapacityExceeded)?;
        env.storage()
            .instance()
            .set(&DataKey::ActiveIssuerCount, &next);
        Self::extend_instance_ttl(env.clone());
        Ok(())
    }

    /// Releases one active-issuer slot. Saturates at zero as a defensive
    /// measure; the accounting never underflows on a valid transition.
    fn release_active_capacity(env: &Env) {
        let count = Self::get_active_issuer_count(env.clone());
        let next = count.saturating_sub(1);
        env.storage()
            .instance()
            .set(&DataKey::ActiveIssuerCount, &next);
        Self::extend_instance_ttl(env.clone());
    }

    fn earliest_reactivation(env: &Env, issuer_id_hash: &BytesN<32>) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::ReactivatableAt(issuer_id_hash.clone()))
            .unwrap_or(0)
    }

    fn extend_reactivatable_ttl(env: Env, issuer_id_hash: &BytesN<32>) {
        env.storage().persistent().extend_ttl(
            &DataKey::ReactivatableAt(issuer_id_hash.clone()),
            TTL_THRESHOLD_LEDGERS,
            TTL_EXTEND_TO_LEDGERS,
        );
    }

    fn extend_instance_ttl(env: Env) {
        env.storage()
            .instance()
            .extend_ttl(TTL_THRESHOLD_LEDGERS, TTL_EXTEND_TO_LEDGERS);
        let live_until = Self::tracked_live_until(&env);
        env.storage()
            .instance()
            .set(&DataKey::InstanceLiveUntil, &live_until);
    }

    fn extend_issuer_ttl(env: Env, issuer_id_hash: BytesN<32>) {
        Self::extend_issuer_key_ttl(env, &DataKey::Issuer(issuer_id_hash));
    }

    fn extend_issuer_key_ttl(env: Env, key: &DataKey) {
        env.storage()
            .persistent()
            .extend_ttl(key, TTL_THRESHOLD_LEDGERS, TTL_EXTEND_TO_LEDGERS);
        if let DataKey::Issuer(issuer_id_hash) = key {
            let tracker = DataKey::IssuerTtl(issuer_id_hash.clone());
            let live_until = Self::tracked_live_until(&env);
            env.storage().persistent().set(&tracker, &live_until);
            env.storage().persistent().extend_ttl(
                &tracker,
                TTL_THRESHOLD_LEDGERS,
                TTL_EXTEND_TO_LEDGERS,
            );
        }
    }

    fn extend_tombstone_ttl(env: Env, key: &DataKey) {
        env.storage()
            .persistent()
            .extend_ttl(key, TTL_THRESHOLD_LEDGERS, TTL_EXTEND_TO_LEDGERS);
    }

    fn extend_address_ttl(env: Env, issuer_address: Address) {
        let tracker = DataKey::AddressTtl(issuer_address.clone());
        let live_until = Self::tracked_live_until(&env);
        env.storage().persistent().extend_ttl(
            &DataKey::AddressIssuer(issuer_address),
            TTL_THRESHOLD_LEDGERS,
            TTL_EXTEND_TO_LEDGERS,
        );
        env.storage().persistent().set(&tracker, &live_until);
        env.storage().persistent().extend_ttl(
            &tracker,
            TTL_THRESHOLD_LEDGERS,
            TTL_EXTEND_TO_LEDGERS,
        );
    }

    fn tracked_live_until(env: &Env) -> u32 {
        env.ledger()
            .sequence()
            .saturating_add(TTL_EXTEND_TO_LEDGERS.min(env.storage().max_ttl()))
    }

    fn require_auth(address: &Address) {
        address.require_auth();
    }

    pub fn get_issuer_by_address(
        env: Env,
        issuer_address: Address,
    ) -> Result<IssuerRecord, IssuerError> {
        let issuer_id_hash: BytesN<32> = env
            .storage()
            .persistent()
            .get(&DataKey::AddressIssuer(issuer_address.clone()))
            .ok_or(IssuerError::IssuerAddressNotFound)?;

        let record = env
            .storage()
            .persistent()
            .get(&DataKey::Issuer(issuer_id_hash))
            .ok_or(IssuerError::IssuerNotFound)?;
        Self::extend_address_ttl(env, issuer_address);
        Ok(record)
    }
}

#[cfg(test)]
mod test {
    extern crate std;

    use super::{DataKey, IssuerRegistryContract, IssuerRegistryContractClient};
    use earnproof_shared::{
        ContractError, GovernanceRole, GovernanceRoleState, IssuerError, IssuerQueryStatus,
        IssuerStatus, IssuerStatusResult, MAX_ISSUER_STATUS_BATCH, TTL_THRESHOLD_LEDGERS,
        AdminTransferState, ContractError, IssuerError, IssuerQueryStatus, IssuerStatus,
        IssuerStatusResult, ADMIN_TRANSFER_EXPIRY_LEDGERS, MAX_ISSUER_STATUS_BATCH,
        ContractError, IssuerError, IssuerQueryStatus, IssuerRevocationState, IssuerStatus,
        IssuerStatusResult, ISSUER_REVOCATION_TOMBSTONE_VERSION, MAX_ISSUER_STATUS_BATCH,
        TTL_THRESHOLD_LEDGERS,
    };
    use soroban_sdk::{
        testutils::{
            storage::Persistent as _, Address as _, Events, Ledger as _, MockAuth, MockAuthInvoke,
        },
        vec, Address, BytesN, Env, IntoVal, Vec,
    };

    const ADMIN: &str = "GCFIRY65OQE7DFP5KLNS2PF2LVZMUZYJX4OZIEQ36N2IQANUB5XVYOJR";
    const ISSUER_ONE: &str = "GCATS5YOVB6ROX2WUNKGNQ2MP3GMXDMKSG2O4N5CLX3A6W4PZGZZI55U";
    const ISSUER_TWO: &str = "GDWUSKGGFDI4FRXK5EBTRECZSVQSSWJHHJOGH6JWG3AUMFFMQ435DIAG";

    fn bytes(env: &Env, value: u8) -> BytesN<32> {
        BytesN::from_array(env, &[value; 32])
    }

    fn setup() -> (Env, IssuerRegistryContractClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(IssuerRegistryContract, ());
        let client = IssuerRegistryContractClient::new(&env, &contract_id);
        let admin = Address::from_str(&env, ADMIN);
        client.initialize(&admin);
        (env, client, admin)
    }

    // ── existing tests ────────────────────────────────────────────────────────
    // -----------------------------------------------------------------------
    // Existing behavioral tests (preserved)
    // -----------------------------------------------------------------------

    #[test]
    fn admin_nomination_expiry_never_changes_current_admin() {
        let (env, client, admin) = setup();
        env.ledger().set_sequence_number(40);
        let nominee = Address::from_str(&env, ISSUER_ONE);
        client.nominate_admin(&nominee);
        let status = client.get_admin_transfer_status();
        assert_eq!(status.state, AdminTransferState::Pending);
        assert_eq!(status.expires_at_ledger, 40 + ADMIN_TRANSFER_EXPIRY_LEDGERS);

        env.ledger().set_sequence_number(status.expires_at_ledger);
        assert_eq!(
            client.try_accept_admin(),
            Err(Ok(ContractError::AdminNominationExpired))
        );
        assert_eq!(client.get_admin(), admin);
        client.cleanup_expired_admin_transfer();
        assert_eq!(
            client.get_admin_transfer_status().state,
            AdminTransferState::Absent
        );
        assert_eq!(client.get_admin(), admin);
    }

    #[test]
    fn registers_and_reads_active_issuer() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 1);
        let metadata_hash = bytes(&env, 2);
        let provenance_commitment = bytes(&env, 99);
        let issuer_address = Address::from_str(&env, ISSUER_ONE);

        client.register_issuer(
            &issuer_id,
            &issuer_address,
            &metadata_hash,
            &provenance_commitment,
        );

        let record = client.get_issuer(&issuer_id);
        assert_eq!(record.issuer_id_hash, issuer_id);
        assert_eq!(record.issuer_address, issuer_address);
        assert_eq!(record.metadata_hash, metadata_hash);
        assert_eq!(record.provenance_commitment, provenance_commitment);
        assert_eq!(record.status, IssuerStatus::Active);
        assert!(client.is_active_issuer(&issuer_id));
        assert!(client.is_active_address(&issuer_address));
    }

    #[test]
    fn status_transitions_reject_reactivated_revoked_issuer() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 1);
        let issuer_address = Address::from_str(&env, ISSUER_ONE);

        client.register_issuer(
            &issuer_id,
            &issuer_address,
            &bytes(&env, 2),
            &bytes(&env, 99),
        );
        let reason = soroban_sdk::BytesN::from_array(&client.env, &[1u8; 32]);
        client.suspend_issuer(&bytes(&env, 0x90), &issuer_id, &reason);
        assert!(!client.is_active_issuer(&issuer_id));

        client.reactivate_issuer(&bytes(&env, 0x91), &issuer_id, &reason);
        assert!(client.is_active_issuer(&issuer_id));

        client.revoke_issuer(&bytes(&env, 0x92), &issuer_id, &reason);
        assert!(!client.is_active_issuer(&issuer_id));
    }

    #[test]
    fn rejects_duplicate_issuer_id() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 1);
        let issuer_address = Address::from_str(&env, ISSUER_ONE);

        client.register_issuer(
            &issuer_id,
            &issuer_address,
            &bytes(&env, 2),
            &bytes(&env, 99),
        );

        let result = client.try_register_issuer(
            &issuer_id,
            &Address::from_str(&env, ISSUER_TWO),
            &bytes(&env, 3),
            &bytes(&env, 99),
        );
        assert_eq!(result, Err(Ok(IssuerError::IssuerAlreadyRegistered)));
    }

    #[test]
    fn revoked_issuer_cannot_be_reactivated() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 1);
        let issuer_address = Address::from_str(&env, ISSUER_ONE);

        client.register_issuer(
            &issuer_id,
            &issuer_address,
            &bytes(&env, 2),
            &bytes(&env, 99),
        );
        let reason = soroban_sdk::BytesN::from_array(&client.env, &[1u8; 32]);
        client.revoke_issuer(&bytes(&env, 0x90), &issuer_id, &reason);

        let result = client.try_reactivate_issuer(&bytes(&env, 0x91), &issuer_id, &reason);
        assert_eq!(result, Err(Ok(IssuerError::InvalidTransition)));
    }

    #[test]
    fn revocation_tombstone_is_minimal_stable_and_blocks_identifier_reuse() {
        let (env, client, _admin) = setup();
        env.ledger().set_sequence_number(77);
        env.ledger().set_timestamp(1_234);
        let issuer_id = bytes(&env, 0x41);
        let issuer_address = Address::from_str(&env, ISSUER_ONE);
        client.register_issuer(
            &issuer_id,
            &issuer_address,
            &bytes(&env, 0x42),
            &bytes(&env, 0x43),
        );
        client.revoke_issuer(&bytes(&env, 0x44), &issuer_id, &bytes(&env, 0x45));

        let result = client.get_issuer_revocation_tombstone(&issuer_id);
        assert_eq!(result.state, IssuerRevocationState::Revoked);
        assert_eq!(result.version, ISSUER_REVOCATION_TOMBSTONE_VERSION);
        assert_eq!(result.issuer_id_hash, issuer_id);
        assert_eq!(result.revoked_at, 1_234);
        assert_eq!(result.revoked_ledger, 77);

        env.ledger().set_sequence_number(78);
        env.ledger().set_timestamp(1_235);
        assert_eq!(
            client.try_revoke_issuer(&bytes(&env, 0x46), &issuer_id, &bytes(&env, 0x47)),
            Err(Ok(IssuerError::IssuerRevoked))
        );
        assert_eq!(client.get_issuer_revocation_tombstone(&issuer_id), result);

        env.as_contract(&client.address, || {
            env.storage()
                .persistent()
                .remove(&DataKey::Issuer(issuer_id.clone()));
        });
        assert_eq!(
            client.try_register_issuer(
                &issuer_id,
                &Address::from_str(&env, ISSUER_TWO),
                &bytes(&env, 0x48),
                &bytes(&env, 0x49),
            ),
            Err(Ok(IssuerError::IssuerRevoked))
        );
    }

    #[test]
    fn revocation_status_queries_are_bounded_ordered_and_unambiguous() {
        let (env, client, _admin) = setup();
        let active = bytes(&env, 0x51);
        let revoked = bytes(&env, 0x52);
        let unknown = bytes(&env, 0x53);
        client.register_issuer(
            &active,
            &Address::from_str(&env, ISSUER_ONE),
            &bytes(&env, 0x54),
            &bytes(&env, 0x55),
        );
        client.register_issuer(
            &revoked,
            &Address::from_str(&env, ISSUER_TWO),
            &bytes(&env, 0x56),
            &bytes(&env, 0x57),
        );
        client.revoke_issuer(&bytes(&env, 0x58), &revoked, &bytes(&env, 0x59));

        let request = vec![
            &env,
            unknown.clone(),
            revoked.clone(),
            active.clone(),
            revoked,
        ];
        let results = client.get_issuer_revocation_statuses(&request);
        assert_eq!(
            results.get(0).unwrap().state,
            IssuerRevocationState::Unknown
        );
        assert_eq!(
            results.get(1).unwrap().state,
            IssuerRevocationState::Revoked
        );
        assert_eq!(
            results.get(2).unwrap().state,
            IssuerRevocationState::NotRevoked
        );
        assert_eq!(
            results.get(3).unwrap().state,
            IssuerRevocationState::Revoked
        );

        let mut oversized = Vec::new(&env);
        for index in 0..=MAX_ISSUER_STATUS_BATCH {
            oversized.push_back(BytesN::from_array(&env, &[index as u8; 32]));
        }
        assert_eq!(
            client.try_get_issuer_revocation_statuses(&oversized),
            Err(Ok(IssuerError::BatchTooLarge))
        );
    }

    #[test]
    fn extends_issuer_storage_ttl() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 1);
        let issuer_address = Address::from_str(&env, ISSUER_ONE);

        client.register_issuer(
            &issuer_id,
            &issuer_address,
            &bytes(&env, 2),
            &bytes(&env, 99),
        );

        env.as_contract(&client.address, || {
            assert!(
                env.storage()
                    .persistent()
                    .get_ttl(&DataKey::Issuer(issuer_id.clone()))
                    > TTL_THRESHOLD_LEDGERS
            );
            assert!(
                env.storage()
                    .persistent()
                    .get_ttl(&DataKey::AddressIssuer(issuer_address.clone()))
                    > TTL_THRESHOLD_LEDGERS
            );
        });
    }

    // ── bounded batch issuer status query tests ───────────────────────────────

    #[test]
    fn batch_status_reports_each_state_in_request_order() {
        let (env, client, _admin) = setup();
        let active = bytes(&env, 1);
        let suspended = bytes(&env, 2);
        let revoked = bytes(&env, 3);
        let unknown = bytes(&env, 4);

        client.register_issuer(
            &active,
            &Address::from_str(&env, ISSUER_ONE),
            &bytes(&env, 10),
            &soroban_sdk::BytesN::from_array(&env, &[0x99u8; 32]),
        );
        client.register_issuer(
            &suspended,
            &Address::from_str(&env, ISSUER_TWO),
            &bytes(&env, 11),
            &soroban_sdk::BytesN::from_array(&env, &[0x99u8; 32]),
        );
        client.suspend_issuer(
            &suspended,
            &soroban_sdk::BytesN::from_array(&env, &[0x99u8; 32]),
        );
        client.register_issuer(
            &revoked,
            &Address::generate(&env),
            &bytes(&env, 12),
            &soroban_sdk::BytesN::from_array(&env, &[0x99u8; 32]),
        );
        client.revoke_issuer(
            &revoked,
            &soroban_sdk::BytesN::from_array(&env, &[0x99u8; 32]),
        );

        // Deliberately out of registration order to prove request order wins.
        let request = vec![
            &env,
            unknown.clone(),
            revoked.clone(),
            active.clone(),
            suspended.clone(),
        ];
        let results = client.get_issuer_statuses(&request);

        let expected = vec![
            &env,
            IssuerStatusResult {
                issuer_id_hash: unknown,
                status: IssuerQueryStatus::NotFound,
            },
            IssuerStatusResult {
                issuer_id_hash: revoked,
                status: IssuerQueryStatus::Revoked,
            },
            IssuerStatusResult {
                issuer_id_hash: active,
                status: IssuerQueryStatus::Active,
            },
            IssuerStatusResult {
                issuer_id_hash: suspended,
                status: IssuerQueryStatus::Suspended,
            },
        ];
        assert_eq!(results, expected);
    }

    #[test]
    fn batch_status_preserves_duplicate_identifiers() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 1);
        client.register_issuer(
            &issuer_id,
            &Address::from_str(&env, ISSUER_ONE),
            &bytes(&env, 2),
            &soroban_sdk::BytesN::from_array(&env, &[0x99u8; 32]),
        );

        let request = vec![
            &env,
            issuer_id.clone(),
            issuer_id.clone(),
            issuer_id.clone(),
        ];
        let results = client.get_issuer_statuses(&request);

        assert_eq!(results.len(), 3);
        for entry in results.iter() {
            assert_eq!(entry.issuer_id_hash, issuer_id);
            assert_eq!(entry.status, IssuerQueryStatus::Active);
        }
    }

    #[test]
    fn batch_status_empty_request_returns_empty_response() {
        let (env, client, _admin) = setup();
        let request: Vec<BytesN<32>> = Vec::new(&env);
        let results = client.get_issuer_statuses(&request);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn batch_status_at_maximum_is_accepted() {
        let (env, client, _admin) = setup();
        let mut request: Vec<BytesN<32>> = Vec::new(&env);
        for index in 0..MAX_ISSUER_STATUS_BATCH {
            request.push_back(bytes(&env, index as u8));
        }
        let results = client.get_issuer_statuses(&request);
        assert_eq!(results.len(), MAX_ISSUER_STATUS_BATCH);
        // None of these were registered, so every entry is unambiguously unknown.
        for entry in results.iter() {
            assert_eq!(entry.status, IssuerQueryStatus::NotFound);
        }
    }

    #[test]
    fn batch_status_over_maximum_is_rejected_before_reads() {
        let (env, client, _admin) = setup();
        let mut request: Vec<BytesN<32>> = Vec::new(&env);
        for index in 0..(MAX_ISSUER_STATUS_BATCH + 1) {
            request.push_back(bytes(&env, index as u8));
        }
        let result = client.try_get_issuer_statuses(&request);
        assert_eq!(result, Err(Ok(IssuerError::BatchTooLarge)));
    }

    #[test]
    fn batch_status_extends_ttl_like_single_item_query() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 7);
        client.register_issuer(
            &issuer_id,
            &Address::from_str(&env, ISSUER_ONE),
            &bytes(&env, 8),
            &soroban_sdk::BytesN::from_array(&env, &[0x99u8; 32]),
        );

        let request = vec![&env, issuer_id.clone()];
        let _ = client.get_issuer_statuses(&request);

        env.as_contract(&client.address, || {
            assert!(
                env.storage()
                    .persistent()
                    .get_ttl(&DataKey::Issuer(issuer_id.clone()))
                    > TTL_THRESHOLD_LEDGERS
            );
        });
    }

    // ── upgrade governance tests ──────────────────────────────────────────────

    #[test]
    fn contract_version_initialized_to_one() {
        let (_env, client, _admin) = setup();
        assert_eq!(client.get_contract_version(), 1);
    }

    #[test]
    fn approve_and_check_allowlist() {
        let (env, client, _admin) = setup();
        let hash = bytes(&env, 0xab);

        assert!(!client.is_upgrade_allowed(&hash));
        client.approve_upgrade(&bytes(&env, 0x90), &hash, &2);
        assert!(client.is_upgrade_allowed(&hash));
    }

    #[test]
    fn revoke_removes_from_allowlist() {
        let (env, client, _admin) = setup();
        let hash = bytes(&env, 0xcd);

        client.approve_upgrade(&bytes(&env, 0x90), &hash, &2);
        client.revoke_upgrade_approval(&bytes(&env, 0x91));
        assert!(!client.is_upgrade_allowed(&hash));
    }

    #[test]
    fn approve_upgrade_rejects_downgrade_version() {
        let (env, client, _admin) = setup();
        let res = client.try_approve_upgrade(&bytes(&env, 0x90), &bytes(&env, 1), &1);
        assert_eq!(res, Err(Ok(ContractError::InvalidInput)));
    }

    #[test]
    fn upgrade_contract_rejects_non_allowlisted_hash() {
        let (env, client, _admin) = setup();
        let res = client.try_upgrade_contract(&bytes(&env, 0xff), &2);
        assert_eq!(res, Err(Ok(ContractError::NoUpgradeApproval)));
    }

    /// Auth guard: upgrade_contract without admin signature must panic.
    #[test]
    #[should_panic]
    fn upgrade_contract_requires_admin_auth() {
        let env = Env::default();
        let contract_id = env.register(IssuerRegistryContract, ());
        let client = IssuerRegistryContractClient::new(&env, &contract_id);
        let admin = Address::from_str(&env, ADMIN);

        env.mock_all_auths();
        client.initialize(&admin);
        let hash = BytesN::from_array(&env, &[0xde; 32]);
        client.approve_upgrade(&hash, &2);
        env.ledger().set_sequence_number(
            env.ledger().sequence() + earnproof_shared::UPGRADE_TIMELOCK_LEDGERS,
        );
        env.set_auths(&[]);

        client.upgrade_contract(&hash, &2);
    }

    #[test]
    fn upgrade_advances_version_and_consumes_allowlist() {
        let (env, client, _admin) = setup();
        let hash = bytes(&env, 0x42);

        client.approve_upgrade(&hash, &2);
        env.ledger().set_sequence_number(
            env.ledger().sequence() + earnproof_shared::UPGRADE_TIMELOCK_LEDGERS,
        );
        client.upgrade_contract(&hash, &2);

        assert_eq!(client.get_contract_version(), 2);
        assert!(!client.is_upgrade_allowed(&hash));
    }

    #[test]
    fn upgrade_hash_cannot_be_replayed() {
        let (env, client, _admin) = setup();
        let hash = bytes(&env, 0x42);

        client.approve_upgrade(&hash, &2);
        env.ledger().set_sequence_number(
            env.ledger().sequence() + earnproof_shared::UPGRADE_TIMELOCK_LEDGERS,
        );
        client.upgrade_contract(&hash, &2);
        let res = client.try_upgrade_contract(&hash, &2);
        assert_eq!(res, Err(Ok(ContractError::NoUpgradeApproval)));
    }

    /// Persistent issuer state must survive an upgrade.
    #[test]
    fn state_preserved_across_upgrade() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 1);
        let issuer_address = Address::from_str(&env, ISSUER_ONE);

        client.register_issuer(
            &issuer_id,
            &issuer_address,
            &bytes(&env, 2),
            &bytes(&env, 99),
        );
        assert!(client.is_active_issuer(&issuer_id));

        let hash = bytes(&env, 0x77);
        client.approve_upgrade(&hash, &2);
        env.ledger().set_sequence_number(
            env.ledger().sequence() + earnproof_shared::UPGRADE_TIMELOCK_LEDGERS,
        );
        client.upgrade_contract(&hash, &2);

        // Issuer record must still be intact.
        assert!(client.is_active_issuer(&issuer_id));
        assert_eq!(client.get_contract_version(), 2);
    }

    #[test]
    fn cannot_re_approve_old_version_after_upgrade() {
        let (env, client, _admin) = setup();
        let hash_v2 = bytes(&env, 0x01);
        let old_hash = bytes(&env, 0x02);

        client.approve_upgrade(&hash_v2, &2);
        env.ledger().set_sequence_number(
            env.ledger().sequence() + earnproof_shared::UPGRADE_TIMELOCK_LEDGERS,
        );
        client.upgrade_contract(&hash_v2, &2);

        // Attempting to allowlist version 1 after reaching version 2.
        let res = client.try_approve_upgrade(&bytes(&env, 0x91), &old_hash, &1);
        assert_eq!(res, Err(Ok(ContractError::InvalidInput)));
    }

    // -----------------------------------------------------------------------
    // Event payload tests
    //
    // The Soroban test environment clears the event buffer at the start of each
    // top-level contract invocation (invocation metering is enabled by default
    // in Env::default()). Therefore env.events().all().events() reflects only
    // the events from the most recent invocation. Tests assert on the count
    // returned by a single invocation rather than a before/after diff.
    //
    // Failed invocations produce no contract events (failed_call events are
    // filtered out by all()). The catch_unwind tests confirm this by asserting
    // that a failed call leaves zero success events.
    // -----------------------------------------------------------------------

    /// register_issuer must emit exactly one event on success.
    #[test]
    fn register_issuer_emits_one_event() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 1);
        let metadata_hash = bytes(&env, 2);
        let issuer_address = Address::from_str(&env, ISSUER_ONE);

        client.register_issuer(
            &issuer_id,
            &issuer_address,
            &metadata_hash,
            &bytes(&env, 99),
        );

        assert_eq!(
            env.events().all().events().len(),
            1,
            "expected exactly one event on registration"
        );
    }

    /// Duplicate registration panics before emitting any success event.
    #[test]
    fn register_issuer_failure_emits_no_success_event() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 1);
        let issuer_address = Address::from_str(&env, ISSUER_ONE);

        client.register_issuer(
            &issuer_id,
            &issuer_address,
            &bytes(&env, 2),
            &bytes(&env, 99),
        );

        // Attempt a duplicate — the invocation must panic.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.register_issuer(
                &issuer_id,
                &issuer_address,
                &bytes(&env, 3),
                &bytes(&env, 99),
            );
        }));
        assert!(result.is_err(), "expected panic on duplicate");
        // Failed invocations emit no contract success events.
        assert_eq!(
            env.events().all().events().len(),
            0,
            "no success event should be emitted on a failed registration"
        );
    }

    /// update_issuer emits exactly one event on success.
    #[test]
    fn update_issuer_emits_one_event() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 1);
        let issuer_address = Address::from_str(&env, ISSUER_ONE);
        let new_metadata = bytes(&env, 99);

        client.register_issuer(
            &issuer_id,
            &issuer_address,
            &bytes(&env, 2),
            &bytes(&env, 99),
        );
        client.update_issuer(&issuer_id, &new_metadata);

        assert_eq!(
            env.events().all().events().len(),
            1,
            "expected exactly one event on metadata update"
        );
    }

    /// Updating a revoked issuer panics and emits no success event.
    #[test]
    fn update_revoked_issuer_emits_no_event() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 1);
        let issuer_address = Address::from_str(&env, ISSUER_ONE);

        client.register_issuer(
            &issuer_id,
            &issuer_address,
            &bytes(&env, 2),
            &bytes(&env, 99),
        );
        client.revoke_issuer(
            &bytes(&env, 0x90),
            &issuer_id,
            &soroban_sdk::BytesN::from_array(&client.env, &[1u8; 32]),
        );

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.update_issuer(&issuer_id, &bytes(&env, 99));
        }));
        assert!(result.is_err(), "expected panic on revoked issuer update");
        assert_eq!(
            env.events().all().events().len(),
            0,
            "no success event should be emitted on a failed update"
        );
    }

    /// suspend_issuer emits exactly one event.
    #[test]
    fn suspend_issuer_emits_one_event() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 1);
        let issuer_address = Address::from_str(&env, ISSUER_ONE);

        client.register_issuer(
            &issuer_id,
            &issuer_address,
            &bytes(&env, 2),
            &bytes(&env, 99),
        );
        client.suspend_issuer(
            &bytes(&env, 0x90),
            &issuer_id,
            &soroban_sdk::BytesN::from_array(&client.env, &[1u8; 32]),
        );

        assert_eq!(
            env.events().all().events().len(),
            1,
            "expected exactly one event on suspension"
        );
    }

    /// reactivate_issuer emits exactly one event.
    #[test]
    fn reactivate_issuer_emits_one_event() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 1);
        let issuer_address = Address::from_str(&env, ISSUER_ONE);

        client.register_issuer(
            &issuer_id,
            &issuer_address,
            &bytes(&env, 2),
            &bytes(&env, 99),
        );
        client.suspend_issuer(
            &bytes(&env, 0x90),
            &issuer_id,
            &soroban_sdk::BytesN::from_array(&client.env, &[1u8; 32]),
        );
        client.reactivate_issuer(
            &bytes(&env, 0x91),
            &issuer_id,
            &soroban_sdk::BytesN::from_array(&client.env, &[1u8; 32]),
        );

        assert_eq!(
            env.events().all().events().len(),
            1,
            "expected exactly one event on reactivation"
        );
    }

    /// revoke_issuer emits exactly one event.
    #[test]
    fn revoke_issuer_emits_one_event() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 1);
        let issuer_address = Address::from_str(&env, ISSUER_ONE);

        client.register_issuer(
            &issuer_id,
            &issuer_address,
            &bytes(&env, 2),
            &bytes(&env, 99),
        );
        client.revoke_issuer(
            &bytes(&env, 0x90),
            &issuer_id,
            &soroban_sdk::BytesN::from_array(&client.env, &[1u8; 32]),
        );

        assert_eq!(
            env.events().all().events().len(),
            1,
            "expected exactly one event on revocation"
        );
    }

    /// rotate_issuer_address emits exactly one event containing both old and new addresses.
    #[test]
    fn rotate_address_emits_one_event() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 1);
        let old_address = Address::from_str(&env, ISSUER_ONE);
        let new_address = Address::from_str(&env, ISSUER_TWO);

        client.register_issuer(&issuer_id, &old_address, &bytes(&env, 2), &bytes(&env, 99));
        client.rotate_issuer_address(&issuer_id, &new_address);
        assert_eq!(client.get_issuer(&issuer_id).issuer_address, old_address);
        client.accept_issuer_address_rotation(&issuer_id);
        assert_eq!(client.get_issuer(&issuer_id).issuer_address, new_address);
    }

    /// rotate_issuer_address on a revoked issuer panics and emits no success event.
    #[test]
    fn rotate_revoked_issuer_address_emits_no_event() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 1);
        let old_address = Address::from_str(&env, ISSUER_ONE);
        let new_address = Address::from_str(&env, ISSUER_TWO);

        client.register_issuer(&issuer_id, &old_address, &bytes(&env, 2), &bytes(&env, 99));
        client.revoke_issuer(
            &bytes(&env, 0x90),
            &issuer_id,
            &soroban_sdk::BytesN::from_array(&client.env, &[1u8; 32]),
        );

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.rotate_issuer_address(&issuer_id, &new_address);
        }));
        assert!(result.is_err(), "expected panic on revoked issuer rotation");
        assert_eq!(
            env.events().all().events().len(),
            0,
            "no success event should be emitted on a failed rotation"
        );
    }

    /// Each successful mutation emits exactly one event (full lifecycle).
    /// Each call is checked independently since the event buffer resets per
    /// invocation.
    #[test]
    fn each_mutation_emits_exactly_one_event() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 1);
        let issuer_address = Address::from_str(&env, ISSUER_ONE);
        let new_address = Address::from_str(&env, ISSUER_TWO);

        // register
        client.register_issuer(
            &issuer_id,
            &issuer_address,
            &bytes(&env, 2),
            &bytes(&env, 99),
        );
        assert_eq!(env.events().all().events().len(), 1);

        // update metadata
        client.update_issuer(&issuer_id, &bytes(&env, 3));
        assert_eq!(env.events().all().events().len(), 1);

        // suspend
        client.suspend_issuer(
            &bytes(&env, 0x90),
            &issuer_id,
            &soroban_sdk::BytesN::from_array(&client.env, &[1u8; 32]),
        );
        assert_eq!(env.events().all().events().len(), 1);

        // reactivate
        client.reactivate_issuer(
            &bytes(&env, 0x91),
            &issuer_id,
            &soroban_sdk::BytesN::from_array(&client.env, &[1u8; 32]),
        );
        assert_eq!(env.events().all().events().len(), 1);

        // nominate and accept address rotation
        client.rotate_issuer_address(&issuer_id, &new_address);
        client.accept_issuer_address_rotation(&issuer_id);
        assert_eq!(env.events().all().events().len(), 1);

        // revoke
        client.revoke_issuer(
            &bytes(&env, 0x92),
            &issuer_id,
            &soroban_sdk::BytesN::from_array(&client.env, &[1u8; 32]),
        );
        assert_eq!(env.events().all().events().len(), 1);
    }

    // -----------------------------------------------------------------------
    // Auth mock-parity (#72)
    //
    // Every test above uses env.mock_all_auths() via setup(), which lets any
    // caller through unconditionally — it can never observe that
    // revoke_issuer actually demands the *admin's* signature specifically.
    // This test scopes mock_auths to a real, valid signer that is not the
    // admin (the registered issuer's own address) and asserts the contract's
    // real require_auth(&admin) check rejects it — proving the issuer's own
    // valid signature cannot authorize an admin-only operation on itself.
    // -----------------------------------------------------------------------

    #[test]
    fn revoke_issuer_rejects_a_valid_signature_from_the_issuer_itself() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(IssuerRegistryContract, ());
        let client = IssuerRegistryContractClient::new(&env, &contract_id);
        let admin = Address::from_str(&env, ADMIN);
        client.initialize(&admin);

        // mock_auths (below) registers a stand-in auth contract at each
        // mocked address, so the address must be one the test Env generated
        // itself — a hardcoded G-string constant (like ISSUER_ONE, used by
        // every other test in this module under mock_all_auths()) is not a
        // valid registration target here.
        let issuer_id = bytes(&env, 1);
        let issuer_address = Address::generate(&env);
        client.register_issuer(
            &issuer_id,
            &issuer_address,
            &bytes(&env, 2),
            &bytes(&env, 99),
        );

        // From here on, only the issuer's own signature is authorized for
        // this specific revoke_issuer invocation — not a blanket
        // mock_all_auths(). The issuer's signature is genuinely valid (it is
        // a real, well-formed authorization the host will accept); it is
        // simply for the wrong address. If require_auth(&admin) were ever
        // weakened to accept any authorized caller, this is what would stop
        // silently passing.
        let reason = soroban_sdk::BytesN::from_array(&env, &[1u8; 32]);
        env.mock_auths(&[MockAuth {
            address: &issuer_address,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "revoke_issuer",
                args: (bytes(&env, 0x90), issuer_id.clone(), reason.clone()).into_val(&env),
                sub_invokes: &[],
            },
        }]);

        let result = client.try_revoke_issuer(&bytes(&env, 0x90), &issuer_id, &reason);
        assert!(
            result.is_err(),
            "the issuer's own valid signature must not authorize revoking itself; only the admin's signature may"
        );

        // And unrevoked: the rejected call must not have mutated state.
        assert_eq!(client.get_issuer(&issuer_id).status, IssuerStatus::Active);
    }

    // ── numeric boundary tests ────────────────────────────────────────────────

    /// Contract version boundaries for issuer-registry.
    /// While issuer-registry has no direct numeric user inputs, it does support
    /// contract versioning and upgrade governance. This test covers version boundaries.
    #[test]
    fn contract_version_initialized_and_upgradeable() {
        let (env, client, _admin) = setup();

        // Contract version should be initialized to 1
        assert_eq!(client.get_contract_version(), 1);

        // Valid: upgrade to next version
        client.approve_upgrade(&bytes(&env, 0x90), &bytes(&env, 1), &2);
        assert!(client.is_upgrade_allowed(&bytes(&env, 1)));
    }

    #[test]
    fn contract_version_upgrade_boundaries() {
        let (env, client, _admin) = setup();

        // Valid: immediate next version
        client.approve_upgrade(&bytes(&env, 1), &2);
        env.ledger().set_sequence_number(
            env.ledger().sequence() + earnproof_shared::UPGRADE_TIMELOCK_LEDGERS,
        );
        client.upgrade_contract(&bytes(&env, 1), &2);
        assert_eq!(client.get_contract_version(), 2);

        // Valid: large version number
        client.approve_upgrade(&bytes(&env, 2), &u32::MAX);
        env.ledger().set_sequence_number(
            env.ledger().sequence() + earnproof_shared::UPGRADE_TIMELOCK_LEDGERS,
        );
        client.upgrade_contract(&bytes(&env, 2), &u32::MAX);
        assert_eq!(client.get_contract_version(), u32::MAX);
    }

    #[test]
    fn contract_version_equal_current_rejected() {
        let (env, client, _admin) = setup();
        // Current version is 1; attempting version 1 is rejected
        let res = client.try_approve_upgrade(&bytes(&env, 1), &bytes(&env, 0x90), &1);
        assert_eq!(res, Err(Ok(ContractError::InvalidInput)));
    }

    #[test]
    fn contract_version_below_current_rejected() {
        let (env, client, _admin) = setup();
        // Current version is 1; attempting version 0 is rejected
        let res = client.try_approve_upgrade(&bytes(&env, 1), &bytes(&env, 0x90), &0);
        assert_eq!(res, Err(Ok(ContractError::InvalidInput)));
    }

    /// Test storage invariants: failed boundary cases must not modify state.
    #[test]
    fn failed_upgrade_version_downgrade_leaves_state_unchanged() {
        let (env, client, _admin) = setup();

        let contract_version_before = client.get_contract_version();
        let hash = bytes(&env, 0x88);

        // Attempt to allowlist a downgrade
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.approve_upgrade(&hash, &bytes(&env, 0x90), &0);
        }));

        // Must have panicked
        assert!(result.is_err());

        // Contract version must not change
        assert_eq!(
            client.get_contract_version(),
            contract_version_before,
            "contract version must not change on failed upgrade approval"
        );

        // Hash must not be on allowlist
        assert!(
            !client.is_upgrade_allowed(&hash),
            "failed upgrade approval must not add hash to allowlist"
        );
    }

    // ── adversarial initialization tests ───────────────────────────────────────

    /// Verify that first initialization writes exactly the documented state
    /// with no partial writes or missing fields.
    ///
    /// Required behavior: First call to `initialize` results in:
    /// - Admin address set and readable
    /// - ContractVersion = 1
    #[test]
    fn initialization_writes_exactly_documented_state() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(IssuerRegistryContract, ());
        let client = IssuerRegistryContractClient::new(&env, &contract_id);
        let admin = Address::from_str(&env, ADMIN);

        // Perform initialization
        client.initialize(&admin);

        // Verify exact state written
        assert_eq!(client.get_admin(), admin, "admin must be set");
        assert_eq!(
            client.get_contract_version(),
            1,
            "contract version must be exactly 1 after initialization"
        );

        // Verify storage keys are set
        env.as_contract(&contract_id, || {
            let instance = env.storage().instance();
            assert!(
                instance.has(&DataKey::Admin),
                "Admin key must exist in instance storage"
            );
            assert!(
                instance.has(&DataKey::ContractVersion),
                "ContractVersion key must exist in instance storage"
            );
        });
    }

    /// Verify that repeated initialization by any address fails without
    /// altering state or emitting events.
    ///
    /// Required behavior for re-initialization guard:
    /// - Second call to `initialize` with any admin (same or different) panics
    /// - Storage is byte-for-byte unchanged
    /// - No additional events are emitted
    #[test]
    fn reinitialization_by_same_admin_fails_atomically() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(IssuerRegistryContract, ());
        let client = IssuerRegistryContractClient::new(&env, &contract_id);
        let admin = Address::from_str(&env, ADMIN);

        // First initialization succeeds
        client.initialize(&admin);
        let contract_version_after_first = client.get_contract_version();

        // Attempt second initialization with same admin
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.initialize(&admin);
        }));

        // Must have panicked with "already initialized"
        assert!(result.is_err(), "re-initialization must panic");

        // Verify state is byte-for-byte identical
        assert_eq!(
            client.get_admin(),
            admin,
            "admin must not change after failed re-initialization"
        );
        assert_eq!(
            client.get_contract_version(),
            contract_version_after_first,
            "contract version must not change after failed re-initialization"
        );
    }

    /// Verify that re-initialization by a different address also fails
    /// without state or event changes.
    ///
    /// This tests that the re-initialization guard does not discriminate
    /// based on caller identity — it prevents any re-initialization attempt.
    #[test]
    fn reinitialization_by_different_admin_fails_atomically() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(IssuerRegistryContract, ());
        let client = IssuerRegistryContractClient::new(&env, &contract_id);
        let admin = Address::from_str(&env, ADMIN);
        let other = Address::from_str(&env, ISSUER_ONE);

        // First initialization with original admin
        client.initialize(&admin);
        let stored_admin = client.get_admin();
        let contract_version_after_first = client.get_contract_version();

        // Attempt re-initialization with different admin
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.initialize(&other);
        }));

        // Must have panicked
        assert!(
            result.is_err(),
            "re-initialization by different admin must panic"
        );

        // Verify state is unchanged: original admin must still be stored
        assert_eq!(
            client.get_admin(),
            stored_admin,
            "admin must not change when different address attempts re-initialization"
        );
        assert_eq!(
            client.get_contract_version(),
            contract_version_after_first,
            "contract version must not change after failed re-initialization by different admin"
        );
    }

    /// Verify that the re-initialization guard does not allow partial state
    /// modification on subsequent initialization attempts.
    #[test]
    fn reinitialization_guard_is_absolute() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(IssuerRegistryContract, ());
        let client = IssuerRegistryContractClient::new(&env, &contract_id);
        let admin = Address::from_str(&env, ADMIN);

        // First initialization
        client.initialize(&admin);

        // Multiple re-initialization attempts must all fail
        for attempt in 1..=3 {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                client.initialize(&admin);
            }));

            assert!(
                result.is_err(),
                "re-initialization attempt {} must fail",
                attempt
            );

            // Admin must remain unchanged
            assert_eq!(
                client.get_admin(),
                admin,
                "admin must not change after re-initialization attempt {}",
                attempt
            );
        }
    }

    /// Verify that initialization state is maintained across subsequent
    /// issuer registration and upgrade operations.
    ///
    /// Tests that the initialization state (admin, contract version) is stable
    /// and correct before and after other contract operations.
    #[test]
    fn initialization_state_stable_across_operations() {
        let (env, client, admin) = setup();

        // State immediately after initialization
        assert_eq!(client.get_admin(), admin);
        assert_eq!(client.get_contract_version(), 1);

        // Perform issuer registration
        let issuer_id = bytes(&env, 1);
        let issuer_address = Address::from_str(&env, ISSUER_ONE);
        client.register_issuer(
            &issuer_id,
            &issuer_address,
            &bytes(&env, 2),
            &bytes(&env, 99),
        );

        // Admin must remain unchanged
        assert_eq!(
            client.get_admin(),
            admin,
            "admin must not change after issuer registration"
        );
        // Contract version must still be 1 (no upgrade yet)
        assert_eq!(
            client.get_contract_version(),
            1,
            "contract version must not change on issuer registration"
        );
    }

    /// Summary test: issuer-registry initialization spec verification.
    ///
    /// This test serves as executable documentation of what the test matrix
    /// expects from issuer-registry initialization:
    /// - Standalone contract (no dependency addresses)
    /// - Has re-initialization guard
    /// - Does NOT emit an event during initialization
    /// - Sets: admin, contract_version=1
    #[test]
    fn issuer_registry_initialization_spec_summary() {
        // CONTRACT SPEC: issuer-registry
        // - Name: "issuer-registry"
        // - Has re-initialization guard: YES (panics "already initialized")
        // - Emits initialization event: NO
        // - Takes dependency addresses: NO
        // - Dependencies: []
        // - First init writes:
        //   - Admin: passed address (requires auth)
        //   - ContractVersion: 1
        // - Re-init guard: DataKey::Admin presence check; panics if set
        // - Re-init allowed by different admin: NO (guard blocks all)
        // - Invalid config cases: None (no dependencies to validate)

        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(IssuerRegistryContract, ());
        let client = IssuerRegistryContractClient::new(&env, &contract_id);
        let admin = Address::from_str(&env, ADMIN);

        // Verify the spec
        client.initialize(&admin);
        assert_eq!(client.get_admin(), admin);
        assert_eq!(client.get_contract_version(), 1);

        // Re-initialization must fail
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.initialize(&admin)
        }))
        .is_err());
    }

    #[test]
    fn configuration_digest_matches_host_helper_and_version_changes() {
        let (env, client, admin) = setup();
        let initial = client.get_config_digest();
        assert_eq!(
            IssuerRegistryContractClient::get_config_digest_version(&client),
            earnproof_shared::CONFIG_DIGEST_VERSION
        );
        assert_eq!(
            initial,
            earnproof_shared::issuer_registry_digest(&env, &admin, 1)
        );

        let wasm_hash = bytes(&env, 0xd1);
        client.approve_upgrade(&wasm_hash, &2);
        env.ledger().set_sequence_number(
            env.ledger().sequence() + earnproof_shared::UPGRADE_TIMELOCK_LEDGERS,
        );
        client.upgrade_contract(&wasm_hash, &2);
        assert_ne!(client.get_config_digest(), initial);
    }

    #[test]
    fn ttl_status_tracks_only_caller_named_issuer_entries() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 0xe1);
        let unknown_id = bytes(&env, 0xe2);
        let issuer_address = Address::from_str(&env, ISSUER_ONE);

        assert_eq!(
            client.get_instance_ttl_status().health,
            earnproof_shared::TtlHealth::Healthy
        );
        assert_eq!(
            client.get_issuer_ttl_status(&unknown_id).health,
            earnproof_shared::TtlHealth::Missing
        );
        client.register_issuer(
            &issuer_id,
            &issuer_address,
            &bytes(&env, 0xe3),
            &bytes(&env, 0x99),
        );
        assert_eq!(
            client.get_issuer_ttl_status(&issuer_id).health,
            earnproof_shared::TtlHealth::Healthy
        );
        assert_eq!(
            client.get_address_ttl_status(&issuer_address).health,
            earnproof_shared::TtlHealth::Healthy
        );
    }

    // ── genesis identity (issue #192) ────────────────────────────────────────

    #[test]
    fn genesis_is_recorded_at_initialization() {
        let (env, client, _admin) = setup();
        let genesis = client.get_genesis();
        assert_ne!(genesis.genesis_id, BytesN::from_array(&env, &[0u8; 32]));
        assert_eq!(genesis.initialized_at_ledger, env.ledger().sequence());
    }

    #[test]
    fn genesis_id_differs_across_contract_instances() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::from_str(&env, ADMIN);

        let a = env.register(IssuerRegistryContract, ());
        let a = IssuerRegistryContractClient::new(&env, &a);
        a.initialize(&admin);

        let b = env.register(IssuerRegistryContract, ());
        let b = IssuerRegistryContractClient::new(&env, &b);
        b.initialize(&admin);

        assert_ne!(a.get_genesis().genesis_id, b.get_genesis().genesis_id);
    }

    #[test]
    fn genesis_id_is_deterministic_for_the_same_inputs() {
        let (env, client, _admin) = setup();
        let recomputed = env.as_contract(&client.address, || {
            earnproof_shared::compute_genesis_id(&env, "earnproof_issuer_registry")
        });
        assert_eq!(client.get_genesis().genesis_id, recomputed);
    }

    #[test]
    fn genesis_is_stable_across_unrelated_mutations() {
        let (env, client, _admin) = setup();
        let genesis_before = client.get_genesis();

        client.register_issuer(
            &bytes(&env, 1),
            &Address::from_str(&env, ISSUER_ONE),
            &bytes(&env, 2),
            &bytes(&env, 99),
        );

        assert_eq!(client.get_genesis(), genesis_before);
    }

    #[test]
    fn get_genesis_fails_before_initialization() {
        let env = Env::default();
        let contract_id = env.register(IssuerRegistryContract, ());
        let client = IssuerRegistryContractClient::new(&env, &contract_id);
        use earnproof_shared::ContractError;

        let result = client.try_get_genesis();
        assert_eq!(result, Err(Ok(ContractError::NotInitialized)));
    }

    // ── issuer metadata URI hash commitments (issue 179) ───────────────────────

    #[test]
    fn register_issuer_initializes_metadata_commitments() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 1);
        let issuer_address = Address::from_str(&env, ISSUER_ONE);
        client.register_issuer(
            &issuer_id,
            &issuer_address,
            &bytes(&env, 2),
            &bytes(&env, 99),
        );
        let record = client.get_issuer(&issuer_id);
        assert_eq!(record.metadata_hash, bytes(&env, 2));
        // The URI commitment starts at the all-zero "unset" sentinel.
        assert_eq!(record.metadata_uri_hash, bytes(&env, 0));
        assert_eq!(record.metadata_revision, 1);
    }

    #[test]
    fn set_metadata_commitment_updates_both_and_increments_revision() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 1);
        let issuer_address = Address::from_str(&env, ISSUER_ONE);
        client.register_issuer(
            &issuer_id,
            &issuer_address,
            &bytes(&env, 2),
            &bytes(&env, 99),
        );

        let content = bytes(&env, 0x11);
        let uri = bytes(&env, 0x22);
        client.set_issuer_metadata_commitment(&issuer_id, &content, &uri);

        let record = client.get_issuer(&issuer_id);
        // Golden-vector parity: the contract stores the exact bytes supplied,
        // treating them as opaque commitments (no re-hashing).
        assert_eq!(record.metadata_hash, content);
        assert_eq!(record.metadata_uri_hash, uri);
        assert_eq!(record.metadata_revision, 2);
    }

    #[test]
    fn set_metadata_commitment_emits_exactly_one_event() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 1);
        let issuer_address = Address::from_str(&env, ISSUER_ONE);
        client.register_issuer(
            &issuer_id,
            &issuer_address,
            &bytes(&env, 2),
            &bytes(&env, 99),
        );
        client.set_issuer_metadata_commitment(&issuer_id, &bytes(&env, 0x11), &bytes(&env, 0x22));
        assert_eq!(env.events().all().events().len(), 1);
    }

    #[test]
    fn set_metadata_commitment_rejects_empty_content_commitment() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 1);
        let issuer_address = Address::from_str(&env, ISSUER_ONE);
        client.register_issuer(
            &issuer_id,
            &issuer_address,
            &bytes(&env, 2),
            &bytes(&env, 99),
        );
        let result = client.try_set_issuer_metadata_commitment(
            &issuer_id,
            &bytes(&env, 0),
            &bytes(&env, 0x22),
        );
        assert_eq!(result, Err(Ok(IssuerError::InvalidMetadataCommitment)));
    }

    #[test]
    fn set_metadata_commitment_rejects_empty_uri_commitment() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 1);
        let issuer_address = Address::from_str(&env, ISSUER_ONE);
        client.register_issuer(
            &issuer_id,
            &issuer_address,
            &bytes(&env, 2),
            &bytes(&env, 99),
        );
        let result = client.try_set_issuer_metadata_commitment(
            &issuer_id,
            &bytes(&env, 0x11),
            &bytes(&env, 0),
        );
        assert_eq!(result, Err(Ok(IssuerError::InvalidMetadataCommitment)));
    }

    #[test]
    fn set_metadata_commitment_rejects_unknown_issuer() {
        let (env, client, _admin) = setup();
        let result = client.try_set_issuer_metadata_commitment(
            &bytes(&env, 7),
            &bytes(&env, 0x11),
            &bytes(&env, 0x22),
        );
        assert_eq!(result, Err(Ok(IssuerError::IssuerNotFound)));
    }

    #[test]
    fn set_metadata_commitment_rejects_revoked_issuer() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 1);
        let issuer_address = Address::from_str(&env, ISSUER_ONE);
        client.register_issuer(
            &issuer_id,
            &issuer_address,
            &bytes(&env, 2),
            &bytes(&env, 99),
        );
        client.revoke_issuer(&issuer_id, &bytes(&env, 98));
        let result = client.try_set_issuer_metadata_commitment(
            &issuer_id,
            &bytes(&env, 0x11),
            &bytes(&env, 0x22),
        );
        assert_eq!(result, Err(Ok(IssuerError::IssuerRevoked)));
    }

    #[test]
    fn update_issuer_increments_metadata_revision() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 1);
        let issuer_address = Address::from_str(&env, ISSUER_ONE);
        client.register_issuer(
            &issuer_id,
            &issuer_address,
            &bytes(&env, 2),
            &bytes(&env, 99),
        );
        client.update_issuer(&issuer_id, &bytes(&env, 3));
        let record = client.get_issuer(&issuer_id);
        assert_eq!(record.metadata_hash, bytes(&env, 3));
        assert_eq!(record.metadata_revision, 2);
        // update_issuer leaves the URI commitment untouched.
        assert_eq!(record.metadata_uri_hash, bytes(&env, 0));
    }

    // ── issuer status effective ledger metadata (issue 180) ────────────────────

    #[test]
    fn register_issuer_records_status_effective_metadata() {
        let (env, client, _admin) = setup();
        env.ledger().with_mut(|li| {
            li.sequence_number = 100;
            li.timestamp = 555;
        });
        let issuer_id = bytes(&env, 1);
        let issuer_address = Address::from_str(&env, ISSUER_ONE);
        client.register_issuer(
            &issuer_id,
            &issuer_address,
            &bytes(&env, 2),
            &bytes(&env, 99),
        );
        let record = client.get_issuer(&issuer_id);
        assert_eq!(record.status_effective_ledger, 100);
        assert_eq!(record.status_effective_timestamp, 555);
    }

    #[test]
    fn suspend_updates_status_effective_metadata_atomically() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 1);
        let issuer_address = Address::from_str(&env, ISSUER_ONE);
        client.register_issuer(
            &issuer_id,
            &issuer_address,
            &bytes(&env, 2),
            &bytes(&env, 99),
        );

        env.ledger().with_mut(|li| {
            li.sequence_number = 900;
            li.timestamp = 9_000;
        });
        client.suspend_issuer(&issuer_id, &bytes(&env, 98));
        let record = client.get_issuer(&issuer_id);
        assert_eq!(record.status, IssuerStatus::Suspended);
        assert_eq!(record.status_effective_ledger, 900);
        assert_eq!(record.status_effective_timestamp, 9_000);
    }

    #[test]
    fn each_transition_records_its_own_effective_ledger() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 1);
        let issuer_address = Address::from_str(&env, ISSUER_ONE);
        client.register_issuer(
            &issuer_id,
            &issuer_address,
            &bytes(&env, 2),
            &bytes(&env, 99),
        );
        let reason = soroban_sdk::BytesN::from_array(&client.env, &[1u8; 32]);

        env.ledger().with_mut(|li| {
            li.sequence_number = 10;
            li.timestamp = 100;
        });
        client.suspend_issuer(&issuer_id, &bytes(&env, 98));
        assert_eq!(client.get_issuer(&issuer_id).status_effective_ledger, 10);

        env.ledger().with_mut(|li| {
            li.sequence_number = 20;
            li.timestamp = 200;
        });
        client.reactivate_issuer(&issuer_id, &bytes(&env, 98));
        assert_eq!(client.get_issuer(&issuer_id).status_effective_ledger, 20);

        env.ledger().with_mut(|li| {
            li.sequence_number = 30;
            li.timestamp = 300;
        });
        client.revoke_issuer(&issuer_id, &bytes(&env, 98));
        let record = client.get_issuer(&issuer_id);
        assert_eq!(record.status, IssuerStatus::Revoked);
        assert_eq!(record.status_effective_ledger, 30);
        assert_eq!(record.status_effective_timestamp, 300);
    }

    #[test]
    fn failed_transition_leaves_effective_metadata_unchanged() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 1);
        let issuer_address = Address::from_str(&env, ISSUER_ONE);
        client.register_issuer(
            &issuer_id,
            &issuer_address,
            &bytes(&env, 2),
            &bytes(&env, 99),
        );
        env.ledger().with_mut(|li| {
            li.sequence_number = 40;
            li.timestamp = 400;
        });
        client.revoke_issuer(&issuer_id, &bytes(&env, 98));
        let before = client.get_issuer(&issuer_id);

        // A revoked issuer cannot be reactivated; the rejected call must not
        // touch the effective metadata.
        env.ledger().with_mut(|li| {
            li.sequence_number = 50;
            li.timestamp = 500;
        });
        let result = client.try_reactivate_issuer(&issuer_id, &bytes(&env, 98));
        assert_eq!(result, Err(Ok(IssuerError::InvalidTransition)));
        let after = client.get_issuer(&issuer_id);
        assert_eq!(
            after.status_effective_ledger,
            before.status_effective_ledger
        );
        assert_eq!(
            after.status_effective_timestamp,
            before.status_effective_timestamp
        );
    }

    #[test]
    fn issuer_rotation_requires_acceptance_and_can_be_cancelled() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 0x31);
        let old_address = Address::from_str(&env, ISSUER_ONE);
        let new_address = Address::from_str(&env, ISSUER_TWO);
        client.register_issuer(&issuer_id, &old_address, &bytes(&env, 2), &bytes(&env, 99));
        client.rotate_issuer_address(&issuer_id, &new_address);

        let pending = client.get_pending_issuer_rotation(&issuer_id).unwrap();
        assert_eq!(pending.old_address, old_address);
        assert_eq!(pending.new_address, new_address);
        assert_eq!(client.get_issuer(&issuer_id).issuer_address, old_address);
        assert!(client.is_active_address(&old_address));
        assert!(!client.is_active_address(&new_address));

        client.cancel_issuer_address_rotation(&issuer_id);
        assert!(client.get_pending_issuer_rotation(&issuer_id).is_none());
        assert_eq!(client.get_issuer(&issuer_id).issuer_address, old_address);
    }

    #[test]
    fn only_the_replacement_address_can_accept_rotation() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(IssuerRegistryContract, ());
        let client = IssuerRegistryContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let old_address = Address::generate(&env);
        let new_address = Address::generate(&env);
        let issuer_id = bytes(&env, 0x30);
        client.initialize(&admin);
        client.register_issuer(&issuer_id, &old_address, &bytes(&env, 2), &bytes(&env, 99));
        client.rotate_issuer_address(&issuer_id, &new_address);

        env.mock_auths(&[MockAuth {
            address: &old_address,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "accept_issuer_address_rotation",
                args: (issuer_id.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        assert!(client
            .try_accept_issuer_address_rotation(&issuer_id)
            .is_err());
        assert_eq!(client.get_issuer(&issuer_id).issuer_address, old_address);
    }

    #[test]
    fn issuer_rotation_expiry_boundary_and_status_change_prevent_acceptance() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 0x32);
        let old_address = Address::from_str(&env, ISSUER_ONE);
        let new_address = Address::from_str(&env, ISSUER_TWO);
        client.register_issuer(&issuer_id, &old_address, &bytes(&env, 2), &bytes(&env, 99));
        client.rotate_issuer_address(&issuer_id, &new_address);
        let expiry = client
            .get_pending_issuer_rotation(&issuer_id)
            .unwrap()
            .expires_at_ledger;
        env.ledger()
            .with_mut(|ledger| ledger.sequence_number = expiry);
        assert_eq!(
            client.try_accept_issuer_address_rotation(&issuer_id),
            Err(Ok(IssuerError::InvalidTransition))
        );

        let second_id = bytes(&env, 0x33);
        let second_address = Address::generate(&env);
        client.register_issuer(
            &second_id,
            &second_address,
            &bytes(&env, 2),
            &bytes(&env, 99),
        );
        client.rotate_issuer_address(&second_id, &new_address);
        client.suspend_issuer(&bytes(&env, 0x34), &second_id, &bytes(&env, 1));
        assert!(client.get_pending_issuer_rotation(&second_id).is_none());
        assert_eq!(
            client.try_accept_issuer_address_rotation(&second_id),
            Err(Ok(IssuerError::InvalidTransition))
        );

        let third_id = bytes(&env, 0x3B);
        let third_address = Address::generate(&env);
        client.register_issuer(&third_id, &third_address, &bytes(&env, 4), &bytes(&env, 97));
        client.rotate_issuer_address(&third_id, &Address::generate(&env));
        client.revoke_issuer(&bytes(&env, 0x3C), &third_id, &bytes(&env, 1));
        assert!(client.get_pending_issuer_rotation(&third_id).is_none());
    }

    #[test]
    fn issuer_rotation_acceptance_moves_address_and_rejects_replay() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 0x35);
        let old_address = Address::from_str(&env, ISSUER_ONE);
        let new_address = Address::from_str(&env, ISSUER_TWO);
        client.register_issuer(&issuer_id, &old_address, &bytes(&env, 2), &bytes(&env, 99));
        client.rotate_issuer_address(&issuer_id, &new_address);
        client.accept_issuer_address_rotation(&issuer_id);
        assert_eq!(client.get_issuer(&issuer_id).issuer_address, new_address);
        assert!(!client.is_active_address(&old_address));
        assert!(client.is_active_address(&new_address));
        assert_eq!(
            client.try_accept_issuer_address_rotation(&issuer_id),
            Err(Ok(IssuerError::InvalidTransition))
        );
    }

    #[test]
    fn expiring_issuer_management_role_authorizes_only_inside_its_window() {
        let (env, client, _admin) = setup();
        let delegate = Address::generate(&env);
        let issuer_id = bytes(&env, 0x36);
        let issuer_address = Address::from_str(&env, ISSUER_ONE);
        client.register_issuer(
            &issuer_id,
            &issuer_address,
            &bytes(&env, 2),
            &bytes(&env, 99),
        );
        let activation = env.ledger().sequence();
        let expiration = activation + 2;
        client.grant_governance_role(
            &bytes(&env, 0x37),
            &GovernanceRole::IssuerManagement,
            &delegate,
            &activation,
            &Some(expiration),
        );
        let assignment = client
            .get_governance_assignment(&GovernanceRole::IssuerManagement, &delegate)
            .unwrap();
        assert!(assignment.is_active_at(activation));
        assert!(!assignment.is_pending_at(activation));
        assert_eq!(
            client.governance_role_state(&GovernanceRole::IssuerManagement, &delegate),
            GovernanceRoleState::Active
        );
        client.suspend_issuer_by_role(&bytes(&env, 0x38), &issuer_id, &bytes(&env, 1), &delegate);

        let second_id = bytes(&env, 0x39);
        let second_address = Address::from_str(&env, ISSUER_TWO);
        client.register_issuer(
            &second_id,
            &second_address,
            &bytes(&env, 3),
            &bytes(&env, 98),
        );
        env.ledger()
            .with_mut(|ledger| ledger.sequence_number = expiration);
        assert_eq!(
            client.governance_role_state(&GovernanceRole::IssuerManagement, &delegate),
            GovernanceRoleState::Expired
        );
        assert_eq!(
            client.try_suspend_issuer_by_role(
                &bytes(&env, 0x3A),
                &second_id,
                &bytes(&env, 1),
                &delegate,
            ),
            Err(Ok(IssuerError::InvalidTransition))
        );
    }

    #[test]
    fn compromised_active_key_is_removed_and_cannot_be_reused() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 0x60);
        let issuer_address = Address::from_str(&env, ISSUER_ONE);
        let key_hash = bytes(&env, 0x61);
        client.register_issuer(
            &issuer_id,
            &issuer_address,
            &bytes(&env, 2),
            &bytes(&env, 99),
        );
        client.propose_issuer_signing_key(&issuer_id, &key_hash, &1);
        client.activate_issuer_signing_key(&issuer_id);

        env.ledger().with_mut(|ledger| ledger.sequence_number = 700);
        let record =
            client.mark_signing_key_compromised(&issuer_id, &key_hash, &1, &bytes(&env, 0x62));
        assert_eq!(record.record_version, 1);
        assert_eq!(record.effective_ledger, 700);
        assert_eq!(record.sequence, 0);
        assert!(client.get_active_issuer_signing_key(&issuer_id).is_none());
        assert_eq!(
            client.try_propose_issuer_signing_key(&issuer_id, &key_hash, &1),
            Err(Ok(IssuerError::InvalidTransition))
        );
        assert_eq!(
            client.try_mark_signing_key_compromised(&issuer_id, &key_hash, &1, &bytes(&env, 0x63),),
            Err(Ok(IssuerError::InvalidTransition))
        );
    }

    #[test]
    fn compromised_history_is_bounded_and_unknown_keys_fail_atomically() {
        let (env, client, _admin) = setup();
        let issuer_id = bytes(&env, 0x64);
        client.register_issuer(
            &issuer_id,
            &Address::from_str(&env, ISSUER_ONE),
            &bytes(&env, 2),
            &bytes(&env, 99),
        );
        let first = bytes(&env, 0x65);
        let second = bytes(&env, 0x66);
        client.propose_issuer_signing_key(&issuer_id, &first, &1);
        client.activate_issuer_signing_key(&issuer_id);
        client.propose_issuer_signing_key(&issuer_id, &second, &1);
        client.activate_issuer_signing_key(&issuer_id);
        client.mark_signing_key_compromised(&issuer_id, &first, &1, &bytes(&env, 0x67));

        let records = client.get_compromised_key_history(&issuer_id, &0, &1);
        assert_eq!(records.len(), 1);
        assert_eq!(records.get(0).unwrap().key_hash, first);
        assert_eq!(
            client.try_get_compromised_key_history(&issuer_id, &0, &21),
            Err(Ok(IssuerError::BatchTooLarge))
        );
        assert_eq!(
            client.try_mark_signing_key_compromised(
                &issuer_id,
                &bytes(&env, 0x68),
                &1,
                &bytes(&env, 0x69),
            ),
            Err(Ok(IssuerError::InvalidTransition))
        );
        assert!(client.get_active_issuer_signing_key(&issuer_id).is_some());
    }

    #[test]
    fn only_admin_can_mark_a_signing_key_compromised() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(IssuerRegistryContract, ());
        let client = IssuerRegistryContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let outsider = Address::generate(&env);
        let issuer = Address::generate(&env);
        let issuer_id = bytes(&env, 0x70);
        let key_hash = bytes(&env, 0x71);
        client.initialize(&admin);
        client.register_issuer(&issuer_id, &issuer, &bytes(&env, 2), &bytes(&env, 99));
        client.propose_issuer_signing_key(&issuer_id, &key_hash, &1);

        env.mock_auths(&[MockAuth {
            address: &outsider,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "mark_signing_key_compromised",
                args: (issuer_id.clone(), key_hash.clone(), 1u32, bytes(&env, 0x72)).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        assert!(client
            .try_mark_signing_key_compromised(&issuer_id, &key_hash, &1, &bytes(&env, 0x72),)
            .is_err());
        assert!(client
            .get_compromised_signing_key(&issuer_id, &key_hash, &1)
            .is_none());
    }
}

#[cfg(test)]
mod upgrade_timelock_tests {
    extern crate std;

    use super::{DataKey, IssuerRegistryContract, IssuerRegistryContractClient};
    use earnproof_shared::{
        ContractError, UpgradeApproval, UPGRADE_APPROVAL_EXPIRY_LEDGERS, UPGRADE_TIMELOCK_LEDGERS,
    };
    use soroban_sdk::{testutils::Ledger as _, Address, BytesN, Env};

    const ADMIN: &str = "GCFIRY65OQE7DFP5KLNS2PF2LVZMUZYJX4OZIEQ36N2IQANUB5XVYOJR";

    fn make_wasm_hash(env: &Env) -> BytesN<32> {
        BytesN::from_array(env, &[1u8; 32])
    }

    fn setup() -> (Env, IssuerRegistryContractClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(IssuerRegistryContract, ());
        let client = IssuerRegistryContractClient::new(&env, &contract_id);
        let admin = Address::from_str(&env, ADMIN);
        client.initialize(&admin);
        (env, client, admin)
    }

    // ── SUITE 1: Approve stores correct timing ────────────────

    #[test]
    fn test_approve_stores_created_at() {
        let (env, client, _) = setup();

        let start_ledger = env.ledger().sequence();

        client.approve_upgrade(
            &BytesN::from_array(&env, &[1u8; 32]),
            &make_wasm_hash(&env),
            &2,
        );

        env.as_contract(&client.address, || {
            let approval: Option<UpgradeApproval> =
                env.storage().instance().get(&DataKey::UpgradeApproval);
            assert!(approval.is_some(), "Approval must be stored");
            assert_eq!(approval.unwrap().created_at, start_ledger);
        });
    }

    #[test]
    fn test_approve_stores_earliest_execution() {
        let (env, client, _) = setup();

        let start = env.ledger().sequence();

        client.approve_upgrade(
            &BytesN::from_array(&env, &[1u8; 32]),
            &make_wasm_hash(&env),
            &2,
        );

        env.as_contract(&client.address, || {
            let approval: Option<UpgradeApproval> =
                env.storage().instance().get(&DataKey::UpgradeApproval);
            assert_eq!(
                approval.unwrap().earliest_execution,
                start.saturating_add(UPGRADE_TIMELOCK_LEDGERS)
            );
        });
    }

    #[test]
    fn test_approve_stores_expires_at() {
        let (env, client, _) = setup();

        let start = env.ledger().sequence();

        client.approve_upgrade(
            &BytesN::from_array(&env, &[1u8; 32]),
            &make_wasm_hash(&env),
            &2,
        );

        env.as_contract(&client.address, || {
            let approval: Option<UpgradeApproval> =
                env.storage().instance().get(&DataKey::UpgradeApproval);
            assert_eq!(
                approval.unwrap().expires_at,
                start.saturating_add(UPGRADE_APPROVAL_EXPIRY_LEDGERS)
            );
        });
    }

    #[test]
    fn test_re_approval_resets_all_timing_metadata() {
        let (env, client, _) = setup();

        // First approval
        client.approve_upgrade(
            &BytesN::from_array(&env, &[1u8; 32]),
            &make_wasm_hash(&env),
            &2,
        );

        let (first_created, first_earliest, first_expires) =
            env.as_contract(&client.address, || {
                let approval: Option<UpgradeApproval> =
                    env.storage().instance().get(&DataKey::UpgradeApproval);
                let approval_ref = approval.as_ref().unwrap();
                (
                    approval_ref.created_at,
                    approval_ref.earliest_execution,
                    approval_ref.expires_at,
                )
            });

        // Advance ledger
        env.ledger()
            .set_sequence_number(env.ledger().sequence() + 1000);

        // Re-approve (outside as_contract closure)
        client.approve_upgrade(
            &BytesN::from_array(&env, &[2u8; 32]),
            &make_wasm_hash(&env),
            &2,
        );

        env.as_contract(&client.address, || {
            let approval2: Option<UpgradeApproval> =
                env.storage().instance().get(&DataKey::UpgradeApproval);
            let approval2 = approval2.unwrap();

            assert_ne!(
                approval2.created_at, first_created,
                "Re-approval must reset created_at (no stale reuse)"
            );
            assert_ne!(
                approval2.earliest_execution, first_earliest,
                "Re-approval must reset earliest_execution"
            );
            assert_ne!(
                approval2.expires_at, first_expires,
                "Re-approval must reset expires_at"
            );
        });
    }

    // ── SUITE 2: Timelock enforcement ─────────────────────────

    #[test]
    fn test_execute_before_timelock_rejected() {
        let (env, client, _) = setup();

        let hash = make_wasm_hash(&env);

        client.approve_upgrade(&hash, &BytesN::from_array(&env, &[1u8; 32]), &2);

        // Try immediately (before timelock)
        let result = client.try_upgrade_contract(&hash, &2);

        assert_eq!(
            result,
            Err(Ok(ContractError::UpgradeTimelockNotElapsed)),
            "Execute before timelock must be rejected"
        );
    }

    #[test]
    fn test_execute_exactly_at_timelock_succeeds() {
        let (env, client, _) = setup();

        let hash = make_wasm_hash(&env);

        client.approve_upgrade(&hash, &BytesN::from_array(&env, &[1u8; 32]), &2);

        // Advance to exactly earliest_execution
        env.ledger()
            .set_sequence_number(env.ledger().sequence() + UPGRADE_TIMELOCK_LEDGERS);

        let result = client.try_upgrade_contract(&hash, &2);

        assert!(result.is_ok(), "Execute at earliest_execution must succeed");
    }

    #[test]
    fn test_execute_one_before_timelock_rejected() {
        let (env, client, _) = setup();

        let hash = make_wasm_hash(&env);

        client.approve_upgrade(&hash, &BytesN::from_array(&env, &[1u8; 32]), &2);

        // One ledger before timelock
        env.ledger()
            .set_sequence_number(env.ledger().sequence() + UPGRADE_TIMELOCK_LEDGERS - 1);

        assert!(client.try_upgrade_contract(&hash, &2).is_err());
    }

    // ── SUITE 3: Expiry enforcement ────────────────────────────

    #[test]
    fn test_execute_after_expiry_rejected() {
        let (env, client, _) = setup();

        let hash = make_wasm_hash(&env);

        client.approve_upgrade(&hash, &BytesN::from_array(&env, &[1u8; 32]), &2);

        // Advance past expiry
        env.ledger()
            .set_sequence_number(env.ledger().sequence() + UPGRADE_APPROVAL_EXPIRY_LEDGERS + 1);

        let result = client.try_upgrade_contract(&hash, &2);

        assert_eq!(
            result,
            Err(Ok(ContractError::UpgradeApprovalExpired)),
            "Execute after expiry must be rejected"
        );
    }

    #[test]
    fn test_execute_exactly_at_expiry_rejected() {
        let (env, client, _) = setup();

        let hash = make_wasm_hash(&env);

        client.approve_upgrade(&hash, &BytesN::from_array(&env, &[1u8; 32]), &2);

        // Advance to exactly expires_at
        env.ledger()
            .set_sequence_number(env.ledger().sequence() + UPGRADE_APPROVAL_EXPIRY_LEDGERS);

        let result = client.try_upgrade_contract(&hash, &2);

        assert_eq!(
            result,
            Err(Ok(ContractError::UpgradeApprovalExpired)),
            "Execute at exact expiry ledger must be rejected (>=)"
        );
    }

    #[test]
    fn test_failed_execute_leaves_approval_unchanged() {
        let (env, client, _) = setup();

        let hash = make_wasm_hash(&env);

        client.approve_upgrade(&hash, &BytesN::from_array(&env, &[1u8; 32]), &2);

        // Attempt execute before timelock (fails)
        let _ = client.try_upgrade_contract(&hash, &2);

        // Approval must still exist unchanged
        env.as_contract(&client.address, || {
            let approval: Option<UpgradeApproval> =
                env.storage().instance().get(&DataKey::UpgradeApproval);
            assert!(
                approval.is_some(),
                "Failed execute must not remove approval"
            );
        });
    }

    // ── SUITE 4: Revocation ────────────────────────────────────

    #[test]
    fn test_revoke_removes_approval() {
        let (env, client, _) = setup();

        client.approve_upgrade(&make_wasm_hash(&env), &2);

        client.revoke_upgrade_approval();

        env.as_contract(&client.address, || {
            let approval: Option<UpgradeApproval> =
                env.storage().instance().get(&DataKey::UpgradeApproval);
            assert!(approval.is_none(), "Revoke must remove approval");
        });
    }

    #[test]
    fn test_revoke_before_timelock_succeeds() {
        let (env, client, _) = setup();

        client.approve_upgrade(&make_wasm_hash(&env), &2);

        // Revoke immediately (before timelock)
        assert!(client.try_revoke_upgrade_approval().is_ok());
    }

    #[test]
    fn test_revoke_after_expiry_succeeds_cleanup() {
        let (env, client, _) = setup();

        client.approve_upgrade(&make_wasm_hash(&env), &2);

        env.ledger()
            .set_sequence_number(env.ledger().sequence() + UPGRADE_APPROVAL_EXPIRY_LEDGERS + 1);

        // Revoke should succeed even on expired approval (cleanup)
        assert!(client.try_revoke_upgrade_approval().is_ok());
    }

    #[test]
    fn test_revoke_idempotent_when_no_approval() {
        let (_env, client, _) = setup();

        // Revoke with no approval — should not panic
        assert!(client.try_revoke_upgrade_approval().is_ok());
    }

    // ── SUITE 5: Overflow boundary ─────────────────────────────

    #[test]
    fn test_approve_at_max_ledger_does_not_overflow() {
        let (env, client, _) = setup();

        // Set high ledger sequence number within host bounds
        env.ledger().set_sequence_number(10_000_000);

        // Must not panic — saturating_add used
        let result = client.try_approve_upgrade(
            &make_wasm_hash(&env),
            &BytesN::from_array(&env, &[1u8; 32]),
            &2,
        );

        assert!(
            result.is_ok(),
            "Approve at high ledger sequence must succeed"
        );
    }

    #[test]
    fn test_saturating_add_caps_at_u32_max() {
        // Unit test for the arithmetic
        let near_max: u32 = u32::MAX - 100;

        let result = near_max.saturating_add(UPGRADE_TIMELOCK_LEDGERS);

        assert_eq!(result, u32::MAX, "saturating_add must cap at u32::MAX");
    }

    // ── SUITE 6: Replay prevention ─────────────────────────────

    #[test]
    fn test_cannot_replay_used_approval() {
        let (env, client, _) = setup();

        let hash = make_wasm_hash(&env);

        client.approve_upgrade(&hash, &BytesN::from_array(&env, &[1u8; 32]), &2);

        // Advance past timelock
        env.ledger()
            .set_sequence_number(env.ledger().sequence() + UPGRADE_TIMELOCK_LEDGERS);

        // Execute (consumes approval)
        client.upgrade_contract(&hash, &2);

        // Replay attempt — must fail (no approval)
        let result = client.try_upgrade_contract(&hash, &2);

        assert_eq!(
            result,
            Err(Ok(ContractError::NoUpgradeApproval)),
            "Replaying used approval must fail"
        );
    }

    #[test]
    fn test_hash_mismatch_rejected() {
        let (env, client, _) = setup();

        let approved_hash = make_wasm_hash(&env);
        let different_hash = BytesN::from_array(&env, &[2u8; 32]);

        client.approve_upgrade(&approved_hash, &BytesN::from_array(&env, &[1u8; 32]), &2);

        env.ledger()
            .set_sequence_number(env.ledger().sequence() + UPGRADE_TIMELOCK_LEDGERS);

        let result = client.try_upgrade_contract(&different_hash, &2);

        assert_eq!(result, Err(Ok(ContractError::WasmHashMismatch)));
    }

    // ── SUITE 7: Authorization ─────────────────────────────────

    #[test]
    fn test_approve_requires_admin_auth() {
        let env = Env::default();
        // Do NOT mock all auths — test auth enforcement
        let contract_id = env.register(IssuerRegistryContract, ());
        let client = IssuerRegistryContractClient::new(&env, &contract_id);
        let admin = Address::from_str(&env, ADMIN);

        env.mock_all_auths();
        client.initialize(&admin);
        env.set_auths(&[]);

        let result = client.try_approve_upgrade(
            &make_wasm_hash(&env),
            &BytesN::from_array(&env, &[1u8; 32]),
            &2,
        );

        assert!(result.is_err(), "Non-admin must not approve");
    }

    #[test]
    fn test_execute_requires_admin_auth() {
        let env = Env::default();
        let contract_id = env.register(IssuerRegistryContract, ());
        let client = IssuerRegistryContractClient::new(&env, &contract_id);
        let admin = Address::from_str(&env, ADMIN);

        env.mock_all_auths();
        client.initialize(&admin);
        client.approve_upgrade(
            &make_wasm_hash(&env),
            &BytesN::from_array(&env, &[1u8; 32]),
            &2,
        );
        env.set_auths(&[]);

        let result = client.try_upgrade_contract(&make_wasm_hash(&env), &2);

        assert!(result.is_err(), "Non-admin must not execute");
    }

    #[test]
    fn test_revoke_requires_admin_auth() {
        let env = Env::default();
        let contract_id = env.register(IssuerRegistryContract, ());
        let client = IssuerRegistryContractClient::new(&env, &contract_id);
        let admin = Address::from_str(&env, ADMIN);

        env.mock_all_auths();
        client.initialize(&admin);
        client.approve_upgrade(
            &make_wasm_hash(&env),
            &BytesN::from_array(&env, &[1u8; 32]),
            &2,
        );
        env.set_auths(&[]);

        let result = client.try_revoke_upgrade_approval();

        assert!(result.is_err(), "Non-admin must not revoke");
    }
}
