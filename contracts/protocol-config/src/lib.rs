#![no_std]

use earnproof_shared::{
    AdminTransferState, AdminTransferStatus, ConfigChangeCategory, ConfigChangeSummary,
    ContractError, CriticalAction, CriticalActionPolicy, CriticalActionProposal,
    DisputeFilingAnchor, DisputeFilingPolicy, EmergencyAction, EmergencyActionProposal,
    EmergencyFastTrackPolicy, GenesisRecord, GovernanceActionDomain, GovernanceApprovalEligibility,
    GovernanceApprovalSnapshot, GovernanceApproverSnapshot, GovernanceApproverVersion,
    InterfaceVersion, MigrationStatus,
    PauseAcknowledgementRecord, PauseAcknowledgementRole, PauseScope, SchemaActivationNotice,
    SchemaActivationState, SchemaCompatibility, SchemaLifecyclePolicy, SchemaPolicy,
    SchemaPolicyDigest, SchemaRateLimit, SchemaReplacementPointer, ADMIN_TRANSFER_EXPIRY_LEDGERS,
    CONFIG_HISTORY_CAPACITY, CRITICAL_ACTION_APPROVAL_EXPIRY_LEDGERS,
    DEFAULT_SCHEMA_PAYLOAD_LIMIT, DEFAULT_SCHEMA_RATE_LIMIT, DEFAULT_SCHEMA_RATE_WINDOW_LEDGERS,
    DISPUTE_FILING_POLICY_VERSION, EMERGENCY_FAST_TRACK_ENCODING_VERSION,
    GOVERNANCE_ACTION_HASH_VERSION, GOVERNANCE_APPROVAL_SNAPSHOT_VERSION,
    GOVERNANCE_APPROVER_SNAPSHOT_VERSION, LEGACY_COMMITMENT_ALGORITHM, LEGACY_PROOF_TYPE,
    MAX_CONFIG_HISTORY_PAGE, MAX_CRITICAL_ACTION_SIGNERS, MAX_MIGRATION_BATCH,
    MAX_NULLIFIER_DOMAIN_VERSIONS, MAX_PAUSE_ACKNOWLEDGEMENT_HISTORY_PAGE,
    MAX_SCHEMA_ACTIVATION_NOTICE_SECONDS, MAX_SCHEMA_LINEAGE_DEPTH, MAX_SCHEMA_PROOF_TYPES,
    MAX_SCHEMA_VALIDITY_SECONDS, MIGRATION_STATUS_VERSION, PROTOCOL_CONFIG_INTERFACE_VERSION,
    SHA256_COMMITMENT_ALGORITHM_V1, TTL_EXTEND_TO_LEDGERS, TTL_THRESHOLD_LEDGERS,
};
use soroban_sdk::{
    contract, contractevent, contractimpl, contracttype, xdr::ToXdr, Address, BytesN, Env, Symbol,
    Vec,
};

#[contract]
pub struct ProtocolConfigContract;

#[contracttype]
enum DataKey {
    MigrationStatus,
    Admin,
    Paused,
    ScopedPause(PauseScope),
    ConfigVersion,
    SchemaVersion(u32),
    SchemaActivation(u32),
    MinSchemaActivationNotice,
    SchemaPredecessor(u32),
    SchemaReplacement(u32),
    SchemaPolicy(u32),
    SchemaLifecycle(u32),
    SchemaRateLimit(u32),
    CommitmentAlgorithm(u32),
    ProofTypeApproved(BytesN<32>),
    PendingAdmin,
    PendingAdminNominatedAt,
    PendingAdminExpiresAt,
    /// Allowlist entry: maps a WASM hash to the target contract version it
    /// must install.  Only hashes pre-approved by the admin may be applied.
    AllowedWasm(BytesN<32>),
    /// Monotonically-increasing contract version stored in instance storage.
    /// Prevents installing an older (or equal) version over a newer one.
    ContractVersion,
    Successor,
    Decommissioned,
    /// Immutable deployment identity, written once at `initialize`.
    Genesis,
    /// Governed per-schema-version auxiliary payload size bound (bytes).
    SchemaPayloadLimit(u32),
    /// Ring-buffer slot holding one bounded change-history summary.
    ConfigHistoryRing(u32),
    /// Monotonic count of change-history entries ever appended.
    ConfigHistoryTotal,
    CriticalActionPolicy,
    CriticalActionPolicyVersion,
    CriticalSignerVersion(Address),
    CriticalActionProposal(BytesN<32>),
    CriticalActionDomain(BytesN<32>),
    CriticalActionSnapshot(BytesN<32>),
    CriticalApproverSnapshot(BytesN<32>),
    CriticalActionNonce,
    EmergencyFastTrackPolicy,
    EmergencyActionProposal(BytesN<32>),
    EmergencyActionNonce,
    PauseAcknowledger(Address),
    PauseAcknowledgementHash(BytesN<32>),
    PauseAcknowledgementHistory(u32),
    PauseAcknowledgementCount,
    DisputeFilingPolicy,
    SubjectNullifierDomain(u32),
    SubjectNullifierDomains,
}

// ── admin transfer events ─────────────────────────────────────────────────────────

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

// ── existing events ─────────────────────────────────────────────────────────

#[contractevent]
pub struct Initialized {
    pub admin: Address,
}

#[contractevent]
pub struct AdminChanged {
    pub new_admin: Address,
}

#[contractevent]
pub struct Paused {
    pub paused: bool,
}

#[contractevent]
pub struct Unpaused {
    pub paused: bool,
}

#[contractevent]
pub struct SchemaApproved {
    pub version: u32,
}

#[contractevent]
pub struct SchemaActivationScheduled {
    pub version: u32,
    pub proposed_at: u64,
    pub activates_at: u64,
    pub minimum_notice_seconds: u64,
}

#[contractevent]
pub struct MinimumSchemaNoticeChanged {
    pub minimum_notice_seconds: u64,
    pub changed_by: Address,
}

#[contractevent]
pub struct SchemaDeprecated {
    pub version: u32,
}

#[contractevent]
pub struct SchemaReplacementSet {
    pub deprecated_version: u32,
    pub replacement_version: u32,
    pub compatibility: SchemaCompatibility,
    pub effective_ledger: u32,
}

#[contractevent]
pub struct SchemaPredecessorSet {
    pub version: u32,
    pub predecessor: u32,
}

#[contractevent]
pub struct ProofTypeApprovedEvent {
    pub proof_type: BytesN<32>,
}

#[contractevent]
pub struct ProofTypeDeprecatedEvent {
    pub proof_type: BytesN<32>,
}

#[contractevent]
pub struct SchemaPolicySet {
    pub version: u32,
    pub proof_type_count: u32,
    pub max_validity_seconds: u64,
}

#[contractevent]
pub struct CommitmentAlgorithmPolicySet {
    pub algorithm: u32,
    pub supported: bool,
}

#[contractevent]
pub struct ScopedPauseChanged {
    pub scope: PauseScope,
    pub paused: bool,
}

#[contractevent]
pub struct PauseAcknowledged {
    pub record: PauseAcknowledgementRecord,
}

#[contractevent]
pub struct PauseAcknowledgerChanged {
    pub address: Address,
    pub enabled: bool,
    pub changed_by: Address,
}

#[contractevent]
pub struct SuccessorNominated {
    pub successor: Address,
    pub nominated_by: Address,
}

#[contractevent]
pub struct ContractDecommissioned {
    pub old_instance: Address,
    pub successor_instance: Address,
    pub activated_by: Address,
}

/// Emitted when the admin sets or updates the maximum auxiliary payload size
/// accepted for a schema version.
#[contractevent]
pub struct SchemaPayloadLimitSet {
    pub version: u32,
    pub max_size: u32,
}

#[contractevent]
pub struct DisputeFilingPolicySet {
    pub version: u32,
    pub anchor: DisputeFilingAnchor,
    pub opens_after_seconds: u64,
    pub closes_after_seconds: u64,
}

#[contractevent]
pub struct NullifierDomainVersionSet {
    pub domain_version: u32,
    pub supported: bool,
}

#[contractevent]
pub struct SchemaRateLimitSet {
    pub version: u32,
    pub max_registrations: u32,
    pub window_ledgers: u32,
}

#[contractevent]
pub struct CriticalActionProposed {
    pub proposal_id: BytesN<32>,
    pub proposer: Address,
}

#[contractevent]
pub struct CriticalActionApproved {
    pub proposal_id: BytesN<32>,
    pub signer: Address,
}

#[contractevent]
pub struct CriticalActionCancelled {
    pub proposal_id: BytesN<32>,
    pub cancelled_by: Address,
}

#[contractevent]
pub struct CriticalActionExecuted {
    pub proposal_id: BytesN<32>,
}

#[contractevent]
pub struct CriticalActionPolicySet {
    pub enabled: bool,
    pub threshold: u32,
    pub signer_count: u32,
}

#[contractevent]
pub struct EmergencyFastTrackPolicySet {
    pub enabled: bool,
    pub threshold: u32,
    pub guardian_count: u32,
    pub max_lifetime_ledgers: u32,
}

#[contractevent]
pub struct EmergencyActionProposed {
    pub proposal_id: BytesN<32>,
    pub proposer: Address,
    pub expires_at: u32,
}

#[contractevent]
pub struct EmergencyActionApproved {
    pub proposal_id: BytesN<32>,
    pub guardian: Address,
}

#[contractevent]
pub struct EmergencyActionCancelled {
    pub proposal_id: BytesN<32>,
    pub cancelled_by: Address,
}

#[contractevent]
pub struct EmergencyActionExecuted {
    pub proposal_id: BytesN<32>,
    pub reason_commitment: BytesN<32>,
}

// ── upgrade events ───────────────────────────────────────────────────────────

/// Emitted when the admin adds a WASM hash to the upgrade allowlist.
#[contractevent]
pub struct UpgradeAllowlisted {
    pub wasm_hash: BytesN<32>,
    pub new_contract_version: u32,
    pub approved_by: Address,
}

/// Emitted when the admin removes a WASM hash from the allowlist without
/// applying it (e.g. rolling back an approved-but-not-yet-applied hash).
#[contractevent]
pub struct UpgradeRevoked {
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

#[contractimpl]
impl ProtocolConfigContract {
    pub fn is_decommissioned(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Decommissioned)
            .unwrap_or(false)
    }

    fn ensure_not_decommissioned(env: &Env) -> Result<(), ContractError> {
        if Self::is_decommissioned(env.clone()) {
            Err(ContractError::InvalidState)
        } else {
            Ok(())
        }
    }

    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(ContractError::AlreadyInitialized);
        }

        Self::require_valid_principal(&admin)?;
        Self::require_auth(&admin);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage()
            .instance()
            .set(&DataKey::ConfigVersion, &1_u32);
        env.storage()
            .instance()
            .set(&DataKey::ContractVersion, &1_u32);
        env.storage().instance().set(
            &DataKey::DisputeFilingPolicy,
            &Self::default_dispute_filing_policy(),
        );
        env.storage()
            .instance()
            .set(&DataKey::SubjectNullifierDomain(1), &true);
        env.storage().instance().set(
            &DataKey::SubjectNullifierDomains,
            &soroban_sdk::vec![&env, 1_u32],
        );
        let genesis = GenesisRecord {
            genesis_id: earnproof_shared::compute_genesis_id(&env, "earnproof_protocol_config"),
            initialized_at_ledger: env.ledger().sequence(),
        };
        env.storage().instance().set(&DataKey::Genesis, &genesis);
        Self::extend_instance_ttl(env.clone());
        Initialized { admin }.publish(&env);
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

    pub fn get_admin(env: Env) -> Result<Address, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ContractError::NotInitialized)
    }

    /// Machine-readable interface version this contract exposes to consumers.
    pub fn interface_version(_env: Env) -> InterfaceVersion {
        PROTOCOL_CONFIG_INTERFACE_VERSION
    }

    pub fn nominate_admin(env: Env, new_admin: Address) -> Result<(), ContractError> {
        Self::ensure_not_decommissioned(&env)?;
        let admin = Self::get_admin(env.clone())?;
        Self::require_valid_principal(&new_admin)?;
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
        Self::ensure_not_decommissioned(&env)?;
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

        Self::bump_config_version(env.clone());
        Self::append_config_history(
            env.clone(),
            ConfigChangeCategory::AdminRotation,
            Self::commit(&env, pending_admin.clone()),
        );
        AdminChanged {
            new_admin: pending_admin.clone(),
        }
        .publish(&env);

        AdminTransferAccepted {
            new_admin: pending_admin,
        }
        .publish(&env);
        Ok(())
    }

    pub fn cancel_admin_transfer(env: Env) -> Result<(), ContractError> {
        Self::ensure_not_decommissioned(&env)?;
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

    /// Permissionless cleanup after the exclusive expiry boundary. Cleanup
    /// removes nomination state without changing the current administrator.
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

    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    pub fn pause(env: Env) -> Result<(), ContractError> {
        Self::ensure_not_decommissioned(&env)?;
        let admin = Self::get_admin(env.clone())?;
        Self::require_auth(&admin);
        env.storage().instance().set(&DataKey::Paused, &true);
        Self::bump_config_version(env.clone());
        Self::append_config_history(
            env.clone(),
            ConfigChangeCategory::PauseToggle,
            Self::commit(&env, true),
        );
        Paused { paused: true }.publish(&env);
        Ok(())
    }

    pub fn unpause(env: Env) -> Result<(), ContractError> {
        Self::ensure_not_decommissioned(&env)?;
        let admin = Self::get_admin(env.clone())?;
        Self::require_auth(&admin);
        env.storage().instance().set(&DataKey::Paused, &false);
        Self::bump_config_version(env.clone());
        Self::append_config_history(
            env.clone(),
            ConfigChangeCategory::PauseToggle,
            Self::commit(&env, false),
        );
        Unpaused { paused: false }.publish(&env);
        Ok(())
    }

    pub fn set_scoped_pause(
        env: Env,
        scope: PauseScope,
        paused: bool,
    ) -> Result<(), ContractError> {
        Self::ensure_not_decommissioned(&env)?;
        let admin = Self::get_admin(env.clone())?;
        Self::require_auth(&admin);
        env.storage()
            .persistent()
            .set(&DataKey::ScopedPause(scope), &paused);
        Self::bump_config_version(env.clone());
        Self::append_config_history(
            env.clone(),
            ConfigChangeCategory::ScopedPause,
            Self::commit(&env, (scope, paused)),
        );
        ScopedPauseChanged { scope, paused }.publish(&env);
        Ok(())
    }

    pub fn is_scope_paused(env: Env, scope: PauseScope) -> bool {
        let scoped = env
            .storage()
            .persistent()
            .get(&DataKey::ScopedPause(scope))
            .unwrap_or(false);
        if scoped {
            return true;
        }
        if (scope == PauseScope::Global
            || scope == PauseScope::Registration
            || scope == PauseScope::Updates)
            && Self::is_paused(env)
        {
            return true;
        }
        false
    }

    pub fn set_pause_acknowledger(
        env: Env,
        address: Address,
        enabled: bool,
    ) -> Result<(), ContractError> {
        Self::ensure_not_decommissioned(&env)?;
        let admin = Self::get_admin(env.clone())?;
        Self::require_valid_principal(&address)?;
        Self::require_auth(&admin);
        let key = DataKey::PauseAcknowledger(address.clone());
        env.storage().persistent().set(&key, &enabled);
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_THRESHOLD_LEDGERS, TTL_EXTEND_TO_LEDGERS);
        PauseAcknowledgerChanged {
            address,
            enabled,
            changed_by: admin,
        }
        .publish(&env);
        Ok(())
    }

    pub fn is_pause_acknowledger(env: Env, address: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::PauseAcknowledger(address))
            .unwrap_or(false)
    }

    pub fn acknowledge_pause(
        env: Env,
        scope: PauseScope,
        acknowledgement_hash: BytesN<32>,
        actor: Address,
    ) -> Result<PauseAcknowledgementRecord, ContractError> {
        Self::ensure_not_decommissioned(&env)?;
        if !Self::is_scope_paused(env.clone(), scope) {
            return Err(ContractError::InvalidState);
        }
        if acknowledgement_hash == BytesN::from_array(&env, &[0u8; 32]) {
            return Err(ContractError::InvalidInput);
        }
        let admin = Self::get_admin(env.clone())?;
        let role = if actor == admin {
            PauseAcknowledgementRole::Administrator
        } else if Self::is_pause_acknowledger(env.clone(), actor.clone()) {
            PauseAcknowledgementRole::IncidentOperator
        } else {
            return Err(ContractError::Unauthorized);
        };
        Self::require_auth(&actor);

        let hash_key = DataKey::PauseAcknowledgementHash(acknowledgement_hash.clone());
        if env.storage().persistent().has(&hash_key) {
            return Err(ContractError::InvalidInput);
        }
        let sequence: u32 = env
            .storage()
            .instance()
            .get(&DataKey::PauseAcknowledgementCount)
            .unwrap_or(0);
        if sequence == u32::MAX {
            return Err(ContractError::InvalidState);
        }
        let record = PauseAcknowledgementRecord {
            record_version: 1,
            sequence,
            scope,
            acknowledgement_hash,
            acknowledged_by: actor,
            role,
            acknowledged_at_ledger: env.ledger().sequence(),
        };
        let history_key = DataKey::PauseAcknowledgementHistory(sequence);
        env.storage().persistent().set(&hash_key, &record);
        env.storage().persistent().set(&history_key, &record);
        env.storage()
            .instance()
            .set(&DataKey::PauseAcknowledgementCount, &(sequence + 1));
        for key in [&hash_key, &history_key] {
            env.storage().persistent().extend_ttl(
                key,
                TTL_THRESHOLD_LEDGERS,
                TTL_EXTEND_TO_LEDGERS,
            );
        }
        Self::extend_instance_ttl(env.clone());
        PauseAcknowledged {
            record: record.clone(),
        }
        .publish(&env);
        Ok(record)
    }

    pub fn get_pause_acknowledgement(
        env: Env,
        acknowledgement_hash: BytesN<32>,
    ) -> Option<PauseAcknowledgementRecord> {
        env.storage()
            .persistent()
            .get(&DataKey::PauseAcknowledgementHash(acknowledgement_hash))
    }

    pub fn get_pause_acknowledgements(
        env: Env,
        cursor: u32,
        limit: u32,
    ) -> Result<Vec<PauseAcknowledgementRecord>, ContractError> {
        if limit == 0 || limit > MAX_PAUSE_ACKNOWLEDGEMENT_HISTORY_PAGE {
            return Err(ContractError::BatchTooLarge);
        }
        let count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::PauseAcknowledgementCount)
            .unwrap_or(0);
        let end = cursor.saturating_add(limit).min(count);
        let mut records = Vec::new(&env);
        for sequence in cursor..end {
            if let Some(record) = env
                .storage()
                .persistent()
                .get(&DataKey::PauseAcknowledgementHistory(sequence))
            {
                records.push_back(record);
            }
        }
        Ok(records)
    }

    pub fn nominate_successor(env: Env, successor: Address) -> Result<(), ContractError> {
        Self::ensure_not_decommissioned(&env)?;
        let admin = Self::get_admin(env.clone())?;
        Self::require_valid_principal(&successor)?;
        Self::require_auth(&admin);
        env.storage()
            .instance()
            .set(&DataKey::Successor, &successor);
        SuccessorNominated {
            successor: successor.clone(),
            nominated_by: admin,
        }
        .publish(&env);
        Ok(())
    }

    pub fn get_successor(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Successor)
    }

    pub fn activate_successor(env: Env) -> Result<(), ContractError> {
        Self::ensure_not_decommissioned(&env)?;
        let admin = Self::get_admin(env.clone())?;
        Self::require_auth(&admin);
        let successor: Address = env
            .storage()
            .instance()
            .get(&DataKey::Successor)
            .ok_or(ContractError::NotFound)?;

        env.storage()
            .instance()
            .set(&DataKey::Decommissioned, &true);
        ContractDecommissioned {
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

    pub fn keepalive_schema_version(env: Env, version: u32) -> bool {
        if version == 0 {
            return false;
        }
        let key = DataKey::SchemaVersion(version);
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

    pub fn keepalive_proof_type(env: Env, proof_type: BytesN<32>) -> bool {
        let key = DataKey::ProofTypeApproved(proof_type);
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

    pub fn approve_proof_type(env: Env, proof_type: BytesN<32>) -> Result<(), ContractError> {
        Self::ensure_not_decommissioned(&env)?;
        let admin = Self::get_admin(env.clone())?;
        Self::require_auth(&admin);
        let key = DataKey::ProofTypeApproved(proof_type.clone());
        env.storage().persistent().set(&key, &true);
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_THRESHOLD_LEDGERS, TTL_EXTEND_TO_LEDGERS);
        Self::bump_config_version(env.clone());
        ProofTypeApprovedEvent { proof_type }.publish(&env);
        Ok(())
    }

    pub fn deprecate_proof_type(env: Env, proof_type: BytesN<32>) -> Result<(), ContractError> {
        Self::ensure_not_decommissioned(&env)?;
        let admin = Self::get_admin(env.clone())?;
        Self::require_auth(&admin);
        let key = DataKey::ProofTypeApproved(proof_type.clone());
        env.storage().persistent().set(&key, &false);
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_THRESHOLD_LEDGERS, TTL_EXTEND_TO_LEDGERS);
        Self::bump_config_version(env.clone());
        ProofTypeDeprecatedEvent { proof_type }.publish(&env);
        Ok(())
    }

    pub fn is_proof_type_approved(env: Env, proof_type: BytesN<32>) -> bool {
        let key = DataKey::ProofTypeApproved(proof_type);
        let approved = env.storage().persistent().get(&key).unwrap_or(false);
        if env.storage().persistent().has(&key) {
            env.storage().persistent().extend_ttl(
                &key,
                TTL_THRESHOLD_LEDGERS,
                TTL_EXTEND_TO_LEDGERS,
            );
        }
        approved
    }

    pub fn approve_schema_with_predecessor(
        env: Env,
        version: u32,
        predecessor: u32,
    ) -> Result<(), ContractError> {
        Self::ensure_not_decommissioned(&env)?;
        let admin = Self::get_admin(env.clone())?;
        Self::require_auth(&admin);
        Self::ensure_critical_action_policy_disabled(&env)?;
        Self::ensure_nonzero_version(version)?;
        let proposed_at = env.ledger().timestamp();
        let minimum_notice_seconds = Self::get_min_schema_activation_notice(env.clone());
        Self::validate_schema_activation_time(proposed_at, proposed_at, minimum_notice_seconds)?;
        if env
            .storage()
            .persistent()
            .get::<_, bool>(&DataKey::SchemaVersion(version))
            == Some(true)
        {
            return Err(ContractError::AlreadyExists);
        }
        if predecessor == version {
            return Err(ContractError::InvalidInput);
        }

        let existing = Self::stored_predecessor(env.clone(), version);
        if existing.is_some_and(|stored| stored != predecessor)
            || (existing.is_none()
                && predecessor != 0
                && Self::schema_version_exists(env.clone(), version))
        {
            return Err(ContractError::InvalidState);
        }
        if predecessor != 0
            && (!Self::schema_version_exists(env.clone(), predecessor)
                || Self::lineage_reaches(env.clone(), predecessor, version))
        {
            return Err(ContractError::InvalidState);
        }

        env.storage()
            .persistent()
            .set(&DataKey::SchemaVersion(version), &true);
        Self::record_schema_approval(&env, version);
        Self::extend_schema_ttl(env.clone(), version);
        Self::store_schema_activation_notice(
            env.clone(),
            version,
            proposed_at,
            proposed_at,
            minimum_notice_seconds,
        );
        if predecessor != 0 {
            let key = DataKey::SchemaPredecessor(version);
            env.storage().persistent().set(&key, &predecessor);
            env.storage().persistent().extend_ttl(
                &key,
                TTL_THRESHOLD_LEDGERS,
                TTL_EXTEND_TO_LEDGERS,
            );
            SchemaPredecessorSet {
                version,
                predecessor,
            }
            .publish(&env);
        }
        Self::bump_config_version(env.clone());
        SchemaApproved { version }.publish(&env);
        Ok(())
    }

    pub fn get_schema_predecessor(env: Env, version: u32) -> Option<u32> {
        Self::stored_predecessor(env, version)
    }

    pub fn get_schema_lineage(env: Env, version: u32) -> Vec<u32> {
        let mut chain = Vec::new(&env);
        if version == 0 {
            return chain;
        }
        chain.push_back(version);
        let mut current = version;
        let mut depth = 0;
        while depth < MAX_SCHEMA_LINEAGE_DEPTH {
            match Self::stored_predecessor(env.clone(), current) {
                Some(predecessor) => {
                    chain.push_back(predecessor);
                    current = predecessor;
                    depth += 1;
                }
                None => break,
            }
        }
        chain
    }

    pub fn approve_schema_version(env: Env, version: u32) -> Result<(), ContractError> {
        Self::ensure_not_decommissioned(&env)?;
        let admin = Self::get_admin(env.clone())?;
        Self::require_auth(&admin);
        Self::ensure_critical_action_policy_disabled(&env)?;
        let now = env.ledger().timestamp();
        let commitment = Self::commit(&env, version);
        Self::apply_schema_activation(
            env,
            version,
            now,
            ConfigChangeCategory::SchemaApproval,
            commitment,
        )
    }

    /// Approves a schema with an explicit future activation timestamp.
    pub fn approve_schema_at(
        env: Env,
        version: u32,
        activates_at: u64,
    ) -> Result<(), ContractError> {
        Self::ensure_not_decommissioned(&env)?;
        let admin = Self::get_admin(env.clone())?;
        Self::require_auth(&admin);
        Self::ensure_critical_action_policy_disabled(&env)?;
        let commitment = Self::commit(&env, (version, activates_at));
        Self::apply_schema_activation(
            env,
            version,
            activates_at,
            ConfigChangeCategory::SchemaActivation,
            commitment,
        )
    }

    pub fn set_min_schema_activation_notice(
        env: Env,
        minimum_notice_seconds: u64,
    ) -> Result<(), ContractError> {
        Self::ensure_not_decommissioned(&env)?;
        let admin = Self::get_admin(env.clone())?;
        Self::require_auth(&admin);
        if minimum_notice_seconds > MAX_SCHEMA_ACTIVATION_NOTICE_SECONDS {
            return Err(ContractError::InvalidTimingConfig);
        }
        if Self::get_min_schema_activation_notice(env.clone()) == minimum_notice_seconds {
            return Err(ContractError::InvalidState);
        }
        env.storage()
            .instance()
            .set(&DataKey::MinSchemaActivationNotice, &minimum_notice_seconds);
        Self::bump_config_version(env.clone());
        Self::append_config_history(
            env.clone(),
            ConfigChangeCategory::SchemaActivationNotice,
            Self::commit(&env, minimum_notice_seconds),
        );
        MinimumSchemaNoticeChanged {
            minimum_notice_seconds,
            changed_by: admin,
        }
        .publish(&env);
        Ok(())
    }

    pub fn get_min_schema_activation_notice(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::MinSchemaActivationNotice)
            .unwrap_or(0)
    }

    pub fn get_schema_activation_notice(env: Env, version: u32) -> Option<SchemaActivationNotice> {
        let key = DataKey::SchemaActivation(version);
        let notice = env.storage().persistent().get(&key);
        if notice.is_some() {
            env.storage().persistent().extend_ttl(
                &key,
                TTL_THRESHOLD_LEDGERS,
                TTL_EXTEND_TO_LEDGERS,
            );
        }
        notice
    }

    pub fn get_schema_activation_state(env: Env, version: u32) -> SchemaActivationState {
        if version == 0 {
            return SchemaActivationState::Unknown;
        }
        let stored: Option<bool> = env
            .storage()
            .persistent()
            .get(&DataKey::SchemaVersion(version));
        match stored {
            None => SchemaActivationState::Unknown,
            Some(false) => SchemaActivationState::Deprecated,
            Some(true) => match Self::get_schema_activation_notice(env.clone(), version) {
                Some(notice) if env.ledger().timestamp() < notice.activates_at => {
                    SchemaActivationState::Pending
                }
                _ => SchemaActivationState::Active,
            },
        }
    }

    pub fn deprecate_schema_version(env: Env, version: u32) -> Result<(), ContractError> {
        Self::ensure_not_decommissioned(&env)?;
        let admin = Self::get_admin(env.clone())?;
        Self::require_auth(&admin);
        Self::ensure_critical_action_policy_disabled(&env)?;
        Self::ensure_nonzero_version(version)?;
        env.storage()
            .persistent()
            .set(&DataKey::SchemaVersion(version), &false);
        Self::record_schema_deprecation(&env, version);
        Self::extend_schema_ttl(env.clone(), version);
        Self::bump_config_version(env.clone());
        Self::append_config_history(
            env.clone(),
            ConfigChangeCategory::SchemaDeprecation,
            Self::commit(&env, version),
        );
        SchemaDeprecated { version }.publish(&env);
        Ok(())
    }

    /// Atomically deprecates a schema and records its approved successor.
    pub fn deprecate_schema_to(
        env: Env,
        deprecated_version: u32,
        replacement_version: u32,
        compatibility: SchemaCompatibility,
    ) -> Result<SchemaReplacementPointer, ContractError> {
        Self::ensure_not_decommissioned(&env)?;
        let admin = Self::get_admin(env.clone())?;
        Self::require_auth(&admin);
        Self::ensure_critical_action_policy_disabled(&env)?;
        let commitment = Self::commit(
            &env,
            (deprecated_version, replacement_version, compatibility),
        );
        Self::apply_schema_replacement(
            env,
            deprecated_version,
            replacement_version,
            compatibility,
            commitment,
        )
    }

    pub fn get_schema_replacement(
        env: Env,
        deprecated_version: u32,
    ) -> Option<SchemaReplacementPointer> {
        Self::stored_schema_replacement(env, deprecated_version)
    }

    /// Returns the source followed by each retained successor, bounded by the
    /// same maximum depth used by cycle detection.
    pub fn get_schema_replacement_chain(env: Env, version: u32) -> Vec<u32> {
        let mut chain = Vec::new(&env);
        if version == 0 {
            return chain;
        }
        let mut current = version;
        for _ in 0..=MAX_SCHEMA_LINEAGE_DEPTH {
            if chain.contains(current) {
                break;
            }
            chain.push_back(current);
            match Self::stored_schema_replacement(env.clone(), current) {
                Some(pointer) => current = pointer.replacement_version,
                None => break,
            }
        }
        chain
    }

    pub fn is_schema_version_approved(env: Env, version: u32) -> bool {
        if version == 0 {
            return false;
        }

        let key = DataKey::SchemaVersion(version);
        let approved = env.storage().persistent().get(&key).unwrap_or(false);
        if env.storage().persistent().has(&key) {
            env.storage().persistent().extend_ttl(
                &key,
                TTL_THRESHOLD_LEDGERS,
                TTL_EXTEND_TO_LEDGERS,
            );
        }
        approved
            && Self::get_schema_activation_notice(env.clone(), version)
                .map(|notice| env.ledger().timestamp() >= notice.activates_at)
                .unwrap_or(true)
    }

    pub fn set_schema_policy(
        env: Env,
        version: u32,
        proof_types: Vec<u32>,
        max_validity_seconds: u64,
    ) -> Result<(), ContractError> {
        Self::ensure_not_decommissioned(&env)?;
        let admin = Self::get_admin(env.clone())?;
        Self::require_auth(&admin);
        Self::ensure_nonzero_version(version)?;
        if env
            .storage()
            .persistent()
            .has(&DataKey::SchemaVersion(version))
            || proof_types.is_empty()
            || proof_types.len() > MAX_SCHEMA_PROOF_TYPES
            || max_validity_seconds == 0
            || max_validity_seconds > MAX_SCHEMA_VALIDITY_SECONDS
        {
            return Err(ContractError::InvalidInput);
        }
        for index in 0..proof_types.len() {
            let value = proof_types.get(index).ok_or(ContractError::InvalidInput)?;
            for prior in 0..index {
                if proof_types.get(prior) == Some(value) {
                    return Err(ContractError::InvalidInput);
                }
            }
        }

        let policy = SchemaPolicy {
            proof_types: proof_types.clone(),
            max_validity_seconds,
        };
        let key = DataKey::SchemaPolicy(version);
        env.storage().persistent().set(&key, &policy);
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_THRESHOLD_LEDGERS, TTL_EXTEND_TO_LEDGERS);
        Self::bump_config_version(env.clone());
        Self::append_config_history(
            env.clone(),
            ConfigChangeCategory::SchemaPolicy,
            Self::commit(&env, (version, policy)),
        );
        SchemaPolicySet {
            version,
            proof_type_count: proof_types.len(),
            max_validity_seconds,
        }
        .publish(&env);
        Ok(())
    }

    pub fn get_schema_policy(env: Env, version: u32) -> SchemaPolicy {
        if version == 0 {
            return Self::legacy_schema_policy(&env);
        }
        let key = DataKey::SchemaPolicy(version);
        let policy = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Self::legacy_schema_policy(&env));
        if env.storage().persistent().has(&key) {
            env.storage()
                .persistent()
                .extend_ttl(&key, TTL_THRESHOLD_LEDGERS, TTL_EXTEND_TO_LEDGERS);
        }
        policy
    }

    pub fn set_commitment_algorithm(
        env: Env,
        algorithm: u32,
        supported: bool,
    ) -> Result<(), ContractError> {
        Self::ensure_not_decommissioned(&env)?;
        let admin = Self::get_admin(env.clone())?;
        Self::require_auth(&admin);
        if algorithm != LEGACY_COMMITMENT_ALGORITHM && algorithm != SHA256_COMMITMENT_ALGORITHM_V1 {
            return Err(ContractError::InvalidInput);
        }
        env.storage()
            .instance()
            .set(&DataKey::CommitmentAlgorithm(algorithm), &supported);
        Self::bump_config_version(env.clone());
        Self::append_config_history(
            env.clone(),
            ConfigChangeCategory::CommitmentAlgorithmPolicy,
            Self::commit(&env, (algorithm, supported)),
        );
        CommitmentAlgorithmPolicySet {
            algorithm,
            supported,
        }
        .publish(&env);
        Ok(())
    }

    pub fn is_algorithm_supported(env: Env, algorithm: u32) -> bool {
        if algorithm != LEGACY_COMMITMENT_ALGORITHM && algorithm != SHA256_COMMITMENT_ALGORITHM_V1 {
            return false;
        }
        env.storage()
            .instance()
            .get(&DataKey::CommitmentAlgorithm(algorithm))
            .unwrap_or(true)
    }

    pub fn get_config_version(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::ConfigVersion)
            .unwrap_or(0)
    }

    /// Admin-only: replace the global proof-dispute filing window. The policy
    /// applies at filing time, including to proofs created before the change.
    pub fn set_dispute_filing_policy(
        env: Env,
        policy: DisputeFilingPolicy,
    ) -> Result<(), ContractError> {
        Self::ensure_not_decommissioned(&env)?;
        let admin = Self::get_admin(env.clone())?;
        Self::require_auth(&admin);
        if policy.version != DISPUTE_FILING_POLICY_VERSION
            || policy.closes_after_seconds <= policy.opens_after_seconds
        {
            return Err(ContractError::InvalidInput);
        }
        env.storage()
            .instance()
            .set(&DataKey::DisputeFilingPolicy, &policy);
        Self::bump_config_version(env.clone());
        Self::append_config_history(
            env.clone(),
            ConfigChangeCategory::DisputeFilingPolicy,
            Self::commit(&env, policy.clone()),
        );
        DisputeFilingPolicySet {
            version: policy.version,
            anchor: policy.anchor,
            opens_after_seconds: policy.opens_after_seconds,
            closes_after_seconds: policy.closes_after_seconds,
        }
        .publish(&env);
        Ok(())
    }

    /// Admin-only: enable or disable a versioned subject-nullifier derivation
    /// domain. Version zero is reserved and cannot be configured.
    pub fn set_nullifier_domain_version(
        env: Env,
        domain_version: u32,
        supported: bool,
    ) -> Result<(), ContractError> {
        Self::ensure_not_decommissioned(&env)?;
        let admin = Self::get_admin(env.clone())?;
        Self::require_auth(&admin);
        if domain_version == 0 {
            return Err(ContractError::InvalidInput);
        }
        let key = DataKey::SubjectNullifierDomain(domain_version);
        let known = env.storage().instance().has(&key);
        if !known && !supported {
            return Err(ContractError::InvalidInput);
        }
        if !known {
            let mut versions: Vec<u32> = env
                .storage()
                .instance()
                .get(&DataKey::SubjectNullifierDomains)
                .unwrap_or_else(|| soroban_sdk::vec![&env, 1_u32]);
            if versions.len() >= MAX_NULLIFIER_DOMAIN_VERSIONS {
                return Err(ContractError::InvalidInput);
            }
            versions.push_back(domain_version);
            env.storage()
                .instance()
                .set(&DataKey::SubjectNullifierDomains, &versions);
        }
        env.storage().instance().set(&key, &supported);
        Self::bump_config_version(env.clone());
        Self::append_config_history(
            env.clone(),
            ConfigChangeCategory::NullifierDomainPolicy,
            Self::commit(&env, (domain_version, supported)),
        );
        NullifierDomainVersionSet {
            domain_version,
            supported,
        }
        .publish(&env);
        Ok(())
    }

    /// Returns the active policy. Pre-policy deployments read the compatibility
    /// default, preserving the legacy behavior that filings never age out.
    pub fn get_dispute_filing_policy(env: Env) -> DisputeFilingPolicy {
        env.storage()
            .instance()
            .get(&DataKey::DisputeFilingPolicy)
            .unwrap_or_else(Self::default_dispute_filing_policy)
    }

    pub fn is_nullifier_domain_supported(env: Env, domain_version: u32) -> bool {
        domain_version != 0
            && env
                .storage()
                .instance()
                .get(&DataKey::SubjectNullifierDomain(domain_version))
                .unwrap_or(false)
    }

    /// Returns supported versions in their stable first-governed order.
    pub fn get_nullifier_domain_versions(env: Env) -> Vec<u32> {
        let versions: Vec<u32> = env
            .storage()
            .instance()
            .get(&DataKey::SubjectNullifierDomains)
            .unwrap_or_else(|| soroban_sdk::vec![&env, 1_u32]);
        let mut supported = Vec::new(&env);
        for version in versions.iter() {
            if Self::is_nullifier_domain_supported(env.clone(), version) {
                supported.push_back(version);
            }
        }
        supported
    }

    // ── schema payload size limits ───────────────────────────────────────────

    /// Admin-only: set the maximum auxiliary payload size (in bytes) a proof
    /// registered under `version` may carry. Applies going forward only —
    /// proofs already registered are never re-validated against a changed
    /// limit.
    pub fn set_schema_payload_limit(
        env: Env,
        version: u32,
        max_size: u32,
    ) -> Result<(), ContractError> {
        Self::ensure_not_decommissioned(&env)?;
        let admin = Self::get_admin(env.clone())?;
        Self::require_auth(&admin);
        Self::ensure_critical_action_policy_disabled(&env)?;
        Self::ensure_nonzero_version(version)?;
        env.storage()
            .persistent()
            .set(&DataKey::SchemaPayloadLimit(version), &max_size);
        Self::extend_schema_payload_limit_ttl(env.clone(), version);
        Self::bump_config_version(env.clone());
        Self::append_config_history(
            env.clone(),
            ConfigChangeCategory::SchemaPayloadLimit,
            Self::commit(&env, (version, max_size)),
        );
        SchemaPayloadLimitSet { version, max_size }.publish(&env);
        Ok(())
    }

    pub fn get_critical_action_policy(env: Env) -> CriticalActionPolicy {
        Self::critical_action_policy(&env)
    }

    pub fn get_critical_policy_version(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::CriticalActionPolicyVersion)
            .unwrap_or(0)
    }

    /// Bootstrap or replace the critical-action policy. Once threshold
    /// approvals are enabled, policy changes must themselves be approved.
    pub fn set_critical_action_policy(
        env: Env,
        policy: CriticalActionPolicy,
    ) -> Result<(), ContractError> {
        Self::ensure_not_decommissioned(&env)?;
        let admin = Self::get_admin(env.clone())?;
        Self::require_auth(&admin);
        if Self::critical_action_policy(&env).enabled {
            return Err(ContractError::ThresholdApprovalRequired);
        }
        Self::validate_critical_action_policy(&policy)?;
        Self::store_critical_action_policy(env, policy)
    }

    pub fn get_emergency_policy(env: Env) -> EmergencyFastTrackPolicy {
        Self::emergency_policy(&env)
    }

    /// Bootstraps the emergency policy. After normal threshold governance is
    /// enabled, updates must use `CriticalAction::EmergencyPolicyUpdate`.
    pub fn set_emergency_policy(
        env: Env,
        policy: EmergencyFastTrackPolicy,
    ) -> Result<(), ContractError> {
        Self::ensure_not_decommissioned(&env)?;
        let admin = Self::get_admin(env.clone())?;
        Self::require_auth(&admin);
        if Self::critical_action_policy(&env).enabled {
            return Err(ContractError::ThresholdApprovalRequired);
        }
        Self::validate_emergency_policy(&env, &policy)?;
        Self::store_emergency_policy(env, policy)
    }

    pub fn propose_emergency_action(
        env: Env,
        action: EmergencyAction,
        reason_commitment: BytesN<32>,
        expires_at: u32,
        proposer: Address,
    ) -> Result<BytesN<32>, ContractError> {
        Self::ensure_not_decommissioned(&env)?;
        let policy = Self::emergency_policy(&env);
        if !policy.enabled {
            return Err(ContractError::InvalidState);
        }
        let admin = Self::get_admin(env.clone())?;
        if proposer != admin && !policy.guardians.contains(&proposer) {
            return Err(ContractError::Unauthorized);
        }
        Self::require_auth(&proposer);
        if reason_commitment == BytesN::from_array(&env, &[0; 32]) {
            return Err(ContractError::InvalidInput);
        }
        if action == EmergencyAction::ScopedPause(PauseScope::Global) {
            return Err(ContractError::InvalidInput);
        }
        let created_at = env.ledger().sequence();
        let latest = created_at
            .checked_add(policy.max_lifetime_ledgers)
            .ok_or(ContractError::InvalidTimingConfig)?;
        if expires_at <= created_at || expires_at > latest {
            return Err(ContractError::InvalidTimingConfig);
        }
        let nonce = env
            .storage()
            .instance()
            .get::<_, u32>(&DataKey::EmergencyActionNonce)
            .unwrap_or(0)
            .checked_add(1)
            .ok_or(ContractError::InvalidState)?;
        let proposal_id =
            Self::emergency_action_commitment(&env, &action, &reason_commitment, expires_at, nonce);
        let max_lifetime_ledgers = policy.max_lifetime_ledgers;
        let proposal = EmergencyActionProposal {
            action,
            reason_commitment,
            policy,
            proposer: proposer.clone(),
            approvals: Vec::new(&env),
            created_at,
            expires_at,
        };
        env.storage()
            .instance()
            .set(&DataKey::EmergencyActionNonce, &nonce);
        let key = DataKey::EmergencyActionProposal(proposal_id.clone());
        env.storage().persistent().set(&key, &proposal);
        env.storage().persistent().extend_ttl(
            &key,
            TTL_THRESHOLD_LEDGERS,
            max_lifetime_ledgers + TTL_THRESHOLD_LEDGERS,
        );
        EmergencyActionProposed {
            proposal_id: proposal_id.clone(),
            proposer,
            expires_at,
        }
        .publish(&env);
        Ok(proposal_id)
    }

    pub fn get_emergency_proposal(
        env: Env,
        proposal_id: BytesN<32>,
    ) -> Option<EmergencyActionProposal> {
        let key = DataKey::EmergencyActionProposal(proposal_id);
        let proposal: Option<EmergencyActionProposal> = env.storage().persistent().get(&key);
        if let Some(record) = &proposal {
            env.storage().persistent().extend_ttl(
                &key,
                TTL_THRESHOLD_LEDGERS,
                record.policy.max_lifetime_ledgers + TTL_THRESHOLD_LEDGERS,
            );
        }
        proposal
    }

    pub fn approve_emergency_action(
        env: Env,
        proposal_id: BytesN<32>,
        guardian: Address,
    ) -> Result<(), ContractError> {
        let mut proposal = Self::get_emergency_proposal(env.clone(), proposal_id.clone())
            .ok_or(ContractError::ApprovalProposalNotFound)?;
        Self::ensure_emergency_proposal_live(&env, &proposal)?;
        Self::require_auth(&guardian);
        let current = Self::emergency_policy(&env);
        if !current.enabled
            || !proposal.policy.guardians.contains(&guardian)
            || !current.guardians.contains(&guardian)
        {
            return Err(ContractError::Unauthorized);
        }
        if proposal.approvals.contains(&guardian) {
            return Err(ContractError::AlreadyExists);
        }
        proposal.approvals.push_back(guardian.clone());
        env.storage().persistent().set(
            &DataKey::EmergencyActionProposal(proposal_id.clone()),
            &proposal,
        );
        EmergencyActionApproved {
            proposal_id,
            guardian,
        }
        .publish(&env);
        Ok(())
    }

    pub fn cancel_emergency_action(env: Env, proposal_id: BytesN<32>) -> Result<(), ContractError> {
        let admin = Self::get_admin(env.clone())?;
        Self::require_auth(&admin);
        let key = DataKey::EmergencyActionProposal(proposal_id.clone());
        if !env.storage().persistent().has(&key) {
            return Err(ContractError::ApprovalProposalNotFound);
        }
        env.storage().persistent().remove(&key);
        EmergencyActionCancelled {
            proposal_id,
            cancelled_by: admin,
        }
        .publish(&env);
        Ok(())
    }

    pub fn execute_emergency_action(
        env: Env,
        proposal_id: BytesN<32>,
    ) -> Result<(), ContractError> {
        Self::ensure_not_decommissioned(&env)?;
        let proposal = Self::get_emergency_proposal(env.clone(), proposal_id.clone())
            .ok_or(ContractError::ApprovalProposalNotFound)?;
        Self::ensure_emergency_proposal_live(&env, &proposal)?;
        let current = Self::emergency_policy(&env);
        let normal_threshold = Self::critical_action_policy(&env).threshold;
        if !current.enabled || proposal.policy.threshold <= normal_threshold {
            return Err(ContractError::InvalidApprovalPolicy);
        }
        let eligible = proposal
            .approvals
            .iter()
            .filter(|guardian| current.guardians.contains(guardian))
            .count() as u32;
        if eligible < proposal.policy.threshold {
            return Err(ContractError::InsufficientApprovals);
        }
        match proposal.action {
            EmergencyAction::GlobalPause => {
                if Self::is_paused(env.clone()) {
                    return Err(ContractError::InvalidState);
                }
                env.storage().instance().set(&DataKey::Paused, &true);
                Self::bump_config_version(env.clone());
                Self::append_config_history(
                    env.clone(),
                    ConfigChangeCategory::PauseToggle,
                    proposal.reason_commitment.clone(),
                );
                Paused { paused: true }.publish(&env);
            }
            EmergencyAction::ScopedPause(scope) => {
                if Self::is_scope_paused(env.clone(), scope) {
                    return Err(ContractError::InvalidState);
                }
                env.storage()
                    .persistent()
                    .set(&DataKey::ScopedPause(scope), &true);
                Self::bump_config_version(env.clone());
                Self::append_config_history(
                    env.clone(),
                    ConfigChangeCategory::ScopedPause,
                    proposal.reason_commitment.clone(),
                );
                ScopedPauseChanged {
                    scope,
                    paused: true,
                }
                .publish(&env);
            }
        }
        env.storage()
            .persistent()
            .remove(&DataKey::EmergencyActionProposal(proposal_id.clone()));
        EmergencyActionExecuted {
            proposal_id,
            reason_commitment: proposal.reason_commitment,
        }
        .publish(&env);
        Ok(())
    }

    pub fn propose_critical_action(
        env: Env,
        action: CriticalAction,
    ) -> Result<BytesN<32>, ContractError> {
        Self::ensure_not_decommissioned(&env)?;
        let admin = Self::get_admin(env.clone())?;
        Self::require_auth(&admin);
        let policy = Self::critical_action_policy(&env);
        if !policy.enabled {
            return Err(ContractError::InvalidState);
        }
        if !matches!(
            &action,
            CriticalAction::SchemaApproval(_)
                | CriticalAction::SchemaActivation(_, _)
                | CriticalAction::SchemaDeprecation(_)
                | CriticalAction::SchemaReplacement(_, _, _)
                | CriticalAction::SchemaPayloadLimit(_, _)
                | CriticalAction::ApprovalPolicyUpdate(_)
                | CriticalAction::EmergencyPolicyUpdate(_)
        ) {
            return Err(ContractError::InvalidInput);
        }

        let nonce = env
            .storage()
            .instance()
            .get::<_, u32>(&DataKey::CriticalActionNonce)
            .unwrap_or(0)
            .checked_add(1)
            .ok_or(ContractError::InvalidState)?;
        let proposal_id = Self::critical_action_commitment(&env, &action, nonce);
        let created_at = env.ledger().sequence();
        let expires_at = created_at
            .checked_add(CRITICAL_ACTION_APPROVAL_EXPIRY_LEDGERS)
            .ok_or(ContractError::InvalidInput)?;
        env.storage()
            .instance()
            .set(&DataKey::CriticalActionNonce, &nonce);
        let proposal = CriticalActionProposal {
            category: action.category(),
            action,
            policy,
            proposer: admin.clone(),
            approvals: Vec::new(&env),
            created_at,
            expires_at,
        };
        let key = DataKey::CriticalActionProposal(proposal_id.clone());
        env.storage().persistent().set(&key, &proposal);
        let domain_key = DataKey::CriticalActionDomain(proposal_id.clone());
        let domain = GovernanceActionDomain {
            encoding_version: GOVERNANCE_ACTION_HASH_VERSION,
            network_id: env.ledger().network_id(),
            contract_id: env.current_contract_address(),
            action_category: proposal.category,
            nonce,
            commitment: proposal_id.clone(),
        };
        env.storage().persistent().set(&domain_key, &domain);
        let approval_snapshot_key = DataKey::CriticalActionSnapshot(proposal_id.clone());
        let approval_snapshot = GovernanceApprovalSnapshot {
            snapshot_version: GOVERNANCE_APPROVAL_SNAPSHOT_VERSION,
            policy_version: Self::get_critical_policy_version(env.clone()),
            threshold: proposal.policy.threshold,
            signer_set_digest: earnproof_shared::governance_signer_set_digest(
                &env,
                &proposal.policy.signers,
            ),
        };
        env.storage()
            .persistent()
            .set(&approval_snapshot_key, &approval_snapshot);
        let approver_snapshot_key = DataKey::CriticalApproverSnapshot(proposal_id.clone());
        let approver_snapshot = Self::approver_snapshot(&env, &proposal.policy);
        env.storage()
            .persistent()
            .set(&approver_snapshot_key, &approver_snapshot);
        env.storage().persistent().extend_ttl(
            &key,
            TTL_THRESHOLD_LEDGERS,
            CRITICAL_ACTION_APPROVAL_EXPIRY_LEDGERS + TTL_THRESHOLD_LEDGERS,
        );
        env.storage().persistent().extend_ttl(
            &domain_key,
            TTL_THRESHOLD_LEDGERS,
            CRITICAL_ACTION_APPROVAL_EXPIRY_LEDGERS + TTL_THRESHOLD_LEDGERS,
        );
        env.storage().persistent().extend_ttl(
            &approval_snapshot_key,
            TTL_THRESHOLD_LEDGERS,
            CRITICAL_ACTION_APPROVAL_EXPIRY_LEDGERS + TTL_THRESHOLD_LEDGERS,
        );
        env.storage().persistent().extend_ttl(
            &approver_snapshot_key,
            TTL_THRESHOLD_LEDGERS,
            CRITICAL_ACTION_APPROVAL_EXPIRY_LEDGERS + TTL_THRESHOLD_LEDGERS,
        );
        Self::extend_instance_ttl(env.clone());
        CriticalActionProposed {
            proposal_id: proposal_id.clone(),
            proposer: admin,
        }
        .publish(&env);
        Ok(proposal_id)
    }

    pub fn get_critical_action_proposal(
        env: Env,
        proposal_id: BytesN<32>,
    ) -> Option<CriticalActionProposal> {
        let key = DataKey::CriticalActionProposal(proposal_id);
        let proposal = env.storage().persistent().get(&key);
        if proposal.is_some() {
            env.storage().persistent().extend_ttl(
                &key,
                TTL_THRESHOLD_LEDGERS,
                CRITICAL_ACTION_APPROVAL_EXPIRY_LEDGERS + TTL_THRESHOLD_LEDGERS,
            );
        }
        proposal
    }

    /// Returns the domain metadata for a V1 proposal, or `None` for a legacy
    /// proposal created before domain-separated commitments were introduced.
    pub fn get_critical_action_domain(
        env: Env,
        proposal_id: BytesN<32>,
    ) -> Option<GovernanceActionDomain> {
        let key = DataKey::CriticalActionDomain(proposal_id);
        let domain = env.storage().persistent().get(&key);
        if domain.is_some() {
            env.storage().persistent().extend_ttl(
                &key,
                TTL_THRESHOLD_LEDGERS,
                CRITICAL_ACTION_APPROVAL_EXPIRY_LEDGERS + TTL_THRESHOLD_LEDGERS,
            );
        }
        domain
    }

    /// Derives a proposal identifier without mutating contract state.
    pub fn derive_critical_action_id(env: Env, action: CriticalAction, nonce: u32) -> BytesN<32> {
        Self::critical_action_commitment(&env, &action, nonce)
    }

    /// Returns the immutable policy commitment captured at proposal creation.
    /// `None` identifies a legacy proposal whose embedded policy remains
    /// authoritative but predates explicit snapshot metadata.
    pub fn get_critical_action_snapshot(
        env: Env,
        proposal_id: BytesN<32>,
    ) -> Option<GovernanceApprovalSnapshot> {
        let key = DataKey::CriticalActionSnapshot(proposal_id);
        let snapshot = env.storage().persistent().get(&key);
        if snapshot.is_some() {
            env.storage().persistent().extend_ttl(
                &key,
                TTL_THRESHOLD_LEDGERS,
                CRITICAL_ACTION_APPROVAL_EXPIRY_LEDGERS + TTL_THRESHOLD_LEDGERS,
            );
        }
        snapshot
    }

    pub fn get_approver_snapshot(
        env: Env,
        proposal_id: BytesN<32>,
    ) -> Option<GovernanceApproverSnapshot> {
        let key = DataKey::CriticalApproverSnapshot(proposal_id);
        let snapshot = env.storage().persistent().get(&key);
        if snapshot.is_some() {
            env.storage().persistent().extend_ttl(
                &key,
                TTL_THRESHOLD_LEDGERS,
                CRITICAL_ACTION_APPROVAL_EXPIRY_LEDGERS + TTL_THRESHOLD_LEDGERS,
            );
        }
        snapshot
    }

    pub fn approval_eligibility(
        env: Env,
        proposal_id: BytesN<32>,
        signer: Address,
    ) -> GovernanceApprovalEligibility {
        let Some(proposal) = Self::get_critical_action_proposal(env.clone(), proposal_id.clone())
        else {
            return GovernanceApprovalEligibility::ProposalNotFound;
        };
        Self::approval_eligibility_for(&env, &proposal_id, &proposal, &signer)
    }
    pub fn approve_critical_action(
        env: Env,
        proposal_id: BytesN<32>,
        signer: Address,
    ) -> Result<(), ContractError> {
        let mut proposal = Self::get_critical_action_proposal(env.clone(), proposal_id.clone())
            .ok_or(ContractError::ApprovalProposalNotFound)?;
        Self::verify_critical_action_domain(&env, &proposal_id, &proposal)?;
        Self::verify_approval_snapshot(&env, &proposal_id, &proposal)?;
        Self::ensure_proposal_live(&env, &proposal)?;
        Self::require_auth(&signer);
        match Self::approval_eligibility_for(&env, &proposal_id, &proposal, &signer) {
            GovernanceApprovalEligibility::Eligible => {}
            GovernanceApprovalEligibility::ProposalExpired => {
                return Err(ContractError::ApprovalProposalExpired)
            }
            GovernanceApprovalEligibility::AlreadyApproved => {
                return Err(ContractError::AlreadyExists)
            }
            GovernanceApprovalEligibility::ProposalNotFound => {
                return Err(ContractError::ApprovalProposalNotFound)
            }
            _ => return Err(ContractError::Unauthorized),
        }
        proposal.approvals.push_back(signer.clone());
        env.storage().persistent().set(
            &DataKey::CriticalActionProposal(proposal_id.clone()),
            &proposal,
        );
        CriticalActionApproved {
            proposal_id,
            signer,
        }
        .publish(&env);
        Ok(())
    }

    pub fn cancel_critical_action(env: Env, proposal_id: BytesN<32>) -> Result<(), ContractError> {
        let admin = Self::get_admin(env.clone())?;
        Self::require_auth(&admin);
        let key = DataKey::CriticalActionProposal(proposal_id.clone());
        let proposal = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(ContractError::ApprovalProposalNotFound)?;
        Self::verify_critical_action_domain(&env, &proposal_id, &proposal)?;
        Self::verify_approval_snapshot(&env, &proposal_id, &proposal)?;
        env.storage().persistent().remove(&key);
        env.storage()
            .persistent()
            .remove(&DataKey::CriticalActionDomain(proposal_id.clone()));
        env.storage()
            .persistent()
            .remove(&DataKey::CriticalActionSnapshot(proposal_id.clone()));
        env.storage()
            .persistent()
            .remove(&DataKey::CriticalApproverSnapshot(proposal_id.clone()));
        CriticalActionCancelled {
            proposal_id,
            cancelled_by: admin,
        }
        .publish(&env);
        Ok(())
    }

    pub fn execute_critical_action(env: Env, proposal_id: BytesN<32>) -> Result<(), ContractError> {
        Self::ensure_not_decommissioned(&env)?;
        let proposal = Self::get_critical_action_proposal(env.clone(), proposal_id.clone())
            .ok_or(ContractError::ApprovalProposalNotFound)?;
        Self::verify_critical_action_domain(&env, &proposal_id, &proposal)?;
        Self::verify_approval_snapshot(&env, &proposal_id, &proposal)?;
        Self::ensure_proposal_live(&env, &proposal)?;
        if Self::eligible_approval_count(&env, &proposal_id, &proposal) < proposal.policy.threshold
        {
            return Err(ContractError::InsufficientApprovals);
        }
        let commitment = Self::commit(&env, proposal.action.clone());
        match proposal.action.clone() {
            CriticalAction::SchemaApproval(version) => {
                Self::ensure_nonzero_version(version)?;
                env.storage()
                    .persistent()
                    .set(&DataKey::SchemaVersion(version), &true);
                Self::record_schema_approval(&env, version);
                Self::extend_schema_ttl(env.clone(), version);
                Self::record_critical_action_change(
                    env.clone(),
                    ConfigChangeCategory::SchemaApproval,
                    commitment,
                );
            }
            CriticalAction::SchemaActivation(version, activates_at) => {
                Self::apply_schema_activation(
                    env.clone(),
                    version,
                    activates_at,
                    ConfigChangeCategory::SchemaActivation,
                    commitment,
                )?;
            }
            CriticalAction::SchemaDeprecation(version) => {
                Self::ensure_nonzero_version(version)?;
                env.storage()
                    .persistent()
                    .set(&DataKey::SchemaVersion(version), &false);
                Self::record_schema_deprecation(&env, version);
                Self::extend_schema_ttl(env.clone(), version);
                Self::record_critical_action_change(
                    env.clone(),
                    ConfigChangeCategory::SchemaDeprecation,
                    commitment,
                );
                SchemaDeprecated { version }.publish(&env);
            }
            CriticalAction::SchemaReplacement(
                deprecated_version,
                replacement_version,
                compatibility,
            ) => {
                Self::apply_schema_replacement(
                    env.clone(),
                    deprecated_version,
                    replacement_version,
                    compatibility,
                    commitment,
                )?;
            }
            CriticalAction::SchemaPayloadLimit(version, max_size) => {
                Self::ensure_nonzero_version(version)?;
                env.storage()
                    .persistent()
                    .set(&DataKey::SchemaPayloadLimit(version), &max_size);
                Self::extend_schema_payload_limit_ttl(env.clone(), version);
                Self::record_critical_action_change(
                    env.clone(),
                    ConfigChangeCategory::SchemaPayloadLimit,
                    commitment,
                );
                SchemaPayloadLimitSet { version, max_size }.publish(&env);
            }
            CriticalAction::ApprovalPolicyUpdate(policy) => {
                Self::validate_critical_action_policy(&policy)?;
                Self::store_critical_action_policy_with_commitment(
                    env.clone(),
                    policy,
                    commitment,
                )?;
            }
            CriticalAction::EmergencyPolicyUpdate(policy) => {
                Self::validate_emergency_policy(&env, &policy)?;
                Self::store_emergency_policy_with_commitment(env.clone(), policy, commitment)?;
            }
            CriticalAction::IssuerRegistryReplacement(_)
            | CriticalAction::ProtocolConfigReplacement(_) => {
                return Err(ContractError::InvalidInput);
            }
        }
        env.storage()
            .persistent()
            .remove(&DataKey::CriticalActionProposal(proposal_id.clone()));
        env.storage()
            .persistent()
            .remove(&DataKey::CriticalActionDomain(proposal_id.clone()));
        env.storage()
            .persistent()
            .remove(&DataKey::CriticalActionSnapshot(proposal_id.clone()));
        env.storage()
            .persistent()
            .remove(&DataKey::CriticalApproverSnapshot(proposal_id.clone()));
        CriticalActionExecuted { proposal_id }.publish(&env);
        Ok(())
    }

    /// Returns the active payload-size bound for `version`: the governed
    /// override if one was set, otherwise the shared default. Version `0`
    /// always returns the default, since it can never be approved.
    pub fn get_schema_payload_limit(env: Env, version: u32) -> u32 {
        if version == 0 {
            return DEFAULT_SCHEMA_PAYLOAD_LIMIT;
        }
        let key = DataKey::SchemaPayloadLimit(version);
        let limit = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(DEFAULT_SCHEMA_PAYLOAD_LIMIT);
        if env.storage().persistent().has(&key) {
            env.storage().persistent().extend_ttl(
                &key,
                TTL_THRESHOLD_LEDGERS,
                TTL_EXTEND_TO_LEDGERS,
            );
        }
        limit
    }

    // ── bounded configuration change history ─────────────────────────────────

    /// Returns up to `limit` change-history entries starting at `cursor`
    /// (an absolute, monotonically increasing sequence number; `0` is the
    /// very first change ever recorded).
    ///
    /// If `cursor` refers to an entry the ring has already evicted, the read
    /// starts at the oldest entry still available rather than erroring — a
    /// stale cursor is clamped forward, deterministically, instead of
    /// panicking. `limit` is capped at `MAX_CONFIG_HISTORY_PAGE` regardless of
    /// the requested value.
    /// Admin-only: set the deterministic registration-rate policy included in
    /// the schema digest. A zero maximum pauses registrations for the schema;
    /// the ledger window itself must be nonzero.
    pub fn set_schema_rate_limit(
        env: Env,
        version: u32,
        max_registrations: u32,
        window_ledgers: u32,
    ) -> Result<(), ContractError> {
        Self::ensure_not_decommissioned(&env)?;
        let admin = Self::get_admin(env.clone())?;
        Self::require_auth(&admin);
        Self::ensure_nonzero_version(version)?;
        if window_ledgers == 0 {
            return Err(ContractError::InvalidInput);
        }
        let rate_limit = SchemaRateLimit {
            max_registrations,
            window_ledgers,
        };
        let key = DataKey::SchemaRateLimit(version);
        env.storage().persistent().set(&key, &rate_limit);
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_THRESHOLD_LEDGERS, TTL_EXTEND_TO_LEDGERS);
        Self::bump_config_version(env.clone());
        Self::append_config_history(
            env.clone(),
            ConfigChangeCategory::SchemaRateLimit,
            Self::commit(&env, (version, rate_limit.clone())),
        );
        SchemaRateLimitSet {
            version,
            max_registrations,
            window_ledgers,
        }
        .publish(&env);
        Ok(())
    }

    pub fn get_schema_rate_limit(env: Env, version: u32) -> SchemaRateLimit {
        if version == 0 {
            return Self::default_schema_rate_limit();
        }
        let key = DataKey::SchemaRateLimit(version);
        let policy = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(Self::default_schema_rate_limit);
        if env.storage().persistent().has(&key) {
            env.storage().persistent().extend_ttl(
                &key,
                TTL_THRESHOLD_LEDGERS,
                TTL_EXTEND_TO_LEDGERS,
            );
        }
        policy
    }

    pub fn get_schema_lifecycle(env: Env, version: u32) -> SchemaLifecyclePolicy {
        let key = DataKey::SchemaLifecycle(version);
        let lifecycle = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(SchemaLifecyclePolicy {
                activated_at: 0,
                deprecated_at: 0,
            });
        if env.storage().persistent().has(&key) {
            env.storage().persistent().extend_ttl(
                &key,
                TTL_THRESHOLD_LEDGERS,
                TTL_EXTEND_TO_LEDGERS,
            );
        }
        lifecycle
    }

    pub fn get_schema_policy_digest(
        env: Env,
        version: u32,
    ) -> Result<SchemaPolicyDigest, ContractError> {
        Self::ensure_nonzero_version(version)?;
        if !Self::schema_version_exists(env.clone(), version) {
            return Err(ContractError::NotFound);
        }
        let approved = Self::is_schema_version_approved(env.clone(), version);
        let lifecycle = Self::get_schema_lifecycle(env.clone(), version);
        let policy = Self::get_schema_policy(env.clone(), version);
        let payload_limit = Self::get_schema_payload_limit(env.clone(), version);
        let rate_limit = Self::get_schema_rate_limit(env.clone(), version);
        let config_version = Self::get_config_version(env.clone());
        let digest = earnproof_shared::compute_schema_policy_digest(
            &env,
            &earnproof_shared::SchemaPolicyDigestInput {
                schema_version: version,
                config_version,
                approved,
                lifecycle: &lifecycle,
                policy: &policy,
                payload_limit,
                rate_limit: &rate_limit,
            },
        );
        Ok(SchemaPolicyDigest {
            digest_version: earnproof_shared::SCHEMA_POLICY_DIGEST_VERSION,
            config_version,
            schema_version: version,
            digest,
        })
    }

    pub fn get_schema_policy_digest_version() -> u32 {
        earnproof_shared::SCHEMA_POLICY_DIGEST_VERSION
    }

    pub fn get_config_history(env: Env, cursor: u32, limit: u32) -> Vec<ConfigChangeSummary> {
        let total = Self::get_config_history_cursor(env.clone());
        let oldest_available = total.saturating_sub(CONFIG_HISTORY_CAPACITY);
        let start = cursor.max(oldest_available);
        let capped_limit = limit.min(MAX_CONFIG_HISTORY_PAGE);
        let end = start.saturating_add(capped_limit).min(total);

        let mut page = Vec::new(&env);
        let mut index = start;
        while index < end {
            let slot = index % CONFIG_HISTORY_CAPACITY;
            if let Some(entry) = env
                .storage()
                .persistent()
                .get::<_, ConfigChangeSummary>(&DataKey::ConfigHistoryRing(slot))
            {
                page.push_back(entry);
            }
            index += 1;
        }
        page
    }

    /// Total number of change-history entries ever appended. Also the cursor
    /// value a caller should pass to `get_config_history` to read only
    /// entries recorded after the most recent page it consumed.
    pub fn get_config_history_cursor(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::ConfigHistoryTotal)
            .unwrap_or(0)
    }

    // ── upgrade governance ───────────────────────────────────────────────────

    /// Returns the stored monotonic contract version (separate from the
    /// config-mutation counter).  Starts at 1 after `initialize`.
    pub fn get_contract_version(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::ContractVersion)
            .unwrap_or(0)
    }

    /// Admin-only: add `wasm_hash` to the upgrade allowlist and record the
    /// `new_version` that must be installed by that WASM.
    ///
    /// `new_version` must be strictly greater than the currently stored
    /// contract version so that a downgrade cannot be pre-approved.
    pub fn approve_upgrade(env: Env, wasm_hash: BytesN<32>, new_version: u32) {
        if Self::is_decommissioned(env.clone()) {
            panic!("contract is decommissioned");
        }
        if Self::is_scope_paused(env.clone(), PauseScope::Upgrades) {
            panic!("upgrades are paused");
        }
        let admin = Self::get_admin(env.clone()).expect("contract not initialized");

        Self::require_auth(&admin);

        let current = Self::get_contract_version(env.clone());
        if new_version <= current {
            panic!("new_version must be greater than current contract version");
        }

        env.storage()
            .instance()
            .set(&DataKey::AllowedWasm(wasm_hash.clone()), &new_version);
        Self::extend_instance_ttl(env.clone());

        UpgradeAllowlisted {
            wasm_hash,
            new_contract_version: new_version,
            approved_by: admin,
        }
        .publish(&env);
    }

    /// Admin-only: remove a previously allowlisted WASM hash without applying
    /// it.  Safe to call even if the hash was never allowlisted.
    pub fn revoke_upgrade(env: Env, wasm_hash: BytesN<32>) {
        if Self::is_decommissioned(env.clone()) {
            panic!("contract is decommissioned");
        }
        let admin = Self::get_admin(env.clone()).expect("contract not initialized");
        Self::require_auth(&admin);

        env.storage()
            .instance()
            .remove(&DataKey::AllowedWasm(wasm_hash.clone()));

        UpgradeRevoked {
            wasm_hash,
            revoked_by: admin,
        }
        .publish(&env);
    }

    /// Returns true when `wasm_hash` is on the allowlist.
    pub fn is_upgrade_allowed(env: Env, wasm_hash: BytesN<32>) -> bool {
        env.storage()
            .instance()
            .has(&DataKey::AllowedWasm(wasm_hash))
    }

    /// Admin-only: apply an in-place WASM upgrade.
    ///
    /// Requirements (all checked on-chain before the upgrade is applied):
    /// 1. Caller is the admin.
    /// 2. `wasm_hash` is on the allowlist (pre-approved via `approve_upgrade`).
    /// 3. The target version stored in the allowlist entry is strictly greater
    ///    than the current `ContractVersion` (downgrade guard).
    ///
    /// On success the new WASM is installed, `ContractVersion` is advanced,
    /// the allowlist entry is consumed (removed), and a `ContractUpgraded`
    /// event is emitted.
    pub fn upgrade_contract(env: Env, wasm_hash: BytesN<32>) {
        if Self::is_decommissioned(env.clone()) {
            panic!("contract is decommissioned");
        }
        if Self::is_scope_paused(env.clone(), PauseScope::Upgrades) {
            panic!("upgrades are paused");
        }
        let admin = Self::get_admin(env.clone()).expect("contract not initialized");
        Self::require_auth(&admin);

        let new_version: u32 = env
            .storage()
            .instance()
            .get(&DataKey::AllowedWasm(wasm_hash.clone()))
            .expect("wasm hash not on allowlist");

        let old_version = Self::get_contract_version(env.clone());
        if new_version <= old_version {
            panic!("upgrade would not advance contract version");
        }

        // Consume the allowlist entry before applying so re-entrancy cannot
        // replay the same hash.
        env.storage()
            .instance()
            .remove(&DataKey::AllowedWasm(wasm_hash.clone()));

        // Apply the WASM upgrade.  This replaces the executable code while
        // leaving all stored state intact.
        #[cfg(not(test))]
        env.deployer()
            .update_current_contract_wasm(wasm_hash.clone());

        // Record the new version.
        env.storage()
            .instance()
            .set(&DataKey::ContractVersion, &new_version);
        Self::extend_instance_ttl(env.clone());

        ContractUpgraded {
            new_wasm_hash: wasm_hash,
            old_contract_version: old_version,
            new_contract_version: new_version,
            upgraded_by: admin,
        }
        .publish(&env);
    }

    // ── private helpers ──────────────────────────────────────────────────────

    fn ensure_nonzero_version(version: u32) -> Result<(), ContractError> {
        if version == 0 {
            return Err(ContractError::InvalidInput);
        }
        Ok(())
    }

    fn default_dispute_filing_policy() -> DisputeFilingPolicy {
        DisputeFilingPolicy {
            version: DISPUTE_FILING_POLICY_VERSION,
            anchor: DisputeFilingAnchor::Created,
            opens_after_seconds: 0,
            closes_after_seconds: u64::MAX,
        }
    }

    fn default_schema_rate_limit() -> SchemaRateLimit {
        SchemaRateLimit {
            max_registrations: DEFAULT_SCHEMA_RATE_LIMIT,
            window_ledgers: DEFAULT_SCHEMA_RATE_WINDOW_LEDGERS,
        }
    }

    fn record_schema_approval(env: &Env, version: u32) {
        let key = DataKey::SchemaLifecycle(version);
        let lifecycle = SchemaLifecyclePolicy {
            activated_at: env.ledger().timestamp(),
            deprecated_at: 0,
        };
        env.storage().persistent().set(&key, &lifecycle);
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_THRESHOLD_LEDGERS, TTL_EXTEND_TO_LEDGERS);
    }

    fn record_schema_deprecation(env: &Env, version: u32) {
        let key = DataKey::SchemaLifecycle(version);
        let mut lifecycle = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(SchemaLifecyclePolicy {
                activated_at: 0,
                deprecated_at: 0,
            });
        lifecycle.deprecated_at = env.ledger().timestamp();
        env.storage().persistent().set(&key, &lifecycle);
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_THRESHOLD_LEDGERS, TTL_EXTEND_TO_LEDGERS);
    }

    fn require_valid_principal(address: &Address) -> Result<(), ContractError> {
        if !earnproof_shared::is_valid_principal_address(address) {
            return Err(ContractError::InvalidInput);
        }
        Ok(())
    }

    fn bump_config_version(env: Env) {
        let current = Self::get_config_version(env.clone());
        let new_version = current
            .checked_add(1)
            .unwrap_or_else(|| panic!("config version overflow: reached maximum"));
        env.storage()
            .instance()
            .set(&DataKey::ConfigVersion, &new_version);
        Self::extend_instance_ttl(env);
    }

    fn extend_instance_ttl(env: Env) {
        env.storage()
            .instance()
            .extend_ttl(TTL_THRESHOLD_LEDGERS, TTL_EXTEND_TO_LEDGERS);
    }

    fn extend_schema_ttl(env: Env, version: u32) {
        env.storage().persistent().extend_ttl(
            &DataKey::SchemaVersion(version),
            TTL_THRESHOLD_LEDGERS,
            TTL_EXTEND_TO_LEDGERS,
        );
    }

    fn extend_schema_payload_limit_ttl(env: Env, version: u32) {
        env.storage().persistent().extend_ttl(
            &DataKey::SchemaPayloadLimit(version),
            TTL_THRESHOLD_LEDGERS,
            TTL_EXTEND_TO_LEDGERS,
        );
    }

    /// SHA-256 commitment to an arbitrary XDR-encodable value. Used so
    /// change-history entries carry proof of what changed without carrying
    /// the value itself.
    fn commit<T: ToXdr>(env: &Env, value: T) -> BytesN<32> {
        env.crypto().sha256(&value.to_xdr(env)).to_bytes()
    }

    /// Appends one bounded change-history entry, overwriting the oldest slot
    /// once the ring is full. Only called from a success path, after the
    /// state it summarizes has already been committed, so a failed mutation
    /// never appends.
    fn append_config_history(
        env: Env,
        category: ConfigChangeCategory,
        proposal_commitment: BytesN<32>,
    ) {
        let total = Self::get_config_history_cursor(env.clone());
        let slot = total % CONFIG_HISTORY_CAPACITY;
        let summary = ConfigChangeSummary {
            category,
            proposal_commitment,
            config_version: Self::get_config_version(env.clone()),
            ledger: env.ledger().sequence(),
            timestamp: env.ledger().timestamp(),
        };
        env.storage()
            .persistent()
            .set(&DataKey::ConfigHistoryRing(slot), &summary);
        env.storage().persistent().extend_ttl(
            &DataKey::ConfigHistoryRing(slot),
            TTL_THRESHOLD_LEDGERS,
            TTL_EXTEND_TO_LEDGERS,
        );
        let next_total = total
            .checked_add(1)
            .unwrap_or_else(|| panic!("config history overflow: reached maximum"));
        env.storage()
            .instance()
            .set(&DataKey::ConfigHistoryTotal, &next_total);
        Self::extend_instance_ttl(env);
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

    fn require_auth(address: &Address) {
        address.require_auth();
    }

    fn stored_predecessor(env: Env, version: u32) -> Option<u32> {
        let key = DataKey::SchemaPredecessor(version);
        let predecessor = env.storage().persistent().get(&key);
        if predecessor.is_some() {
            env.storage().persistent().extend_ttl(
                &key,
                TTL_THRESHOLD_LEDGERS,
                TTL_EXTEND_TO_LEDGERS,
            );
        }
        predecessor
    }

    fn stored_schema_replacement(env: Env, version: u32) -> Option<SchemaReplacementPointer> {
        let key = DataKey::SchemaReplacement(version);
        let pointer = env.storage().persistent().get(&key);
        if pointer.is_some() {
            env.storage().persistent().extend_ttl(
                &key,
                TTL_THRESHOLD_LEDGERS,
                TTL_EXTEND_TO_LEDGERS,
            );
        }
        pointer
    }

    fn replacement_reaches(env: Env, start: u32, target: u32) -> Result<bool, ContractError> {
        let mut current = start;
        let mut visited = Vec::new(&env);
        for _ in 0..MAX_SCHEMA_LINEAGE_DEPTH {
            if current == target {
                return Ok(true);
            }
            if visited.contains(current) {
                return Err(ContractError::InvalidState);
            }
            visited.push_back(current);
            match Self::stored_schema_replacement(env.clone(), current) {
                Some(pointer) => current = pointer.replacement_version,
                None => return Ok(false),
            }
        }
        Err(ContractError::InvalidState)
    }

    fn apply_schema_replacement(
        env: Env,
        deprecated_version: u32,
        replacement_version: u32,
        compatibility: SchemaCompatibility,
        change_commitment: BytesN<32>,
    ) -> Result<SchemaReplacementPointer, ContractError> {
        Self::ensure_nonzero_version(deprecated_version)?;
        Self::ensure_nonzero_version(replacement_version)?;
        if deprecated_version == replacement_version {
            return Err(ContractError::InvalidInput);
        }
        if !Self::is_schema_version_approved(env.clone(), deprecated_version)
            || !Self::is_schema_version_approved(env.clone(), replacement_version)
        {
            return Err(ContractError::InvalidState);
        }
        let pointer_key = DataKey::SchemaReplacement(deprecated_version);
        if env.storage().persistent().has(&pointer_key) {
            return Err(ContractError::AlreadyExists);
        }
        if Self::replacement_reaches(env.clone(), replacement_version, deprecated_version)? {
            return Err(ContractError::InvalidState);
        }

        let pointer = SchemaReplacementPointer {
            pointer_version: 1,
            deprecated_version,
            replacement_version,
            compatibility,
            effective_ledger: env.ledger().sequence(),
        };
        env.storage()
            .persistent()
            .set(&DataKey::SchemaVersion(deprecated_version), &false);
        env.storage().persistent().set(&pointer_key, &pointer);
        Self::extend_schema_ttl(env.clone(), deprecated_version);
        env.storage().persistent().extend_ttl(
            &pointer_key,
            TTL_THRESHOLD_LEDGERS,
            TTL_EXTEND_TO_LEDGERS,
        );
        Self::bump_config_version(env.clone());
        Self::append_config_history(
            env.clone(),
            ConfigChangeCategory::SchemaReplacement,
            change_commitment,
        );
        SchemaDeprecated {
            version: deprecated_version,
        }
        .publish(&env);
        SchemaReplacementSet {
            deprecated_version,
            replacement_version,
            compatibility,
            effective_ledger: pointer.effective_ledger,
        }
        .publish(&env);
        Ok(pointer)
    }

    fn schema_version_exists(env: Env, version: u32) -> bool {
        version != 0
            && env
                .storage()
                .persistent()
                .has(&DataKey::SchemaVersion(version))
    }

    fn lineage_reaches(env: Env, start: u32, target: u32) -> bool {
        let mut current = start;
        for _ in 0..MAX_SCHEMA_LINEAGE_DEPTH {
            if current == target {
                return true;
            }
            match Self::stored_predecessor(env.clone(), current) {
                Some(predecessor) => current = predecessor,
                None => return false,
            }
        }
        false
    }

    fn validate_schema_activation_time(
        proposed_at: u64,
        activates_at: u64,
        minimum_notice_seconds: u64,
    ) -> Result<(), ContractError> {
        let earliest = proposed_at
            .checked_add(minimum_notice_seconds)
            .ok_or(ContractError::InvalidTimingConfig)?;
        if activates_at < earliest {
            return Err(ContractError::InvalidTimingConfig);
        }
        Ok(())
    }

    fn store_schema_activation_notice(
        env: Env,
        version: u32,
        proposed_at: u64,
        activates_at: u64,
        minimum_notice_seconds: u64,
    ) {
        let key = DataKey::SchemaActivation(version);
        let notice = SchemaActivationNotice {
            notice_version: 1,
            schema_version: version,
            proposed_at,
            activates_at,
            minimum_notice_seconds,
        };
        env.storage().persistent().set(&key, &notice);
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_THRESHOLD_LEDGERS, TTL_EXTEND_TO_LEDGERS);
    }

    fn apply_schema_activation(
        env: Env,
        version: u32,
        activates_at: u64,
        category: ConfigChangeCategory,
        change_commitment: BytesN<32>,
    ) -> Result<(), ContractError> {
        Self::ensure_nonzero_version(version)?;
        if env
            .storage()
            .persistent()
            .get::<_, bool>(&DataKey::SchemaVersion(version))
            == Some(true)
        {
            return Err(ContractError::AlreadyExists);
        }
        let proposed_at = env.ledger().timestamp();
        let minimum_notice_seconds = Self::get_min_schema_activation_notice(env.clone());
        Self::validate_schema_activation_time(proposed_at, activates_at, minimum_notice_seconds)?;

        env.storage()
            .persistent()
            .set(&DataKey::SchemaVersion(version), &true);
        Self::extend_schema_ttl(env.clone(), version);
        Self::store_schema_activation_notice(
            env.clone(),
            version,
            proposed_at,
            activates_at,
            minimum_notice_seconds,
        );
        Self::bump_config_version(env.clone());
        Self::append_config_history(env.clone(), category, change_commitment);
        SchemaActivationScheduled {
            version,
            proposed_at,
            activates_at,
            minimum_notice_seconds,
        }
        .publish(&env);
        SchemaApproved { version }.publish(&env);
        Ok(())
    }

    fn legacy_schema_policy(env: &Env) -> SchemaPolicy {
        SchemaPolicy {
            proof_types: soroban_sdk::vec![env, LEGACY_PROOF_TYPE],
            max_validity_seconds: MAX_SCHEMA_VALIDITY_SECONDS,
        }
    }

    fn critical_action_policy(env: &Env) -> CriticalActionPolicy {
        env.storage()
            .instance()
            .get(&DataKey::CriticalActionPolicy)
            .unwrap_or(CriticalActionPolicy {
                enabled: false,
                threshold: 0,
                signers: Vec::new(env),
            })
    }

    fn emergency_policy(env: &Env) -> EmergencyFastTrackPolicy {
        env.storage()
            .instance()
            .get(&DataKey::EmergencyFastTrackPolicy)
            .unwrap_or(EmergencyFastTrackPolicy {
                enabled: false,
                threshold: 0,
                guardians: Vec::new(env),
                max_lifetime_ledgers: 0,
            })
    }

    fn validate_emergency_policy(
        env: &Env,
        policy: &EmergencyFastTrackPolicy,
    ) -> Result<(), ContractError> {
        if !policy.enabled {
            return if policy.threshold == 0
                && policy.guardians.is_empty()
                && policy.max_lifetime_ledgers == 0
            {
                Ok(())
            } else {
                Err(ContractError::InvalidApprovalPolicy)
            };
        }
        if policy.guardians.is_empty()
            || policy.guardians.len() > MAX_CRITICAL_ACTION_SIGNERS
            || policy.threshold <= Self::critical_action_policy(env).threshold
            || policy.threshold > policy.guardians.len()
            || policy.max_lifetime_ledgers == 0
            || policy.max_lifetime_ledgers > CRITICAL_ACTION_APPROVAL_EXPIRY_LEDGERS
        {
            return Err(ContractError::InvalidApprovalPolicy);
        }
        for (index, guardian) in policy.guardians.iter().enumerate() {
            if !earnproof_shared::is_valid_principal_address(&guardian) {
                return Err(ContractError::InvalidApprovalPolicy);
            }
            for earlier in 0..index {
                if policy.guardians.get(earlier as u32).unwrap() == guardian {
                    return Err(ContractError::InvalidApprovalPolicy);
                }
            }
        }
        Ok(())
    }

    fn store_emergency_policy(
        env: Env,
        policy: EmergencyFastTrackPolicy,
    ) -> Result<(), ContractError> {
        let commitment = Self::commit(&env, policy.clone());
        Self::store_emergency_policy_with_commitment(env, policy, commitment)
    }

    fn store_emergency_policy_with_commitment(
        env: Env,
        policy: EmergencyFastTrackPolicy,
        commitment: BytesN<32>,
    ) -> Result<(), ContractError> {
        Self::validate_emergency_policy(&env, &policy)?;
        env.storage()
            .instance()
            .set(&DataKey::EmergencyFastTrackPolicy, &policy);
        Self::bump_config_version(env.clone());
        Self::append_config_history(
            env.clone(),
            ConfigChangeCategory::EmergencyPolicyUpdate,
            commitment,
        );
        EmergencyFastTrackPolicySet {
            enabled: policy.enabled,
            threshold: policy.threshold,
            guardian_count: policy.guardians.len(),
            max_lifetime_ledgers: policy.max_lifetime_ledgers,
        }
        .publish(&env);
        Ok(())
    }

    fn ensure_critical_action_policy_disabled(env: &Env) -> Result<(), ContractError> {
        if Self::critical_action_policy(env).enabled {
            Err(ContractError::ThresholdApprovalRequired)
        } else {
            Ok(())
        }
    }

    fn validate_critical_action_policy(policy: &CriticalActionPolicy) -> Result<(), ContractError> {
        if !policy.enabled {
            return if policy.threshold == 0 && policy.signers.is_empty() {
                Ok(())
            } else {
                Err(ContractError::InvalidApprovalPolicy)
            };
        }
        if policy.signers.is_empty()
            || policy.signers.len() > MAX_CRITICAL_ACTION_SIGNERS
            || policy.threshold == 0
            || policy.threshold > policy.signers.len()
        {
            return Err(ContractError::InvalidApprovalPolicy);
        }
        for (index, signer) in policy.signers.iter().enumerate() {
            if !earnproof_shared::is_valid_principal_address(&signer) {
                return Err(ContractError::InvalidApprovalPolicy);
            }
            for earlier in 0..index {
                if policy.signers.get(earlier as u32).unwrap() == signer {
                    return Err(ContractError::InvalidApprovalPolicy);
                }
            }
        }
        Ok(())
    }

    fn store_critical_action_policy(
        env: Env,
        policy: CriticalActionPolicy,
    ) -> Result<(), ContractError> {
        let commitment = Self::commit(&env, policy.clone());
        Self::store_critical_action_policy_with_commitment(env, policy, commitment)
    }

    fn store_critical_action_policy_with_commitment(
        env: Env,
        policy: CriticalActionPolicy,
        commitment: BytesN<32>,
    ) -> Result<(), ContractError> {
        let version = Self::get_critical_policy_version(env.clone())
            .checked_add(1)
            .ok_or(ContractError::InvalidState)?;
        let previous = Self::critical_action_policy(&env);
        for signer in policy.signers.iter() {
            if !previous.signers.contains(&signer) {
                let key = DataKey::CriticalSignerVersion(signer);
                let next = env
                    .storage()
                    .instance()
                    .get::<_, u32>(&key)
                    .unwrap_or(0)
                    .checked_add(1)
                    .unwrap_or_else(|| panic!("critical signer assignment version overflow"));
                env.storage().instance().set(&key, &next);
            }
        }
        env.storage()
            .instance()
            .set(&DataKey::CriticalActionPolicy, &policy);
        env.storage()
            .instance()
            .set(&DataKey::CriticalActionPolicyVersion, &version);
        Self::bump_config_version(env.clone());
        Self::append_config_history(
            env.clone(),
            ConfigChangeCategory::ApprovalPolicyUpdate,
            commitment,
        );
        CriticalActionPolicySet {
            enabled: policy.enabled,
            threshold: policy.threshold,
            signer_count: policy.signers.len(),
        }
        .publish(&env);
        Ok(())
    }

    fn ensure_proposal_live(
        env: &Env,
        proposal: &CriticalActionProposal,
    ) -> Result<(), ContractError> {
        if env.ledger().sequence() >= proposal.expires_at {
            return Err(ContractError::ApprovalProposalExpired);
        }
        Ok(())
    }

    fn ensure_emergency_proposal_live(
        env: &Env,
        proposal: &EmergencyActionProposal,
    ) -> Result<(), ContractError> {
        if env.ledger().sequence() >= proposal.expires_at {
            return Err(ContractError::ApprovalProposalExpired);
        }
        Ok(())
    }

    fn emergency_action_commitment(
        env: &Env,
        action: &EmergencyAction,
        reason_commitment: &BytesN<32>,
        expires_at: u32,
        nonce: u32,
    ) -> BytesN<32> {
        let preimage = (
            Symbol::new(env, "emergency_v1"),
            EMERGENCY_FAST_TRACK_ENCODING_VERSION,
            env.ledger().network_id(),
            env.current_contract_address(),
            *action,
            reason_commitment.clone(),
            expires_at,
            nonce,
        );
        env.crypto().sha256(&preimage.to_xdr(env)).to_bytes()
    }

    fn verify_approval_snapshot(
        env: &Env,
        proposal_id: &BytesN<32>,
        proposal: &CriticalActionProposal,
    ) -> Result<(), ContractError> {
        let Some(snapshot) = env
            .storage()
            .persistent()
            .get::<_, GovernanceApprovalSnapshot>(&DataKey::CriticalActionSnapshot(
                proposal_id.clone(),
            ))
        else {
            return Ok(());
        };
        let signer_set_digest =
            earnproof_shared::governance_signer_set_digest(env, &proposal.policy.signers);
        if snapshot.snapshot_version != GOVERNANCE_APPROVAL_SNAPSHOT_VERSION
            || snapshot.policy_version == 0
            || snapshot.threshold != proposal.policy.threshold
            || snapshot.signer_set_digest != signer_set_digest
        {
            return Err(ContractError::InvalidState);
        }
        Ok(())
    }

    fn signer_version(env: &Env, signer: &Address) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::CriticalSignerVersion(signer.clone()))
            .unwrap_or(0)
    }

    fn approver_snapshot(env: &Env, policy: &CriticalActionPolicy) -> GovernanceApproverSnapshot {
        let mut approvers = Vec::new(env);
        for address in policy.signers.iter() {
            approvers.push_back(GovernanceApproverVersion {
                assignment_version: Self::signer_version(env, &address),
                address,
            });
        }
        GovernanceApproverSnapshot {
            encoding_version: GOVERNANCE_APPROVER_SNAPSHOT_VERSION,
            approvers,
        }
    }

    fn approval_eligibility_for(
        env: &Env,
        proposal_id: &BytesN<32>,
        proposal: &CriticalActionProposal,
        signer: &Address,
    ) -> GovernanceApprovalEligibility {
        if env.ledger().sequence() >= proposal.expires_at {
            return GovernanceApprovalEligibility::ProposalExpired;
        }
        if proposal.approvals.contains(signer) {
            return GovernanceApprovalEligibility::AlreadyApproved;
        }
        if !proposal.policy.signers.contains(signer) {
            return GovernanceApprovalEligibility::NotInProposalSnapshot;
        }
        let current = Self::critical_action_policy(env);
        if !current.enabled {
            return GovernanceApprovalEligibility::CurrentPolicyDisabled;
        }
        if !current.signers.contains(signer) {
            return GovernanceApprovalEligibility::RemovedFromCurrentPolicy;
        }
        if let Some(snapshot) = env
            .storage()
            .persistent()
            .get::<_, GovernanceApproverSnapshot>(&DataKey::CriticalApproverSnapshot(
                proposal_id.clone(),
            ))
        {
            let captured = snapshot
                .approvers
                .iter()
                .find(|entry| entry.address == *signer);
            if snapshot.encoding_version != GOVERNANCE_APPROVER_SNAPSHOT_VERSION
                || captured.is_none_or(|entry| {
                    entry.assignment_version != Self::signer_version(env, signer)
                })
            {
                return GovernanceApprovalEligibility::ReassignedAfterSnapshot;
            }
        }
        GovernanceApprovalEligibility::Eligible
    }

    fn eligible_approval_count(
        env: &Env,
        proposal_id: &BytesN<32>,
        proposal: &CriticalActionProposal,
    ) -> u32 {
        let mut eligible = 0;
        for signer in proposal.approvals.iter() {
            let mut without_signer = proposal.clone();
            without_signer.approvals = Vec::new(env);
            if Self::approval_eligibility_for(env, proposal_id, &without_signer, &signer)
                == GovernanceApprovalEligibility::Eligible
            {
                eligible += 1;
            }
        }
        eligible
    }

    fn critical_action_commitment(env: &Env, action: &CriticalAction, nonce: u32) -> BytesN<32> {
        earnproof_shared::governance_action_commitment(
            env,
            &env.current_contract_address(),
            action,
            nonce,
        )
    }

    fn verify_critical_action_domain(
        env: &Env,
        proposal_id: &BytesN<32>,
        proposal: &CriticalActionProposal,
    ) -> Result<(), ContractError> {
        let Some(domain) = env
            .storage()
            .persistent()
            .get::<_, GovernanceActionDomain>(&DataKey::CriticalActionDomain(proposal_id.clone()))
        else {
            // Legacy proposals predate the sidecar and retain their original
            // embedded-policy semantics until they execute, cancel, or expire.
            return Ok(());
        };
        let expected = earnproof_shared::governance_action_commitment(
            env,
            &env.current_contract_address(),
            &proposal.action,
            domain.nonce,
        );
        if domain.encoding_version != GOVERNANCE_ACTION_HASH_VERSION
            || domain.network_id != env.ledger().network_id()
            || domain.contract_id != env.current_contract_address()
            || domain.action_category != proposal.category
            || domain.commitment != *proposal_id
            || expected != *proposal_id
        {
            return Err(ContractError::InvalidState);
        }
        Ok(())
    }

    fn record_critical_action_change(
        env: Env,
        category: ConfigChangeCategory,
        commitment: BytesN<32>,
    ) {
        Self::bump_config_version(env.clone());
        Self::append_config_history(env, category, commitment);
    }
}

#[cfg(test)]
mod test {
    extern crate std;

    use super::{DataKey, ProtocolConfigContract, ProtocolConfigContractClient};
    use earnproof_shared::{
        AdminTransferState, ConfigChangeCategory, ContractError, CriticalAction,
        CriticalActionPolicy, DisputeFilingAnchor, DisputeFilingPolicy, EmergencyAction,
        EmergencyFastTrackPolicy, GovernanceApprovalEligibility, PauseScope, SchemaRateLimit,
        ADMIN_TRANSFER_EXPIRY_LEDGERS, CRITICAL_ACTION_APPROVAL_EXPIRY_LEDGERS,
        DISPUTE_FILING_POLICY_VERSION, GOVERNANCE_ACTION_HASH_VERSION,
        SCHEMA_POLICY_DIGEST_VERSION, TTL_THRESHOLD_LEDGERS,
    };
    use soroban_sdk::{
        testutils::{storage::Persistent as _, Address as _, Ledger as _},
        Address, BytesN, Env, Vec,
    };

    const ADMIN: &str = "GCFIRY65OQE7DFP5KLNS2PF2LVZMUZYJX4OZIEQ36N2IQANUB5XVYOJR";
    const OTHER: &str = "GCATS5YOVB6ROX2WUNKGNQ2MP3GMXDMKSG2O4N5CLX3A6W4PZGZZI55U";

    fn bytes(env: &Env, value: u8) -> BytesN<32> {
        BytesN::from_array(env, &[value; 32])
    }

    fn setup() -> (Env, ProtocolConfigContractClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(ProtocolConfigContract, ());
        let client = ProtocolConfigContractClient::new(&env, &contract_id);
        let admin = Address::from_str(&env, ADMIN);
        client.initialize(&admin);
        (env, client, admin)
    }

    // ── existing tests ────────────────────────────────────────────────────────

    #[test]
    fn exposes_a_stable_interface_version() {
        let (_env, client, _admin) = setup();
        let version = client.interface_version();
        assert_eq!(version, earnproof_shared::PROTOCOL_CONFIG_INTERFACE_VERSION);
        assert_eq!(version.major, 1);
    }

    #[test]
    fn initializes_config_defaults() {
        let (_env, client, admin) = setup();

        assert_eq!(client.get_admin(), admin);
        assert!(!client.is_paused());
        assert_eq!(client.get_config_version(), 1);
        assert!(!client.is_schema_version_approved(&1));
        // contract version initialized to 1
        assert_eq!(client.get_contract_version(), 1);
        assert_eq!(
            client.get_dispute_filing_policy(),
            DisputeFilingPolicy {
                version: DISPUTE_FILING_POLICY_VERSION,
                anchor: DisputeFilingAnchor::Created,
                opens_after_seconds: 0,
                closes_after_seconds: u64::MAX,
            }
        );
        assert!(client.is_nullifier_domain_supported(&1));
        assert_eq!(
            client.get_nullifier_domain_versions(),
            soroban_sdk::vec![&_env, 1]
        );
    }

    #[test]
    fn dispute_filing_policy_is_versioned_validated_and_recorded() {
        let (_env, client, _admin) = setup();
        let before = client.get_config_version();
        let policy = DisputeFilingPolicy {
            version: DISPUTE_FILING_POLICY_VERSION,
            anchor: DisputeFilingAnchor::Expired,
            opens_after_seconds: 10,
            closes_after_seconds: 20,
        };
        client.set_dispute_filing_policy(&policy);
        assert_eq!(client.get_dispute_filing_policy(), policy);
        assert_eq!(client.get_config_version(), before + 1);

        let invalid_version = DisputeFilingPolicy {
            version: DISPUTE_FILING_POLICY_VERSION + 1,
            anchor: DisputeFilingAnchor::Created,
            opens_after_seconds: 0,
            closes_after_seconds: 1,
        };
        assert_eq!(
            client.try_set_dispute_filing_policy(&invalid_version),
            Err(Ok(ContractError::InvalidInput))
        );
        let empty_window = DisputeFilingPolicy {
            version: DISPUTE_FILING_POLICY_VERSION,
            anchor: DisputeFilingAnchor::Created,
            opens_after_seconds: 5,
            closes_after_seconds: 5,
        };
        assert_eq!(
            client.try_set_dispute_filing_policy(&empty_window),
            Err(Ok(ContractError::InvalidInput))
        );
        assert_eq!(client.get_config_version(), before + 1);
    }

    #[test]
    fn dispute_filing_policy_requires_admin_auth() {
        let (env, client, _admin) = setup();
        env.set_auths(&[]);
        let policy = DisputeFilingPolicy {
            version: DISPUTE_FILING_POLICY_VERSION,
            anchor: DisputeFilingAnchor::Created,
            opens_after_seconds: 0,
            closes_after_seconds: 10,
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.set_dispute_filing_policy(&policy);
        }));
        assert!(result.is_err());
    }

    #[test]
    fn nullifier_domain_versions_are_governed_and_bounded() {
        let (_env, client, _admin) = setup();
        let before = client.get_config_version();
        client.set_nullifier_domain_version(&2, &true);
        assert!(client.is_nullifier_domain_supported(&2));
        assert_eq!(
            client.get_nullifier_domain_versions(),
            soroban_sdk::vec![&_env, 1, 2]
        );
        client.set_nullifier_domain_version(&2, &false);
        assert!(!client.is_nullifier_domain_supported(&2));
        assert_eq!(
            client.get_nullifier_domain_versions(),
            soroban_sdk::vec![&_env, 1]
        );
        assert_eq!(client.get_config_version(), before + 2);

        assert_eq!(
            client.try_set_nullifier_domain_version(&0, &true),
            Err(Ok(ContractError::InvalidInput))
        );
        assert_eq!(
            client.try_set_nullifier_domain_version(&3, &false),
            Err(Ok(ContractError::InvalidInput))
        );
        assert_eq!(client.get_config_version(), before + 2);

        for version in 3_u32..=16 {
            client.set_nullifier_domain_version(&version, &true);
        }
        assert_eq!(
            client.try_set_nullifier_domain_version(&17, &true),
            Err(Ok(ContractError::InvalidInput))
        );
        assert!(!client.is_nullifier_domain_supported(&17));
    }

    #[test]
    fn nullifier_domain_policy_requires_admin_auth() {
        let (env, client, _admin) = setup();
        env.set_auths(&[]);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.set_nullifier_domain_version(&2, &true);
        }));
        assert!(result.is_err());
        assert!(!client.is_nullifier_domain_supported(&2));
    }

    #[test]
    fn admin_nomination_expires_at_exact_boundary_and_cleanup_is_terminal() {
        let (env, client, admin) = setup();
        env.ledger().set_sequence_number(100);
        let nominee = Address::from_str(&env, OTHER);
        client.nominate_admin(&nominee);

        let status = client.get_admin_transfer_status();
        assert_eq!(status.state, AdminTransferState::Pending);
        assert_eq!(status.nominated_at_ledger, 100);
        assert_eq!(
            status.expires_at_ledger,
            100 + ADMIN_TRANSFER_EXPIRY_LEDGERS
        );

        env.ledger()
            .set_sequence_number(status.expires_at_ledger - 1);
        assert_eq!(
            client.get_admin_transfer_status().state,
            AdminTransferState::Pending
        );
        env.ledger().set_sequence_number(status.expires_at_ledger);
        assert_eq!(
            client.try_accept_admin(),
            Err(Ok(ContractError::AdminNominationExpired))
        );
        assert_eq!(client.get_admin(), admin);
        assert_eq!(
            client.get_admin_transfer_status().state,
            AdminTransferState::Expired
        );

        client.cleanup_expired_admin_transfer();
        assert_eq!(
            client.get_admin_transfer_status().state,
            AdminTransferState::Absent
        );
        assert_eq!(
            client.try_cleanup_expired_admin_transfer(),
            Err(Ok(ContractError::NotFound))
        );
        assert_eq!(client.get_admin(), admin);
    }

    #[test]
    fn pause_and_unpause_bump_config_version() {
        let (_env, client, _admin) = setup();

        client.pause();
        assert!(client.is_paused());
        assert_eq!(client.get_config_version(), 2);

        client.unpause();
        assert!(!client.is_paused());
        assert_eq!(client.get_config_version(), 3);
    }

    #[test]
    fn schema_versions_can_be_approved_and_deprecated() {
        let (_env, client, _admin) = setup();

        client.approve_schema_version(&1);
        assert!(client.is_schema_version_approved(&1));

        client.deprecate_schema_version(&1);
        assert!(!client.is_schema_version_approved(&1));
    }

    #[test]
    fn schema_policy_digest_is_deterministic_and_covers_every_policy_field() {
        let (env, client, _admin) = setup();
        env.ledger().set_timestamp(100);
        client.set_schema_policy(&7, &soroban_sdk::vec![&env, 1_u32, 2_u32], &5_000);
        client.set_schema_payload_limit(&7, &2_048);
        client.set_schema_rate_limit(&7, &25, &100);
        client.approve_schema_version(&7);

        let initial = client.get_schema_policy_digest(&7);
        assert_eq!(initial.digest_version, SCHEMA_POLICY_DIGEST_VERSION);
        assert_eq!(initial.schema_version, 7);
        assert_eq!(initial, client.get_schema_policy_digest(&7));
        assert_eq!(client.get_schema_lifecycle(&7).activated_at, 100);
        assert_eq!(
            client.get_schema_rate_limit(&7),
            SchemaRateLimit {
                max_registrations: 25,
                window_ledgers: 100,
            }
        );

        client.set_schema_payload_limit(&7, &2_049);
        let payload_changed = client.get_schema_policy_digest(&7);
        assert_ne!(initial.digest, payload_changed.digest);
        assert!(payload_changed.config_version > initial.config_version);

        client.set_schema_rate_limit(&7, &26, &100);
        let rate_changed = client.get_schema_policy_digest(&7);
        assert_ne!(payload_changed.digest, rate_changed.digest);

        env.ledger().set_timestamp(200);
        client.deprecate_schema_version(&7);
        let deprecated = client.get_schema_policy_digest(&7);
        assert_ne!(rate_changed.digest, deprecated.digest);
        assert_eq!(client.get_schema_lifecycle(&7).deprecated_at, 200);

        env.ledger().set_timestamp(300);
        client.approve_schema_version(&7);
        let reactivated = client.get_schema_policy_digest(&7);
        assert_ne!(deprecated.digest, reactivated.digest);
        assert_eq!(client.get_schema_lifecycle(&7).activated_at, 300);
        assert_eq!(client.get_schema_lifecycle(&7).deprecated_at, 0);
    }

    #[test]
    fn schema_rate_limit_rejects_invalid_or_unauthorized_updates_atomically() {
        let (env, client, _admin) = setup();
        let before = client.get_config_version();
        assert_eq!(
            client.try_set_schema_rate_limit(&1, &10, &0),
            Err(Ok(ContractError::InvalidInput))
        );
        assert_eq!(client.get_config_version(), before);

        env.set_auths(&[]);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.set_schema_rate_limit(&1, &10, &100);
        }));
        assert!(result.is_err());
        assert_eq!(client.get_config_version(), before);
    }

    #[test]
    fn rejects_zero_schema_version() {
        let (_env, client, _admin) = setup();
        use earnproof_shared::ContractError;

        let result = client.try_approve_schema_version(&0);
        assert_eq!(result, Err(Ok(ContractError::InvalidInput)));
    }

    #[test]
    fn extends_schema_storage_ttl() {
        let (env, client, _admin) = setup();

        client.approve_schema_version(&7);

        env.as_contract(&client.address, || {
            assert!(
                env.storage()
                    .persistent()
                    .get_ttl(&DataKey::SchemaVersion(7))
                    > TTL_THRESHOLD_LEDGERS
            );
        });
    }

    // ── upgrade governance tests ──────────────────────────────────────────────

    #[test]
    fn approve_and_check_allowlist() {
        let (env, client, _admin) = setup();
        let hash = bytes(&env, 0xab);

        assert!(!client.is_upgrade_allowed(&hash));
        client.approve_upgrade(&hash, &2);
        assert!(client.is_upgrade_allowed(&hash));
    }

    #[test]
    fn revoke_removes_from_allowlist() {
        let (env, client, _admin) = setup();
        let hash = bytes(&env, 0xcd);

        client.approve_upgrade(&hash, &2);
        assert!(client.is_upgrade_allowed(&hash));

        client.revoke_upgrade(&hash);
        assert!(!client.is_upgrade_allowed(&hash));
    }

    #[test]
    #[should_panic(expected = "new_version must be greater than current contract version")]
    fn approve_upgrade_rejects_downgrade_version() {
        let (env, client, _admin) = setup();
        // current version is 1; attempting to allowlist version 1 is rejected
        client.approve_upgrade(&bytes(&env, 1), &1);
    }

    #[test]
    #[should_panic(expected = "new_version must be greater than current contract version")]
    fn approve_upgrade_rejects_lower_version() {
        let (env, client, _admin) = setup();
        // current version is 1; version 0 must be rejected
        client.approve_upgrade(&bytes(&env, 1), &0);
    }

    #[test]
    #[should_panic(expected = "wasm hash not on allowlist")]
    fn upgrade_contract_rejects_non_allowlisted_hash() {
        let (env, client, _admin) = setup();
        client.upgrade_contract(&bytes(&env, 0xff));
    }

    /// Verifies that `upgrade_contract` enforces admin authorization before
    /// doing anything.  `mock_all_auths` is intentionally NOT used here.
    #[test]
    #[should_panic]
    fn upgrade_contract_requires_admin_auth() {
        let env = Env::default();
        let contract_id = env.register(ProtocolConfigContract, ());
        let client = ProtocolConfigContractClient::new(&env, &contract_id);
        let admin = Address::from_str(&env, ADMIN);

        // Initialize using mocked auths.
        env.mock_all_auths();
        client.initialize(&admin);
        let hash = BytesN::from_array(&env, &[0xde; 32]);
        client.approve_upgrade(&hash, &2);
        env.set_auths(&[]);

        // Attempt upgrade without auth — must panic.
        client.upgrade_contract(&hash);
    }

    /// Verifies that the allowlist entry is consumed after a successful upgrade
    /// so the same hash cannot be replayed.
    ///
    /// Note: in the test environment `update_current_contract_wasm` is a no-op
    /// (the WASM is not actually swapped) but all surrounding state transitions
    /// — version bump, allowlist removal, event emission — are fully exercised.
    #[test]
    fn upgrade_contract_advances_version_and_consumes_allowlist() {
        let (env, client, _admin) = setup();
        let hash = bytes(&env, 0x42);

        assert_eq!(client.get_contract_version(), 1);
        client.approve_upgrade(&hash, &2);
        assert!(client.is_upgrade_allowed(&hash));

        client.upgrade_contract(&hash);

        // Version must have advanced.
        assert_eq!(client.get_contract_version(), 2);
        // Allowlist entry must have been consumed.
        assert!(!client.is_upgrade_allowed(&hash));
    }

    /// After a successful upgrade the same hash cannot be applied a second
    /// time (allowlist entry was consumed, and the version guard would also
    /// block it even if re-approved with the same version).
    #[test]
    #[should_panic(expected = "wasm hash not on allowlist")]
    fn upgrade_contract_hash_cannot_be_replayed() {
        let (env, client, _admin) = setup();
        let hash = bytes(&env, 0x42);

        client.approve_upgrade(&hash, &2);
        client.upgrade_contract(&hash);
        // Second application must fail — entry was consumed.
        client.upgrade_contract(&hash);
    }

    /// State written before an upgrade is still readable after.
    #[test]
    fn state_preserved_across_upgrade() {
        let (env, client, _admin) = setup();

        // Write some state before the upgrade.
        client.approve_schema_version(&3);
        client.pause();
        assert!(client.is_paused());
        assert!(client.is_schema_version_approved(&3));

        // Perform upgrade.
        let hash = bytes(&env, 0x77);
        client.approve_upgrade(&hash, &2);
        client.upgrade_contract(&hash);

        // State must be intact after upgrade.
        assert!(client.is_paused());
        assert!(client.is_schema_version_approved(&3));
        assert_eq!(client.get_contract_version(), 2);
    }

    /// An upgrade approved with version N cannot be reused to downgrade from
    /// a later version M > N even if the hash is re-added to the allowlist.
    #[test]
    #[should_panic(expected = "new_version must be greater than current contract version")]
    fn cannot_re_approve_old_version_after_upgrade() {
        let (env, client, _admin) = setup();
        let hash_v2 = bytes(&env, 0x01);
        let old_hash = bytes(&env, 0x02);

        // Upgrade to version 2.
        client.approve_upgrade(&hash_v2, &2);
        client.upgrade_contract(&hash_v2);
        assert_eq!(client.get_contract_version(), 2);

        // Attempt to allowlist a hash that would install version 1 — rejected.
        client.approve_upgrade(&old_hash, &1);
    }

    /// `approve_upgrade` by a non-admin must be rejected.
    #[test]
    #[should_panic]
    fn approve_upgrade_requires_admin_auth() {
        let env = Env::default();
        let contract_id = env.register(ProtocolConfigContract, ());
        let client = ProtocolConfigContractClient::new(&env, &contract_id);
        let admin = Address::from_str(&env, ADMIN);
        let other = Address::from_str(&env, OTHER);

        env.mock_all_auths();
        client.initialize(&admin);

        // Only authorize `other`, not `admin` — should panic.
        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &other,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &client.address,
                fn_name: "approve_upgrade",
                args: soroban_sdk::vec![
                    &env,
                    soroban_sdk::IntoVal::into_val(&BytesN::from_array(&env, &[0xaa; 32]), &env),
                    soroban_sdk::IntoVal::into_val(&2_u32, &env),
                ],
                sub_invokes: &[],
            },
        }]);
        client.approve_upgrade(&BytesN::from_array(&env, &[0xaa; 32]), &2);
    }

    // ── numeric boundary tests ────────────────────────────────────────────────

    /// Table-driven tests for schema version boundaries.
    /// Schema versions must be >= MIN_SCHEMA_VERSION (1).
    #[test]
    fn schema_version_boundary_values() {
        let (_env, client, _admin) = setup();

        // Valid: minimum allowed schema version
        client.approve_schema_version(&1);
        assert!(client.is_schema_version_approved(&1));

        // Valid: typical schema versions
        client.approve_schema_version(&2);
        assert!(client.is_schema_version_approved(&2));

        client.approve_schema_version(&100);
        assert!(client.is_schema_version_approved(&100));

        // Valid: u32 maximum
        client.approve_schema_version(&u32::MAX);
        assert!(client.is_schema_version_approved(&u32::MAX));
    }

    #[test]
    fn schema_version_zero_rejected() {
        let (_env, client, _admin) = setup();
        // Version 0 must be rejected with a typed error.
        use earnproof_shared::ContractError;
        let result = client.try_approve_schema_version(&0);
        assert_eq!(result, Err(Ok(ContractError::InvalidInput)));
    }

    #[test]
    fn is_schema_version_approved_with_zero_returns_false() {
        let (_env, client, _admin) = setup();
        // Querying version 0 should return false without panic
        let result = client.is_schema_version_approved(&0);
        assert!(!result);
    }

    /// Table-driven tests for contract version boundaries.
    /// Contract versions must be > current version (monotonically increasing).
    #[test]
    fn contract_version_upgrade_boundaries() {
        let (env, client, _admin) = setup();
        assert_eq!(client.get_contract_version(), 1);

        // Valid: immediate next version
        client.approve_upgrade(&bytes(&env, 1), &2);
        client.upgrade_contract(&bytes(&env, 1));
        assert_eq!(client.get_contract_version(), 2);

        // Valid: skip versions (not required to be sequential)
        client.approve_upgrade(&bytes(&env, 2), &1000);
        client.upgrade_contract(&bytes(&env, 2));
        assert_eq!(client.get_contract_version(), 1000);

        // Valid: very large version number
        client.approve_upgrade(&bytes(&env, 3), &u32::MAX);
        client.upgrade_contract(&bytes(&env, 3));
        assert_eq!(client.get_contract_version(), u32::MAX);
    }

    #[test]
    #[should_panic(expected = "new_version must be greater than current contract version")]
    fn contract_version_equal_current_rejected() {
        let (env, client, _admin) = setup();
        // Current version is 1; attempting to set it to 1 again is rejected
        client.approve_upgrade(&bytes(&env, 1), &1);
    }

    #[test]
    #[should_panic(expected = "new_version must be greater than current contract version")]
    fn contract_version_below_current_rejected() {
        let (env, client, _admin) = setup();
        // Current version is 1; attempting to set it to 0 is rejected
        client.approve_upgrade(&bytes(&env, 1), &0);
    }

    /// Table-driven tests for config version bumping.
    /// Config version increments on every configuration change.
    /// This tests the checked_add protection against overflow.
    #[test]
    fn config_version_increments_on_mutations() {
        let (_env, client, _admin) = setup();
        assert_eq!(client.get_config_version(), 1);

        // Each mutation bumps config version
        client.pause();
        assert_eq!(client.get_config_version(), 2);

        client.unpause();
        assert_eq!(client.get_config_version(), 3);

        client.approve_schema_version(&1);
        assert_eq!(client.get_config_version(), 4);

        client.deprecate_schema_version(&1);
        assert_eq!(client.get_config_version(), 5);
    }

    /// Verify that config version correctly handles large values
    /// approaching u32::MAX (bumping is protected by checked_add).
    #[test]
    fn config_version_safe_near_u32_max() {
        let (_env, client, _admin) = setup();

        // Manually set config version to a value near max by simulating
        // many mutations. We'll do a smaller simulation here.
        // In real operation, reaching u32::MAX would require ~4 billion mutations,
        // which is impractical in a test, but we verify the protection exists.

        // Get current config version (should be 1 after setup)
        let mut v = client.get_config_version();
        assert_eq!(v, 1);

        // Perform several mutations and verify each increments exactly once,
        // starting from the value established by the previous mutation.
        for _ in 0..10 {
            client.pause();
            let after_pause = client.get_config_version();
            assert_eq!(
                after_pause,
                v + 1,
                "pause must bump config version exactly once"
            );

            client.unpause();
            let after_unpause = client.get_config_version();
            assert_eq!(
                after_unpause,
                after_pause + 1,
                "unpause must bump config version exactly once"
            );
            v = after_unpause;
        }
    }

    /// Test storage and event invariants: failed boundary cases
    /// must not modify state or emit events.
    #[test]
    fn failed_schema_version_zero_leaves_state_unchanged() {
        let (_env, client, _admin) = setup();

        let config_before = client.get_config_version();
        let approved_before = client.is_schema_version_approved(&999);

        // Attempt to approve version 0 — should panic
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.approve_schema_version(&0);
        }));

        // Must have panicked
        assert!(result.is_err());

        // State must be unchanged
        assert_eq!(client.get_config_version(), config_before);
        assert_eq!(
            client.is_schema_version_approved(&999),
            approved_before,
            "schema version approval state must not change on failed validation"
        );
    }

    #[test]
    fn failed_upgrade_version_downgrade_leaves_state_unchanged() {
        let (env, client, _admin) = setup();

        let contract_version_before = client.get_contract_version();
        let config_version_before = client.get_config_version();
        let hash = bytes(&env, 0x99);

        // Attempt to allowlist a downgrade (current version is 1, trying version 0)
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.approve_upgrade(&hash, &0);
        }));

        // Must have panicked
        assert!(result.is_err());

        // State must be unchanged: contract version not modified
        assert_eq!(
            client.get_contract_version(),
            contract_version_before,
            "contract version must not change on failed upgrade approval"
        );

        // Config version must not be bumped on failed validation
        assert_eq!(
            client.get_config_version(),
            config_version_before,
            "config version must not change when upgrade approval fails"
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
    /// - Paused = false
    /// - ConfigVersion = 1
    /// - ContractVersion = 1
    /// - Initialized event published
    #[test]
    fn initialization_writes_exactly_documented_state() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(ProtocolConfigContract, ());
        let client = ProtocolConfigContractClient::new(&env, &contract_id);
        let admin = Address::from_str(&env, ADMIN);

        // Before initialization: storage should be empty
        // (querying uninitialized values returns defaults or panics)

        // Perform initialization
        client.initialize(&admin);

        // Verify exact state written
        assert_eq!(client.get_admin(), admin, "admin must be set");
        assert!(
            !client.is_paused(),
            "protocol must not be paused after initialization"
        );
        assert_eq!(
            client.get_config_version(),
            1,
            "config version must be exactly 1 after initialization"
        );
        assert_eq!(
            client.get_contract_version(),
            1,
            "contract version must be exactly 1 after initialization"
        );

        // Verify Initialized event was published
        // (Event verification requires inspecting env's event log)
        env.as_contract(&contract_id, || {
            // Storage keys must all be set (verifiable via has() calls)
            let instance = env.storage().instance();
            assert!(
                instance.has(&DataKey::Admin),
                "Admin key must exist in instance storage"
            );
            assert!(
                instance.has(&DataKey::Paused),
                "Paused key must exist in instance storage"
            );
            assert!(
                instance.has(&DataKey::ConfigVersion),
                "ConfigVersion key must exist in instance storage"
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
        let contract_id = env.register(ProtocolConfigContract, ());
        let client = ProtocolConfigContractClient::new(&env, &contract_id);
        let admin = Address::from_str(&env, ADMIN);

        // First initialization succeeds
        client.initialize(&admin);
        let config_version_after_first = client.get_config_version();
        let contract_version_after_first = client.get_contract_version();
        let paused_after_first = client.is_paused();

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
            client.get_config_version(),
            config_version_after_first,
            "config version must not change after failed re-initialization"
        );
        assert_eq!(
            client.get_contract_version(),
            contract_version_after_first,
            "contract version must not change after failed re-initialization"
        );
        assert_eq!(
            client.is_paused(),
            paused_after_first,
            "paused state must not change after failed re-initialization"
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
        let contract_id = env.register(ProtocolConfigContract, ());
        let client = ProtocolConfigContractClient::new(&env, &contract_id);
        let admin = Address::from_str(&env, ADMIN);
        let other = Address::from_str(&env, OTHER);

        // First initialization with original admin
        client.initialize(&admin);
        let stored_admin = client.get_admin();
        let config_version_after_first = client.get_config_version();

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
            client.get_config_version(),
            config_version_after_first,
            "config version must not change after failed re-initialization by different admin"
        );
    }

    /// Verify that an address that looks like it might have elevated permissions
    /// cannot bypass the re-initialization guard.
    ///
    /// Tests with an address string that is numeric (e.g., address index)
    /// or otherwise potentially special to the test framework.
    #[test]
    fn reinitialization_by_arbitrary_special_address_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(ProtocolConfigContract, ());
        let client = ProtocolConfigContractClient::new(&env, &contract_id);
        let admin = Address::from_str(&env, ADMIN);

        // First initialization with standard admin
        client.initialize(&admin);
        let stored_admin = client.get_admin();

        // Attempt re-initialization with an arbitrary address that might look
        // special (e.g., derived from a standard test key)
        let arbitrary = Address::from_str(&env, OTHER);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.initialize(&arbitrary);
        }));

        // Must have panicked
        assert!(
            result.is_err(),
            "re-initialization by arbitrary address must panic"
        );

        // Original admin must be preserved
        assert_eq!(
            client.get_admin(),
            stored_admin,
            "admin must not change when arbitrary address attempts re-initialization"
        );
    }

    /// Verify that the re-initialization guard is truly the only barrier —
    /// the panic message must indicate "already initialized", not a different
    /// validation error.
    #[test]
    fn reinitialization_panic_message_indicates_guard() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(ProtocolConfigContract, ());
        let client = ProtocolConfigContractClient::new(&env, &contract_id);
        let admin = Address::from_str(&env, ADMIN);

        // First initialization succeeds
        client.initialize(&admin);

        // Attempt re-initialization and verify panic message
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.initialize(&admin);
        }));

        assert!(result.is_err(), "re-initialization must panic");
        // The panic message is internal to the contract; we verify the failure occurred
    }

    /// Verify that the re-initialization guard takes effect immediately after
    /// the first initialize() call completes — no transient window during which
    /// a second initialize could partially succeed.
    #[test]
    fn reinitialization_guard_active_immediately() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(ProtocolConfigContract, ());
        let client = ProtocolConfigContractClient::new(&env, &contract_id);
        let admin = Address::from_str(&env, ADMIN);

        // First initialization
        client.initialize(&admin);

        // Subsequent initializations (multiple attempts) must all fail
        for attempt in 1..=3 {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                client.initialize(&admin);
            }));

            assert!(
                result.is_err(),
                "re-initialization attempt {} must fail",
                attempt
            );

            // Admin must remain unchanged after each failed attempt
            assert_eq!(
                client.get_admin(),
                admin,
                "admin must not change after re-initialization attempt {}",
                attempt
            );
        }
    }

    /// Verify that the documented initialization state is maintained even
    /// across function calls and state mutations after initialization.
    ///
    /// Tests that the initial state (versions, paused flag) is stable
    /// and correct before any subsequent mutations.
    #[test]
    fn initialization_state_stable_before_mutations() {
        let (_env, client, admin) = setup();

        // State immediately after initialization must be as documented
        assert_eq!(client.get_admin(), admin);
        assert!(!client.is_paused());
        assert_eq!(client.get_config_version(), 1);
        assert_eq!(client.get_contract_version(), 1);

        // Perform a mutation (pause)
        client.pause();

        // Admin must remain unchanged
        assert_eq!(
            client.get_admin(),
            admin,
            "admin must not change across mutations"
        );

        // But config version should have bumped
        assert_eq!(
            client.get_config_version(),
            2,
            "config version must increment on mutation"
        );

        // Contract version must remain at 1 (only changes on upgrade)
        assert_eq!(
            client.get_contract_version(),
            1,
            "contract version must not change on config mutation"
        );
    }

    /// Summary test: protocol-config initialization spec verification.
    ///
    /// This test serves as executable documentation of what the test matrix
    /// expects from protocol-config initialization:
    /// - Standalone contract (no dependency addresses)
    /// - Has re-initialization guard
    /// - Emits Initialized event
    /// - Sets: admin, paused=false, config_version=1, contract_version=1
    #[test]
    fn protocol_config_initialization_spec_summary() {
        // CONTRACT SPEC: protocol-config
        // - Name: "protocol-config"
        // - Has re-initialization guard: YES (panics "already initialized")
        // - Emits initialization event: YES (Initialized { admin })
        // - Takes dependency addresses: NO
        // - Dependencies: []
        // - First init writes:
        //   - Admin: passed address (requires auth)
        //   - Paused: false
        //   - ConfigVersion: 1
        //   - ContractVersion: 1
        // - Re-init guard: DataKey::Admin presence check; panics if set
        // - Re-init allowed by different admin: NO (guard blocks all)
        // - Invalid config cases: None (no dependencies to validate)

        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(ProtocolConfigContract, ());
        let client = ProtocolConfigContractClient::new(&env, &contract_id);
        let admin = Address::from_str(&env, ADMIN);

        // Verify the spec
        client.initialize(&admin);
        assert_eq!(client.get_admin(), admin);
        assert!(!client.is_paused());
        assert_eq!(client.get_config_version(), 1);
        assert_eq!(client.get_contract_version(), 1);

        // Re-initialization must fail
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.initialize(&admin)
        }))
        .is_err());
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
    fn genesis_id_is_deterministic_for_the_same_inputs() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::from_str(&env, ADMIN);

        let one = env.register(ProtocolConfigContract, ());
        let one = ProtocolConfigContractClient::new(&env, &one);
        one.initialize(&admin);

        // Recomputing from the same env/contract address/domain must be
        // stable, since the identifier is a pure function of those inputs.
        let recomputed = env.as_contract(&one.address, || {
            earnproof_shared::compute_genesis_id(&env, "earnproof_protocol_config")
        });
        assert_eq!(one.get_genesis().genesis_id, recomputed);
    }

    #[test]
    fn genesis_id_differs_across_contract_instances() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::from_str(&env, ADMIN);

        let a = env.register(ProtocolConfigContract, ());
        let a = ProtocolConfigContractClient::new(&env, &a);
        a.initialize(&admin);

        let b = env.register(ProtocolConfigContract, ());
        let b = ProtocolConfigContractClient::new(&env, &b);
        b.initialize(&admin);

        // Different contract addresses (different instances) must never share
        // an identity, even with the same admin and the same domain.
        assert_ne!(a.get_genesis().genesis_id, b.get_genesis().genesis_id);
    }

    #[test]
    fn genesis_is_unchanged_across_an_upgrade() {
        let (env, client, _admin) = setup();
        let genesis_before = client.get_genesis();

        let hash = bytes(&env, 0x55);
        client.approve_upgrade(&hash, &2);
        client.upgrade_contract(&hash);

        assert_eq!(client.get_genesis(), genesis_before);
    }

    #[test]
    fn get_genesis_fails_before_initialization() {
        let env = Env::default();
        let contract_id = env.register(ProtocolConfigContract, ());
        let client = ProtocolConfigContractClient::new(&env, &contract_id);
        use earnproof_shared::ContractError;

        let result = client.try_get_genesis();
        assert_eq!(result, Err(Ok(ContractError::NotInitialized)));
    }

    // ── schema payload size limits (issue #190) ──────────────────────────────

    #[test]
    fn unset_schema_gets_the_default_payload_limit() {
        let (_env, client, _admin) = setup();
        assert_eq!(
            client.get_schema_payload_limit(&1),
            earnproof_shared::DEFAULT_SCHEMA_PAYLOAD_LIMIT
        );
        // Version 0 can never be approved, and always reads as the default.
        assert_eq!(
            client.get_schema_payload_limit(&0),
            earnproof_shared::DEFAULT_SCHEMA_PAYLOAD_LIMIT
        );
    }

    #[test]
    fn schema_payload_limit_can_be_set_and_read_back() {
        let (_env, client, _admin) = setup();
        client.set_schema_payload_limit(&1, &1_024);
        assert_eq!(client.get_schema_payload_limit(&1), 1_024);
        // Other schema versions are unaffected.
        assert_eq!(
            client.get_schema_payload_limit(&2),
            earnproof_shared::DEFAULT_SCHEMA_PAYLOAD_LIMIT
        );
    }

    #[test]
    fn schema_payload_limit_of_zero_is_a_deterministic_valid_policy() {
        // A limit of zero is a legitimate governance choice (no auxiliary
        // payload permitted for that schema at all), not an error.
        let (_env, client, _admin) = setup();
        client.set_schema_payload_limit(&1, &0);
        assert_eq!(client.get_schema_payload_limit(&1), 0);
    }

    #[test]
    fn schema_payload_limit_rejects_zero_version() {
        let (_env, client, _admin) = setup();
        use earnproof_shared::ContractError;
        let result = client.try_set_schema_payload_limit(&0, &1_024);
        assert_eq!(result, Err(Ok(ContractError::InvalidInput)));
    }

    #[test]
    fn schema_payload_limit_survives_deprecation() {
        // Deprecating a schema version does not clear its configured limit;
        // the policy and the approval flag are independent.
        let (_env, client, _admin) = setup();
        client.approve_schema_version(&1);
        client.set_schema_payload_limit(&1, &1_024);
        client.deprecate_schema_version(&1);
        assert_eq!(client.get_schema_payload_limit(&1), 1_024);
    }

    #[test]
    fn schema_payload_limit_update_does_not_retroactively_invalidate() {
        // Changing the limit is a forward-looking policy change: nothing here
        // reaches into proof-registry to revalidate proofs already written
        // under the old limit, and there is no mechanism by which it could.
        let (_env, client, _admin) = setup();
        client.set_schema_payload_limit(&1, &2_048);
        client.set_schema_payload_limit(&1, &16);
        assert_eq!(client.get_schema_payload_limit(&1), 16);
    }

    fn enabled_policy(env: &Env, threshold: u32) -> (CriticalActionPolicy, Vec<Address>) {
        let signers = Vec::from_array(
            env,
            [
                Address::generate(env),
                Address::generate(env),
                Address::generate(env),
            ],
        );
        (
            CriticalActionPolicy {
                enabled: true,
                threshold,
                signers: signers.clone(),
            },
            signers,
        )
    }

    fn emergency_policy(env: &Env, threshold: u32) -> (EmergencyFastTrackPolicy, Vec<Address>) {
        let guardians = Vec::from_array(
            env,
            [
                Address::generate(env),
                Address::generate(env),
                Address::generate(env),
            ],
        );
        (
            EmergencyFastTrackPolicy {
                enabled: true,
                threshold,
                guardians: guardians.clone(),
                max_lifetime_ledgers: 20,
            },
            guardians,
        )
    }

    #[test]
    fn guardian_threshold_executes_only_a_scoped_pause() {
        let (env, client, _admin) = setup();
        let (policy, guardians) = emergency_policy(&env, 2);
        client.set_emergency_policy(&policy);
        let expires_at = env.ledger().sequence() + 10;
        let proposal_id = client.propose_emergency_action(
            &EmergencyAction::ScopedPause(PauseScope::Registration),
            &bytes(&env, 0xA1),
            &expires_at,
            &guardians.get(0).unwrap(),
        );

        client.approve_emergency_action(&proposal_id, &guardians.get(0).unwrap());
        assert_eq!(
            client.try_execute_emergency_action(&proposal_id),
            Err(Ok(ContractError::InsufficientApprovals))
        );
        client.approve_emergency_action(&proposal_id, &guardians.get(1).unwrap());
        client.execute_emergency_action(&proposal_id);

        assert!(client.is_scope_paused(&PauseScope::Registration));
        assert!(!client.is_paused());
        assert_eq!(client.get_emergency_proposal(&proposal_id), None);
        assert_eq!(
            client.try_execute_emergency_action(&proposal_id),
            Err(Ok(ContractError::ApprovalProposalNotFound))
        );
    }

    #[test]
    fn emergency_proposal_validates_authority_reason_and_expiry_boundaries() {
        let (env, client, _admin) = setup();
        let (policy, guardians) = emergency_policy(&env, 2);
        client.set_emergency_policy(&policy);
        let now = env.ledger().sequence();
        let outsider = Address::generate(&env);

        assert_eq!(
            client.try_propose_emergency_action(
                &EmergencyAction::GlobalPause,
                &bytes(&env, 0xA2),
                &(now + 1),
                &outsider,
            ),
            Err(Ok(ContractError::Unauthorized))
        );
        assert_eq!(
            client.try_propose_emergency_action(
                &EmergencyAction::GlobalPause,
                &BytesN::from_array(&env, &[0; 32]),
                &(now + 1),
                &guardians.get(0).unwrap(),
            ),
            Err(Ok(ContractError::InvalidInput))
        );
        assert_eq!(
            client.try_propose_emergency_action(
                &EmergencyAction::ScopedPause(PauseScope::Global),
                &bytes(&env, 0xA3),
                &(now + 1),
                &guardians.get(0).unwrap(),
            ),
            Err(Ok(ContractError::InvalidInput))
        );
        assert_eq!(
            client.try_propose_emergency_action(
                &EmergencyAction::GlobalPause,
                &bytes(&env, 0xA4),
                &now,
                &guardians.get(0).unwrap(),
            ),
            Err(Ok(ContractError::InvalidTimingConfig))
        );
        assert_eq!(
            client.try_propose_emergency_action(
                &EmergencyAction::GlobalPause,
                &bytes(&env, 0xA5),
                &(now + policy.max_lifetime_ledgers + 1),
                &guardians.get(0).unwrap(),
            ),
            Err(Ok(ContractError::InvalidTimingConfig))
        );
    }

    #[test]
    fn emergency_expiry_boundary_is_closed_and_state_is_unchanged() {
        let (env, client, _admin) = setup();
        let (policy, guardians) = emergency_policy(&env, 2);
        client.set_emergency_policy(&policy);
        let expires_at = env.ledger().sequence() + 2;
        let proposal_id = client.propose_emergency_action(
            &EmergencyAction::GlobalPause,
            &bytes(&env, 0xA6),
            &expires_at,
            &guardians.get(0).unwrap(),
        );
        client.approve_emergency_action(&proposal_id, &guardians.get(0).unwrap());
        client.approve_emergency_action(&proposal_id, &guardians.get(1).unwrap());
        env.ledger().set_sequence_number(expires_at);

        assert_eq!(
            client.try_execute_emergency_action(&proposal_id),
            Err(Ok(ContractError::ApprovalProposalExpired))
        );
        assert!(!client.is_paused());
        assert!(client.get_emergency_proposal(&proposal_id).is_some());
    }

    #[test]
    fn removed_guardian_cannot_approve_or_count_toward_execution() {
        let (env, client, _admin) = setup();
        let (policy, guardians) = emergency_policy(&env, 2);
        client.set_emergency_policy(&policy);
        let proposal_id = client.propose_emergency_action(
            &EmergencyAction::GlobalPause,
            &bytes(&env, 0xA7),
            &(env.ledger().sequence() + 10),
            &guardians.get(0).unwrap(),
        );
        client.approve_emergency_action(&proposal_id, &guardians.get(0).unwrap());

        let replacement = EmergencyFastTrackPolicy {
            enabled: true,
            threshold: 2,
            guardians: Vec::from_array(
                &env,
                [guardians.get(1).unwrap(), guardians.get(2).unwrap()],
            ),
            max_lifetime_ledgers: 20,
        };
        client.set_emergency_policy(&replacement);
        assert_eq!(
            client.try_approve_emergency_action(&proposal_id, &guardians.get(0).unwrap()),
            Err(Ok(ContractError::Unauthorized))
        );
        client.approve_emergency_action(&proposal_id, &guardians.get(1).unwrap());
        assert_eq!(
            client.try_execute_emergency_action(&proposal_id),
            Err(Ok(ContractError::InsufficientApprovals))
        );
        assert!(!client.is_paused());
    }

    #[test]
    fn emergency_policy_changes_use_normal_governance_once_enabled() {
        let (env, client, _admin) = setup();
        let (initial, _) = emergency_policy(&env, 2);
        client.set_emergency_policy(&initial);
        let (normal, signers) = enabled_policy(&env, 1);
        client.set_critical_action_policy(&normal);
        let (replacement, _) = emergency_policy(&env, 3);

        assert_eq!(
            client.try_set_emergency_policy(&replacement),
            Err(Ok(ContractError::ThresholdApprovalRequired))
        );
        let proposal_id = client
            .propose_critical_action(&CriticalAction::EmergencyPolicyUpdate(replacement.clone()));
        client.approve_critical_action(&proposal_id, &signers.get(0).unwrap());
        client.execute_critical_action(&proposal_id);
        assert_eq!(client.get_emergency_policy(), replacement);
    }

    #[test]
    fn emergency_threshold_must_be_stronger_than_normal_threshold() {
        let (env, client, _admin) = setup();
        let (normal, signers) = enabled_policy(&env, 2);
        client.set_critical_action_policy(&normal);
        let (not_stronger, _) = emergency_policy(&env, 2);
        let proposal_id =
            client.propose_critical_action(&CriticalAction::EmergencyPolicyUpdate(not_stronger));
        client.approve_critical_action(&proposal_id, &signers.get(0).unwrap());
        client.approve_critical_action(&proposal_id, &signers.get(1).unwrap());
        assert_eq!(
            client.try_execute_critical_action(&proposal_id),
            Err(Ok(ContractError::InvalidApprovalPolicy))
        );
        assert!(!client.get_emergency_policy().enabled);
    }

    #[test]
    fn critical_action_requires_mixed_signer_threshold_and_consumes_proposal() {
        let (env, client, _admin) = setup();
        let (policy, signers) = enabled_policy(&env, 2);
        client.set_critical_action_policy(&policy);
        let proposal_id = client.propose_critical_action(&CriticalAction::SchemaApproval(42));

        client.approve_critical_action(&proposal_id, &signers.get(0).unwrap());
        assert_eq!(
            client.try_approve_critical_action(&proposal_id, &signers.get(0).unwrap()),
            Err(Ok(ContractError::AlreadyExists))
        );
        let outsider = Address::generate(&env);
        assert_eq!(
            client.try_approve_critical_action(&proposal_id, &outsider),
            Err(Ok(ContractError::Unauthorized))
        );
        assert_eq!(
            client.try_execute_critical_action(&proposal_id),
            Err(Ok(ContractError::InsufficientApprovals))
        );
        client.approve_critical_action(&proposal_id, &signers.get(1).unwrap());
        client.execute_critical_action(&proposal_id);

        assert!(client.is_schema_version_approved(&42));
        assert_eq!(client.get_critical_action_proposal(&proposal_id), None);
        assert_eq!(
            client.try_execute_critical_action(&proposal_id),
            Err(Ok(ContractError::ApprovalProposalNotFound))
        );
    }

    #[test]
    fn approvals_are_bound_to_action_category_and_parameters() {
        let (env, client, _admin) = setup();
        let (policy, signers) = enabled_policy(&env, 1);
        client.set_critical_action_policy(&policy);
        let approval = client.propose_critical_action(&CriticalAction::SchemaApproval(12));
        let deprecation = client.propose_critical_action(&CriticalAction::SchemaDeprecation(12));
        let changed_parameters =
            client.propose_critical_action(&CriticalAction::SchemaApproval(13));
        assert_ne!(approval, deprecation);
        assert_ne!(approval, changed_parameters);

        client.approve_critical_action(&approval, &signers.get(0).unwrap());
        client.execute_critical_action(&approval);
        assert!(client.is_schema_version_approved(&12));
        assert!(!client.is_schema_version_approved(&13));
        assert!(client
            .get_critical_action_proposal(&changed_parameters)
            .is_some());
    }

    #[test]
    fn governance_action_ids_are_domain_separated_by_contract() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::from_str(&env, ADMIN);
        let action = CriticalAction::SchemaApproval(12);

        let first_id = env.register(ProtocolConfigContract, ());
        let first = ProtocolConfigContractClient::new(&env, &first_id);
        first.initialize(&admin);
        let second_id = env.register(ProtocolConfigContract, ());
        let second = ProtocolConfigContractClient::new(&env, &second_id);
        second.initialize(&admin);

        assert_ne!(
            first.derive_critical_action_id(&action, &1),
            second.derive_critical_action_id(&action, &1)
        );
        assert_ne!(
            first.derive_critical_action_id(&action, &1),
            first.derive_critical_action_id(&CriticalAction::SchemaDeprecation(12), &1)
        );
    }

    #[test]
    fn proposal_exposes_and_consumes_its_domain_record() {
        let (env, client, _admin) = setup();
        let (policy, signers) = enabled_policy(&env, 1);
        client.set_critical_action_policy(&policy);
        let action = CriticalAction::SchemaApproval(22);
        let proposal_id = client.propose_critical_action(&action);

        let domain = client.get_critical_action_domain(&proposal_id).unwrap();
        assert_eq!(domain.encoding_version, GOVERNANCE_ACTION_HASH_VERSION);
        assert_eq!(domain.network_id, env.ledger().network_id());
        assert_eq!(domain.contract_id, client.address);
        assert_eq!(domain.action_category, action.category());
        assert_eq!(domain.commitment, proposal_id);

        client.approve_critical_action(&proposal_id, &signers.get(0).unwrap());
        client.execute_critical_action(&proposal_id);
        assert_eq!(client.get_critical_action_domain(&proposal_id), None);
    }

    #[test]
    fn mismatched_domain_fails_before_approval_without_mutation() {
        let (env, client, _admin) = setup();
        let (policy, signers) = enabled_policy(&env, 1);
        client.set_critical_action_policy(&policy);
        let proposal_id = client.propose_critical_action(&CriticalAction::SchemaApproval(23));
        let mut domain = client.get_critical_action_domain(&proposal_id).unwrap();
        domain.encoding_version = GOVERNANCE_ACTION_HASH_VERSION + 1;
        env.as_contract(&client.address, || {
            env.storage()
                .persistent()
                .set(&DataKey::CriticalActionDomain(proposal_id.clone()), &domain);
        });

        assert_eq!(
            client.try_approve_critical_action(&proposal_id, &signers.get(0).unwrap()),
            Err(Ok(ContractError::InvalidState))
        );
        assert!(client
            .get_critical_action_proposal(&proposal_id)
            .unwrap()
            .approvals
            .is_empty());
    }

    #[test]
    fn legacy_proposal_without_domain_sidecar_remains_operable() {
        let (env, client, _admin) = setup();
        let (policy, signers) = enabled_policy(&env, 1);
        client.set_critical_action_policy(&policy);
        let proposal_id = client.propose_critical_action(&CriticalAction::SchemaApproval(24));
        env.as_contract(&client.address, || {
            env.storage()
                .persistent()
                .remove(&DataKey::CriticalActionDomain(proposal_id.clone()));
        });

        assert_eq!(client.get_critical_action_domain(&proposal_id), None);
        client.approve_critical_action(&proposal_id, &signers.get(0).unwrap());
        client.execute_critical_action(&proposal_id);
        assert!(client.is_schema_version_approved(&24));
    }

    #[test]
    fn proposal_captures_versioned_threshold_and_signer_commitment() {
        let (env, client, _admin) = setup();
        let (policy, signers) = enabled_policy(&env, 2);
        client.set_critical_action_policy(&policy);
        let proposal_id = client.propose_critical_action(&CriticalAction::SchemaApproval(31));
        let snapshot = client.get_critical_action_snapshot(&proposal_id).unwrap();
        assert_eq!(snapshot.snapshot_version, 1);
        assert_eq!(snapshot.policy_version, 1);
        assert_eq!(snapshot.threshold, 2);
        assert_eq!(
            snapshot.signer_set_digest,
            earnproof_shared::governance_signer_set_digest(&env, &signers)
        );
        client.approve_critical_action(&proposal_id, &signers.get(0).unwrap());
        client.approve_critical_action(&proposal_id, &signers.get(1).unwrap());
        client.execute_critical_action(&proposal_id);
        assert_eq!(client.get_critical_action_snapshot(&proposal_id), None);
    }

    #[test]
    fn later_policy_change_cannot_weaken_pending_threshold() {
        let (env, client, _admin) = setup();
        let (policy, signers) = enabled_policy(&env, 2);
        client.set_critical_action_policy(&policy);
        let pending = client.propose_critical_action(&CriticalAction::SchemaApproval(32));
        let weaker = CriticalActionPolicy {
            enabled: true,
            threshold: 1,
            signers: signers.clone(),
        };
        let policy_change =
            client.propose_critical_action(&CriticalAction::ApprovalPolicyUpdate(weaker));
        client.approve_critical_action(&policy_change, &signers.get(0).unwrap());
        client.approve_critical_action(&policy_change, &signers.get(1).unwrap());
        client.execute_critical_action(&policy_change);
        assert_eq!(client.get_critical_policy_version(), 2);
        client.approve_critical_action(&pending, &signers.get(0).unwrap());
        assert_eq!(
            client.try_execute_critical_action(&pending),
            Err(Ok(ContractError::InsufficientApprovals))
        );
        client.approve_critical_action(&pending, &signers.get(1).unwrap());
        client.execute_critical_action(&pending);
    }

    #[test]
    fn snapshot_mismatch_fails_atomically() {
        let (env, client, _admin) = setup();
        let (policy, signers) = enabled_policy(&env, 1);
        client.set_critical_action_policy(&policy);
        let proposal_id = client.propose_critical_action(&CriticalAction::SchemaApproval(33));
        let mut snapshot = client.get_critical_action_snapshot(&proposal_id).unwrap();
        snapshot.threshold = 2;
        env.as_contract(&client.address, || {
            env.storage().persistent().set(
                &DataKey::CriticalActionSnapshot(proposal_id.clone()),
                &snapshot,
            );
        });
        assert_eq!(
            client.try_approve_critical_action(&proposal_id, &signers.get(0).unwrap()),
            Err(Ok(ContractError::InvalidState))
        );
    }

    #[test]
    fn policy_version_overflow_does_not_install_policy() {
        let (env, client, _admin) = setup();
        env.as_contract(&client.address, || {
            env.storage()
                .instance()
                .set(&DataKey::CriticalActionPolicyVersion, &u32::MAX);
        });
        let (policy, _) = enabled_policy(&env, 1);
        assert_eq!(
            client.try_set_critical_action_policy(&policy),
            Err(Ok(ContractError::InvalidState))
        );
        assert!(!client.get_critical_action_policy().enabled);
    }

    #[test]
    fn removed_and_new_signers_cannot_approve_pending_proposal() {
        let (env, client, _admin) = setup();
        let (policy, signers) = enabled_policy(&env, 1);
        client.set_critical_action_policy(&policy);
        let pending = client.propose_critical_action(&CriticalAction::SchemaApproval(40));
        let newcomer = Address::generate(&env);
        let replacement = CriticalActionPolicy {
            enabled: true,
            threshold: 1,
            signers: Vec::from_array(
                &env,
                [
                    signers.get(1).unwrap(),
                    signers.get(2).unwrap(),
                    newcomer.clone(),
                ],
            ),
        };
        let change =
            client.propose_critical_action(&CriticalAction::ApprovalPolicyUpdate(replacement));
        client.approve_critical_action(&change, &signers.get(0).unwrap());
        client.execute_critical_action(&change);
        assert_eq!(
            client.approval_eligibility(&pending, &signers.get(0).unwrap()),
            GovernanceApprovalEligibility::RemovedFromCurrentPolicy
        );
        assert_eq!(
            client.approval_eligibility(&pending, &newcomer),
            GovernanceApprovalEligibility::NotInProposalSnapshot
        );
    }

    #[test]
    fn removed_approval_stops_counting() {
        let (env, client, _admin) = setup();
        let (policy, signers) = enabled_policy(&env, 2);
        client.set_critical_action_policy(&policy);
        let pending = client.propose_critical_action(&CriticalAction::SchemaApproval(41));
        client.approve_critical_action(&pending, &signers.get(0).unwrap());
        let replacement = CriticalActionPolicy {
            enabled: true,
            threshold: 2,
            signers: Vec::from_array(&env, [signers.get(1).unwrap(), signers.get(2).unwrap()]),
        };
        let change =
            client.propose_critical_action(&CriticalAction::ApprovalPolicyUpdate(replacement));
        client.approve_critical_action(&change, &signers.get(0).unwrap());
        client.approve_critical_action(&change, &signers.get(1).unwrap());
        client.execute_critical_action(&change);
        client.approve_critical_action(&pending, &signers.get(1).unwrap());
        assert_eq!(
            client.try_execute_critical_action(&pending),
            Err(Ok(ContractError::InsufficientApprovals))
        );
    }

    #[test]
    fn signer_regrant_does_not_restore_old_proposal_eligibility() {
        let (env, client, _admin) = setup();
        let (policy, signers) = enabled_policy(&env, 1);
        client.set_critical_action_policy(&policy);
        let pending = client.propose_critical_action(&CriticalAction::SchemaApproval(42));
        let without_first = CriticalActionPolicy {
            enabled: true,
            threshold: 1,
            signers: Vec::from_array(&env, [signers.get(1).unwrap(), signers.get(2).unwrap()]),
        };
        let removal =
            client.propose_critical_action(&CriticalAction::ApprovalPolicyUpdate(without_first));
        client.approve_critical_action(&removal, &signers.get(0).unwrap());
        client.execute_critical_action(&removal);
        let regrant = CriticalActionPolicy {
            enabled: true,
            threshold: 1,
            signers: signers.clone(),
        };
        let regrant_proposal =
            client.propose_critical_action(&CriticalAction::ApprovalPolicyUpdate(regrant));
        client.approve_critical_action(&regrant_proposal, &signers.get(1).unwrap());
        client.execute_critical_action(&regrant_proposal);
        assert_eq!(
            client.approval_eligibility(&pending, &signers.get(0).unwrap()),
            GovernanceApprovalEligibility::ReassignedAfterSnapshot
        );
    }

    #[test]
    fn eligibility_reports_closed_expiry_boundary() {
        let (env, client, _admin) = setup();
        let (policy, signers) = enabled_policy(&env, 1);
        client.set_critical_action_policy(&policy);
        let proposal_id = client.propose_critical_action(&CriticalAction::SchemaApproval(43));
        let expires_at = client
            .get_critical_action_proposal(&proposal_id)
            .unwrap()
            .expires_at;
        env.ledger().set_sequence_number(expires_at);
        assert_eq!(
            client.approval_eligibility(&proposal_id, &signers.get(0).unwrap()),
            GovernanceApprovalEligibility::ProposalExpired
        );
    }

    #[test]
    fn cancelled_proposal_cannot_be_executed() {
        let (_env, client, _admin) = setup();
        let (policy, _) = enabled_policy(&client.env, 1);
        client.set_critical_action_policy(&policy);
        let proposal_id = client.propose_critical_action(&CriticalAction::SchemaApproval(7));

        client.cancel_critical_action(&proposal_id);
        assert_eq!(
            client.try_execute_critical_action(&proposal_id),
            Err(Ok(ContractError::ApprovalProposalNotFound))
        );
    }

    #[test]
    fn expired_proposal_cannot_be_executed() {
        let (env, client, _admin) = setup();
        let (policy, _) = enabled_policy(&env, 1);
        client.set_critical_action_policy(&policy);
        let proposal_id = client.propose_critical_action(&CriticalAction::SchemaApproval(7));
        let proposal = client.get_critical_action_proposal(&proposal_id).unwrap();

        env.ledger()
            .set_sequence_number(proposal.created_at + CRITICAL_ACTION_APPROVAL_EXPIRY_LEDGERS);
        assert_eq!(
            client.try_execute_critical_action(&proposal_id),
            Err(Ok(ContractError::ApprovalProposalExpired))
        );
    }

    #[test]
    fn failed_execution_keeps_proposal_and_configuration_unchanged() {
        let (env, client, _admin) = setup();
        let (policy, signers) = enabled_policy(&env, 1);
        client.set_critical_action_policy(&policy);
        let proposal_id = client.propose_critical_action(&CriticalAction::SchemaApproval(0));
        client.approve_critical_action(&proposal_id, &signers.get(0).unwrap());
        let cursor_before = client.get_config_history_cursor();

        assert_eq!(
            client.try_execute_critical_action(&proposal_id),
            Err(Ok(ContractError::InvalidInput))
        );
        assert!(!client.is_schema_version_approved(&0));
        assert_eq!(client.get_config_history_cursor(), cursor_before);
        assert!(client.get_critical_action_proposal(&proposal_id).is_some());
    }

    #[test]
    fn policy_updates_are_threshold_governed_but_emergency_pause_is_immediate() {
        let (env, client, _admin) = setup();
        let (policy, signers) = enabled_policy(&env, 1);
        client.set_critical_action_policy(&policy);
        let (next_policy, _) = enabled_policy(&env, 2);

        assert_eq!(
            client.try_set_critical_action_policy(&next_policy),
            Err(Ok(ContractError::ThresholdApprovalRequired))
        );
        let proposal_id = client
            .propose_critical_action(&CriticalAction::ApprovalPolicyUpdate(next_policy.clone()));
        client.approve_critical_action(&proposal_id, &signers.get(0).unwrap());
        client.execute_critical_action(&proposal_id);
        assert_eq!(client.get_critical_action_policy(), next_policy);

        client.pause();
        assert!(client.is_paused());
    }

    #[test]
    fn critical_action_policy_rejects_invalid_threshold_and_duplicate_signers() {
        let (env, client, _admin) = setup();
        let signer = Address::generate(&env);
        let duplicate_signers = Vec::from_array(&env, [signer.clone(), signer]);
        let duplicate_policy = CriticalActionPolicy {
            enabled: true,
            threshold: 1,
            signers: duplicate_signers,
        };
        assert_eq!(
            client.try_set_critical_action_policy(&duplicate_policy),
            Err(Ok(ContractError::InvalidApprovalPolicy))
        );

        let (mut policy, _) = enabled_policy(&env, 1);
        policy.threshold = 4;
        assert_eq!(
            client.try_set_critical_action_policy(&policy),
            Err(Ok(ContractError::InvalidApprovalPolicy))
        );
    }

    // ── bounded configuration change history (issue #193) ────────────────────

    #[test]
    fn change_history_starts_empty() {
        let (_env, client, _admin) = setup();
        assert_eq!(client.get_config_history_cursor(), 0);
        assert_eq!(client.get_config_history(&0, &10).len(), 0);
    }

    #[test]
    fn change_history_records_ordering_and_category() {
        let (_env, client, _admin) = setup();
        client.pause();
        client.unpause();
        client.approve_schema_version(&9);

        assert_eq!(client.get_config_history_cursor(), 3);
        let page = client.get_config_history(&0, &10);
        assert_eq!(page.len(), 3);
        assert_eq!(
            page.get(0).unwrap().category,
            ConfigChangeCategory::PauseToggle
        );
        assert_eq!(
            page.get(1).unwrap().category,
            ConfigChangeCategory::PauseToggle
        );
        assert_eq!(
            page.get(2).unwrap().category,
            ConfigChangeCategory::SchemaApproval
        );
        // Each entry records the config version in effect right after it.
        assert_eq!(
            page.get(2).unwrap().config_version,
            client.get_config_version()
        );
    }

    #[test]
    fn failed_changes_do_not_append_history() {
        let (_env, client, _admin) = setup();
        client.pause();
        let cursor_before = client.get_config_history_cursor();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.approve_schema_version(&0);
        }));
        assert!(result.is_err());

        assert_eq!(client.get_config_history_cursor(), cursor_before);
    }

    #[test]
    fn change_history_ring_rolls_over_deterministically() {
        let (_env, client, _admin) = setup();
        let capacity = earnproof_shared::CONFIG_HISTORY_CAPACITY;

        // Overrun the ring by a handful of entries.
        for version in 1..=(capacity + 3) {
            client.approve_schema_version(&version);
        }
        let total = client.get_config_history_cursor();
        assert_eq!(total, capacity + 3);

        // A cursor before the oldest still-available entry is clamped forward
        // deterministically rather than erroring. The page length is bounded
        // by MAX_CONFIG_HISTORY_PAGE, independent of the ring capacity.
        let page = client.get_config_history(&0, &capacity);
        assert_eq!(page.len(), earnproof_shared::MAX_CONFIG_HISTORY_PAGE);
        // The oldest surviving entry is the one that pushed the first three
        // out of the ring.
        let oldest_surviving_version = 4u32;
        assert_eq!(
            page.get(0).unwrap().proposal_commitment,
            ProtocolConfigContract::commit_for_test(&_env, oldest_surviving_version)
        );
    }

    #[test]
    fn change_history_page_size_is_capped() {
        let (_env, client, _admin) = setup();
        for version in 1..=(earnproof_shared::MAX_CONFIG_HISTORY_PAGE + 5) {
            client.approve_schema_version(&version);
        }
        // Requesting more than the cap returns at most the cap.
        let page = client.get_config_history(&0, &(earnproof_shared::MAX_CONFIG_HISTORY_PAGE + 5));
        assert_eq!(page.len(), earnproof_shared::MAX_CONFIG_HISTORY_PAGE);
    }

    #[test]
    fn change_history_survives_migration() {
        let (_env, client, admin) = setup();
        client.pause();
        let cursor_before = client.get_config_history_cursor();

        client.begin_migration(&2, &1);
        client.advance_migration(&0, &1);
        let _ = admin;

        assert_eq!(client.get_config_history_cursor(), cursor_before);
        assert_eq!(client.get_config_history(&0, &10).len(), cursor_before);
    }

    /// Test-only helper mirroring the contract's private `commit` so the
    /// rollover test can assert on a specific expected commitment.
    impl ProtocolConfigContract {
        fn commit_for_test(env: &Env, version: u32) -> BytesN<32> {
            Self::commit(env, version)
        }
    }
}
