# Generated Contract API

<!-- BEGIN GENERATED: do not edit. -->

## issuer-registry::accept_admin

- Parameters: `env: Env`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::accept_issuer_address_rotation

- Parameters: `env: Env, issuer_id_hash: BytesN<32>,`
- Result: `Result<(), IssuerError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::activate_issuer_signing_key

- Parameters: `env: Env, issuer_id: BytesN<32>`
- Result: `Result<(), IssuerError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::activate_successor

- Parameters: `env: Env, proposal_id: BytesN<32>`
- Result: `Result<(), IssuerError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::advance_migration

- Parameters: `env: Env, expected_cursor: u32, processed_items: u32,`
- Result: `Result<MigrationStatus, ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::approve_upgrade

- Parameters: `env: Env, proposal_id: BytesN<32>, wasm_hash: BytesN<32>, new_version: u32,`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::begin_decommission_grace

- Parameters: `env: Env, proposal_id: BytesN<32>, grace_ledgers: u32,`
- Result: `Result<DecommissionState, ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::begin_layout_migration

- Parameters: `env: Env, target_layout: u32,`
- Result: `Result<StorageLayoutStatus, ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::begin_migration

- Parameters: `env: Env, target_contract_version: u32, total_items: u32,`
- Result: `Result<MigrationStatus, ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::cancel_admin_transfer

- Parameters: `env: Env`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::cancel_issuer_address_rotation

- Parameters: `env: Env, issuer_id_hash: BytesN<32>,`
- Result: `Result<(), IssuerError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::cleanup_expired_admin_transfer

- Parameters: `env: Env`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::complete_layout_migration

- Parameters: `env: Env, target_layout: u32,`
- Result: `Result<StorageLayoutStatus, ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::finalize_decommission

- Parameters: `env: Env, proposal_id: BytesN<32>,`
- Result: `Result<DecommissionState, ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_active_issuer_count

- Parameters: `env: Env`
- Result: `u32`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_active_issuer_signing_key

- Parameters: `env: Env, issuer_id: BytesN<32>,`
- Result: `Option<SigningKeyCommitment>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_address_ttl_status

- Parameters: `env: Env, issuer_address: Address`
- Result: `TtlStatus`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_admin

- Parameters: `env: Env`
- Result: `Result<Address, ContractError>`
- Authorization: none
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_admin_transfer_status

- Parameters: `env: Env`
- Result: `AdminTransferStatus`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_capabilities

- Parameters: `env: Env`
- Result: `ContractCapabilities`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_compromised_key_history

- Parameters: `env: Env, issuer_id: BytesN<32>, cursor: u32, limit: u32,`
- Result: `Result<Vec<CompromisedSigningKeyRecord>, IssuerError>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_compromised_signing_key

- Parameters: `env: Env, issuer_id: BytesN<32>, key_hash: BytesN<32>, algorithm: u32,`
- Result: `Option<CompromisedSigningKeyRecord>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_config_digest

- Parameters: `env: Env`
- Result: `Result<BytesN<32>, ContractError>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_config_digest_version

- Parameters: ``
- Result: `u32`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_contract_version

- Parameters: `env: Env`
- Result: `u32`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_decommission_state

- Parameters: `env: Env`
- Result: `DecommissionState`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_deployment_identity

- Parameters: `env: Env`
- Result: `Option<DeploymentIdentity>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_earliest_reactivation

- Parameters: `env: Env, issuer_id_hash: BytesN<32>`
- Result: `u64`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_genesis

- Parameters: `env: Env`
- Result: `Result<GenesisRecord, ContractError>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_governance_assignment

- Parameters: `env: Env, role: GovernanceRole, address: Address,`
- Result: `Option<GovernanceRoleAssignment>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_governance_assignment_state

- Parameters: `env: Env, role: GovernanceRole, address: Address,`
- Result: `Option<GovernanceAssignmentState>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_instance_ttl_status

- Parameters: `env: Env`
- Result: `TtlStatus`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_issuer

- Parameters: `env: Env, issuer_id_hash: BytesN<32>`
- Result: `Result<IssuerRecord, IssuerError>`
- Authorization: none
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_issuer_assurance_at_sequence

- Parameters: `env: Env, issuer_id: BytesN<32>, sequence: u32,`
- Result: `Option<IssuerAssuranceLevelCommitment>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_issuer_assurance_history

- Parameters: `env: Env, issuer_id: BytesN<32>, offset: u32, limit: u32,`
- Result: `Result<Vec<IssuerAssuranceLevelCommitment>, IssuerError>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_issuer_assurance_level

- Parameters: `env: Env, issuer_id: BytesN<32>,`
- Result: `Option<IssuerAssuranceLevelCommitment>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_issuer_by_address

- Parameters: `env: Env, issuer_address: Address,`
- Result: `Result<IssuerRecord, IssuerError>`
- Authorization: none
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_issuer_epoch

- Parameters: `env: Env`
- Result: `u64`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_issuer_policy_commitments

- Parameters: `env: Env, issuer_id: BytesN<32>,`
- Result: `Option<IssuerPolicyCommitments>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_issuer_revocation_statuses

- Parameters: `env: Env, issuer_id_hashes: Vec<BytesN<32>>,`
- Result: `Result<Vec<IssuerRevocationResult>, IssuerError>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_issuer_revocation_tombstone

- Parameters: `env: Env, issuer_id_hash: BytesN<32>,`
- Result: `IssuerRevocationResult`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_issuer_signing_key_state

- Parameters: `env: Env, issuer_id: BytesN<32>, key_hash: BytesN<32>, algorithm: u32,`
- Result: `Option<SigningKeyLifecycleRecord>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_issuer_statuses

- Parameters: `env: Env, issuer_id_hashes: Vec<BytesN<32>>,`
- Result: `Result<Vec<IssuerStatusResult>, IssuerError>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_issuer_ttl_status

- Parameters: `env: Env, issuer_id_hash: BytesN<32>`
- Result: `TtlStatus`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_latest_upgrade_receipt

- Parameters: `env: Env`
- Result: `Option<UpgradeReceipt>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_max_active_issuers

- Parameters: `env: Env`
- Result: `u32`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_migration_status

- Parameters: `env: Env`
- Result: `Option<MigrationStatus>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_pending_issuer_rotation

- Parameters: `env: Env, issuer_id_hash: BytesN<32>,`
- Result: `Option<PendingIssuerRotation>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_prior_issuer_signing_keys

- Parameters: `env: Env, issuer_id: BytesN<32>,`
- Result: `Vec<SigningKeyCommitment>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_provenance_commitment

- Parameters: `env: Env, issuer_id_hash: BytesN<32>,`
- Result: `Result<BytesN<32>, IssuerError>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_reactivation_cooldown

- Parameters: `env: Env`
- Result: `u64`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_role_activation_delay

- Parameters: `env: Env, role: GovernanceRole`
- Result: `u32`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_rotation_count

- Parameters: `env: Env, issuer_id_hash: BytesN<32>`
- Result: `Result<u32, IssuerError>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_rotation_history

- Parameters: `env: Env, issuer_id_hash: BytesN<32>, offset: u32, limit: u32,`
- Result: `Result<soroban_sdk::Vec<RotationRecord>, IssuerError>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_signing_key_lifecycle

- Parameters: `env: Env, issuer_id: BytesN<32>, cursor: u64, limit: u32,`
- Result: `Result<Vec<SigningKeyLifecycleRecord>, IssuerError>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_signing_key_usage_sequence

- Parameters: `env: Env, issuer_id: BytesN<32>`
- Result: `u64`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_storage_layout

- Parameters: `env: Env`
- Result: `StorageLayoutStatus`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::get_successor

- Parameters: `env: Env`
- Result: `Option<Address>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::governance_role_state

- Parameters: `env: Env, role: GovernanceRole, address: Address,`
- Result: `GovernanceRoleState`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::grant_governance_role

- Parameters: `env: Env, proposal_id: BytesN<32>, role: GovernanceRole, address: Address, activation_ledger: u32, expiration_ledger: Option<u32>,`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::has_capability

- Parameters: `_env: Env, capability: u32`
- Result: `bool`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::initialize

- Parameters: `env: Env, admin: Address`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::interface_version

- Parameters: `_env: Env`
- Result: `InterfaceVersion`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::is_active_address

- Parameters: `env: Env, issuer_address: Address`
- Result: `bool`
- Authorization: none
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::is_active_issuer

- Parameters: `env: Env, issuer_id_hash: BytesN<32>`
- Result: `bool`
- Authorization: none
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::is_decommissioned

- Parameters: `env: Env`
- Result: `bool`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::is_proposal_executed

- Parameters: `env: Env, proposal_id: BytesN<32>`
- Result: `bool`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::is_upgrade_allowed

- Parameters: `env: Env, wasm_hash: BytesN<32>`
- Result: `bool`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::keepalive_address_issuer

- Parameters: `env: Env, issuer_address: Address`
- Result: `bool`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::keepalive_instance

- Parameters: `env: Env`
- Result: `bool`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::keepalive_issuer

- Parameters: `env: Env, issuer_id_hash: BytesN<32>`
- Result: `bool`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::keepalive_revocation_tombstone

- Parameters: `env: Env, issuer_id_hash: BytesN<32>`
- Result: `bool`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::mark_signing_key_compromised

- Parameters: `env: Env, issuer_id: BytesN<32>, key_hash: BytesN<32>, algorithm: u32, reason_hash: BytesN<32>,`
- Result: `Result<CompromisedSigningKeyRecord, IssuerError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::migrate

- Parameters: `env: Env, active_issuer_count: u32`
- Result: `Result<(), IssuerError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::nominate_admin

- Parameters: `env: Env, new_admin: Address`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::nominate_successor

- Parameters: `env: Env, proposal_id: BytesN<32>, successor: Address,`
- Result: `Result<(), IssuerError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::propose_issuer_signing_key

- Parameters: `env: Env, issuer_id: BytesN<32>, key_hash: BytesN<32>, algorithm: u32,`
- Result: `Result<(), IssuerError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::reactivate_issuer

- Parameters: `env: Env, proposal_id: BytesN<32>, issuer_id_hash: BytesN<32>, reason_commitment: BytesN<32>,`
- Result: `Result<(), IssuerError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::refresh_instance_ttl

- Parameters: `env: Env`
- Result: `Result<TtlStatus, ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::register_issuer

- Parameters: `env: Env, issuer_id_hash: BytesN<32>, issuer_address: Address, metadata_hash: BytesN<32>, provenance_commitment: BytesN<32>,`
- Result: `Result<(), IssuerError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::remove_governance_role

- Parameters: `env: Env, role: GovernanceRole, address: Address,`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::retire_issuer_signing_key

- Parameters: `env: Env, issuer_id: BytesN<32>,`
- Result: `Result<SigningKeyLifecycleRecord, IssuerError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::revoke_issuer

- Parameters: `env: Env, proposal_id: BytesN<32>, issuer_id_hash: BytesN<32>, reason_commitment: BytesN<32>,`
- Result: `Result<(), IssuerError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::revoke_upgrade

- Parameters: `env: Env, proposal_id: BytesN<32>, wasm_hash: BytesN<32>,`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::revoke_upgrade_approval

- Parameters: `env: Env, proposal_id: BytesN<32>`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::rotate_issuer_address

- Parameters: `env: Env, issuer_id_hash: BytesN<32>, new_address: Address,`
- Result: `Result<(), IssuerError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::set_deployment_domain

- Parameters: `env: Env, deployment_domain: BytesN<32>,`
- Result: `Result<DeploymentIdentity, ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::set_issuer_assurance_level

- Parameters: `env: Env, issuer_id: BytesN<32>, commitment_version: u32, assurance_level: u32,`
- Result: `Result<IssuerAssuranceLevelCommitment, IssuerError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::set_issuer_metadata_commitment

- Parameters: `env: Env, issuer_id_hash: BytesN<32>, metadata_hash: BytesN<32>, metadata_uri_hash: BytesN<32>,`
- Result: `Result<(), IssuerError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::set_issuer_policy_commitments

- Parameters: `env: Env, issuer_id: BytesN<32>, commitments: IssuerPolicyCommitments,`
- Result: `Result<(), IssuerError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::set_max_active_issuers

- Parameters: `env: Env, new_max: u32, allow_below_usage: bool,`
- Result: `Result<(), IssuerError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::set_reactivation_cooldown

- Parameters: `env: Env, cooldown_seconds: u64`
- Result: `Result<(), IssuerError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::set_role_activation_delay

- Parameters: `env: Env, role: GovernanceRole, delay_ledgers: u32,`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::suspend_issuer

- Parameters: `env: Env, proposal_id: BytesN<32>, issuer_id_hash: BytesN<32>, reason_commitment: BytesN<32>,`
- Result: `Result<(), IssuerError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::suspend_issuer_by_role

- Parameters: `env: Env, proposal_id: BytesN<32>, issuer_id_hash: BytesN<32>, reason_commitment: BytesN<32>, actor: Address,`
- Result: `Result<(), IssuerError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::update_issuer

- Parameters: `env: Env, issuer_id_hash: BytesN<32>, metadata_hash: BytesN<32>,`
- Result: `Result<(), IssuerError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## issuer-registry::upgrade_contract

- Parameters: `env: Env, wasm_hash: BytesN<32>, new_version: u32,`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/issuer-registry/src/lib.rs`

## proof-registry::accept_admin

- Parameters: `env: Env`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::accepted_issuer_registry_version

- Parameters: `_env: Env`
- Result: `InterfaceVersion`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::accepted_protocol_config_version

- Parameters: `_env: Env`
- Result: `InterfaceVersion`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::acknowledge_pause

- Parameters: `env: Env, scope: PauseScope, acknowledgement_hash: BytesN<32>, actor: Address,`
- Result: `Result<PauseAcknowledgementRecord, ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::activate_successor

- Parameters: `env: Env`
- Result: `Result<(), ProofError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::admin_revoke_proof

- Parameters: `env: Env, proof_id_hash: BytesN<32>`
- Result: `Result<(), ProofError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::admin_revoke_proofs_batch

- Parameters: `env: Env, proof_id_hashes: Vec<BytesN<32>>,`
- Result: `Result<(), ProofError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::advance_migration

- Parameters: `env: Env, expected_cursor: u32, processed_items: u32,`
- Result: `Result<MigrationStatus, ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::approve_upgrade

- Parameters: `env: Env, wasm_hash: BytesN<32>, new_version: u32,`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::archive_proof

- Parameters: `env: Env, proof_id_hash: BytesN<32>`
- Result: `Result<(), ProofError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::begin_decommission_grace

- Parameters: `env: Env, grace_ledgers: u32,`
- Result: `Result<DecommissionState, ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::begin_layout_migration

- Parameters: `env: Env, target_layout: u32,`
- Result: `Result<StorageLayoutStatus, ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::begin_migration

- Parameters: `env: Env, target_contract_version: u32, total_items: u32,`
- Result: `Result<MigrationStatus, ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::bound_issuer_registry_version

- Parameters: `env: Env`
- Result: `Result<InterfaceVersion, ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::bound_protocol_config_version

- Parameters: `env: Env`
- Result: `Result<InterfaceVersion, ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::cancel_admin_transfer

- Parameters: `env: Env`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::cleanup_expired_admin_transfer

- Parameters: `env: Env`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::complete_layout_migration

- Parameters: `env: Env, target_layout: u32,`
- Result: `Result<StorageLayoutStatus, ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::finalize_decommission

- Parameters: `env: Env`
- Result: `Result<DecommissionState, ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_admin

- Parameters: `env: Env`
- Result: `Result<Address, ContractError>`
- Authorization: none
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_admin_transfer_status

- Parameters: `env: Env`
- Result: `AdminTransferStatus`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_archived_proof

- Parameters: `env: Env, proof_id_hash: BytesN<32>,`
- Result: `Result<ArchivedProofRecord, ProofError>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_capabilities

- Parameters: `env: Env`
- Result: `ContractCapabilities`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_config_digest

- Parameters: `env: Env`
- Result: `Result<BytesN<32>, ContractError>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_config_digest_version

- Parameters: ``
- Result: `u32`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_contract_version

- Parameters: `env: Env`
- Result: `u32`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_decommission_state

- Parameters: `env: Env`
- Result: `DecommissionState`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_deployment_identity

- Parameters: `env: Env`
- Result: `Option<DeploymentIdentity>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_dispute

- Parameters: `env: Env, proof_id_hash: BytesN<32>`
- Result: `Result<DisputeRecord, ProofError>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_dispute_filing_eligibility

- Parameters: `env: Env, proof_id_hash: BytesN<32>,`
- Result: `Result<DisputeFilingEligibility, ProofError>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_genesis

- Parameters: `env: Env`
- Result: `Result<GenesisRecord, ContractError>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_instance_ttl_status

- Parameters: `env: Env`
- Result: `TtlStatus`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_issuer_proof_usage

- Parameters: `env: Env, issuer: Address`
- Result: `(u32, u32, u32, u32)`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_issuer_registry

- Parameters: `env: Env`
- Result: `Result<Address, ContractError>`
- Authorization: none
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_latest_upgrade_receipt

- Parameters: `env: Env`
- Result: `Option<UpgradeReceipt>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_migration_status

- Parameters: `env: Env`
- Result: `Option<MigrationStatus>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_pause_acknowledgement

- Parameters: `env: Env, acknowledgement_hash: BytesN<32>,`
- Result: `Option<PauseAcknowledgementRecord>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_pause_acknowledgements

- Parameters: `env: Env, cursor: u32, limit: u32,`
- Result: `Result<Vec<PauseAcknowledgementRecord>, ContractError>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_proof

- Parameters: `env: Env, proof_id_hash: BytesN<32>`
- Result: `Result<ProofRecord, ProofError>`
- Authorization: none
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_proof_payload

- Parameters: `env: Env, proof_id_hash: BytesN<32>,`
- Result: `Result<ProofPayloadRecord, ProofError>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_proof_subject_nullifier

- Parameters: `env: Env, proof_id_hash: BytesN<32>,`
- Result: `Result<SubjectNullifier, ProofError>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_proof_ttl_status

- Parameters: `env: Env, proof_id_hash: BytesN<32>`
- Result: `TtlStatus`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_proof_validity

- Parameters: `env: Env, proof_id_hash: BytesN<32>`
- Result: `ProofValidity`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_proof_validity_details

- Parameters: `env: Env, proof_id_hash: BytesN<32>,`
- Result: `Result<ProofValidityDetails, ProofError>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_protocol_config

- Parameters: `env: Env`
- Result: `Result<Address, ContractError>`
- Authorization: none
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_registry_epoch

- Parameters: `env: Env`
- Result: `u32`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_schema_rate_limit_usage

- Parameters: `env: Env, schema: u32,`
- Result: `Result<SchemaRateLimitUsage, ProofError>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_storage_layout

- Parameters: `env: Env`
- Result: `StorageLayoutStatus`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_successor

- Parameters: `env: Env`
- Result: `Option<Address>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_successors

- Parameters: `env: Env, proof_id_hash: BytesN<32>`
- Result: `soroban_sdk::Vec<BytesN<32>>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_upgrade_approval_metadata

- Parameters: `env: Env, target_hash: BytesN<32>`
- Result: `ApprovalQuery`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_upgrade_history

- Parameters: `env: Env, start: u32, limit: u32`
- Result: `Vec<UpgradeHistoryRecord>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::get_upgrade_history_count

- Parameters: `env: Env`
- Result: `u32`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::has_capability

- Parameters: `_env: Env, capability: u32`
- Result: `bool`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::initialize

- Parameters: `env: Env, admin: Address, issuer_registry: Address, protocol_config: Address,`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::initialize_in_domain

- Parameters: `env: Env, admin: Address, issuer_registry: Address, protocol_config: Address, deployment_domain: BytesN<32>,`
- Result: `Result<DeploymentIdentity, ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::interface_version

- Parameters: `_env: Env`
- Result: `InterfaceVersion`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::interface_version

- Parameters: `_env: Env`
- Result: `InterfaceVersion`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::is_active_address

- Parameters: `_env: Env, _issuer_address: Address`
- Result: `bool`
- Authorization: none
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::is_active_address

- Parameters: `_env: Env, _issuer_address: Address`
- Result: `bool`
- Authorization: none
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::is_archived

- Parameters: `env: Env, proof_id_hash: BytesN<32>`
- Result: `bool`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::is_decommissioned

- Parameters: `env: Env`
- Result: `bool`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::is_pause_acknowledger

- Parameters: `env: Env, address: Address`
- Result: `bool`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::is_paused

- Parameters: `_env: Env`
- Result: `bool`
- Authorization: none
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::is_paused

- Parameters: `_env: Env`
- Result: `bool`
- Authorization: none
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::is_revoked

- Parameters: `env: Env, proof_id_hash: BytesN<32>`
- Result: `bool`
- Authorization: none
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::is_schema_version_approved

- Parameters: `_env: Env, _version: u32`
- Result: `bool`
- Authorization: none
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::is_schema_version_approved

- Parameters: `_env: Env, _version: u32`
- Result: `bool`
- Authorization: none
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::is_upgrade_allowed

- Parameters: `env: Env, wasm_hash: BytesN<32>`
- Result: `bool`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::is_valid_proof

- Parameters: `env: Env, proof_id_hash: BytesN<32>`
- Result: `bool`
- Authorization: none
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::keepalive_instance

- Parameters: `env: Env`
- Result: `bool`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::keepalive_proof

- Parameters: `env: Env, proof_id_hash: BytesN<32>`
- Result: `bool`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::nominate_admin

- Parameters: `env: Env, new_admin: Address`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::nominate_successor

- Parameters: `env: Env, successor: Address`
- Result: `Result<(), ProofError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::open_dispute

- Parameters: `env: Env, proof_id_hash: BytesN<32>, disputant: Address, evidence_commitment: BytesN<32>,`
- Result: `Result<(), ProofError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::pause_scope

- Parameters: `env: Env, scope: PauseScope`
- Result: `Result<(), ProofError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::proof_validity

- Parameters: `env: Env, proof_id_hash: BytesN<32>`
- Result: `ProofValidity`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::refresh_instance_ttl

- Parameters: `env: Env`
- Result: `Result<TtlStatus, ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::register_proof

- Parameters: `env: Env, proof_id_hash: BytesN<32>, commitment_hash: BytesN<32>, issuer_address: Address, schema_version: u32, expires_at: u64, predecessor_id_hash: Option<BytesN<32>>, proof_type: BytesN<32>,`
- Result: `Result<(), ProofError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::register_proof_with_activation

- Parameters: `env: Env, proof_id_hash: BytesN<32>, commitment_hash: BytesN<32>, issuer_address: Address, schema_version: u32, expires_at: u64, activates_at: u64,`
- Result: `Result<(), ProofError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::register_proof_with_nullifier

- Parameters: `env: Env, proof_id_hash: BytesN<32>, commitment_hash: BytesN<32>, issuer_address: Address, schema_version: u32, expires_at: u64, predecessor_id_hash: Option<BytesN<32>>, proof_type: BytesN<32>, nullifier: SubjectNullifier,`
- Result: `Result<(), ProofError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::register_proof_with_payload

- Parameters: `env: Env, proof_id_hash: BytesN<32>, commitment_hash: BytesN<32>, issuer_address: Address, schema_version: u32, expires_at: u64, payload: Bytes,`
- Result: `Result<(), ProofError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::register_proof_with_predecessor

- Parameters: `env: Env, proof_id_hash: BytesN<32>, commitment_hash: BytesN<32>, issuer_address: Address, schema_version: u32, expires_at: u64, predecessor_id_hash: Option<BytesN<32>>, proof_type: BytesN<32>,`
- Result: `Result<(), ProofError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::register_proofs_batch

- Parameters: `env: Env, entries: Vec<ProofRegistrationInput>, issuer_address: Address,`
- Result: `Result<(), ProofError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::register_typed_proof

- Parameters: `env: Env, proof_id_hash: BytesN<32>, commitment_hash: BytesN<32>, issuer_address: Address, schema_version: u32, expires_at: u64, proof_type: BytesN<32>,`
- Result: `Result<(), ProofError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::register_typed_proof_payload

- Parameters: `env: Env, proof_id_hash: BytesN<32>, commitment_hash: BytesN<32>, issuer_address: Address, schema_version: u32, expires_at: u64, proof_type: BytesN<32>, payload: Bytes,`
- Result: `Result<(), ProofError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::reject_dispute

- Parameters: `env: Env, proof_id_hash: BytesN<32>`
- Result: `Result<(), ProofError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::resolve_dispute

- Parameters: `env: Env, proof_id_hash: BytesN<32>`
- Result: `Result<(), ProofError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::revoke_proof

- Parameters: `env: Env, proof_id_hash: BytesN<32>`
- Result: `Result<(), ProofError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::revoke_proofs_batch

- Parameters: `env: Env, proof_id_hashes: Vec<BytesN<32>>,`
- Result: `Result<(), ProofError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::revoke_upgrade

- Parameters: `env: Env, wasm_hash: BytesN<32>`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::revoke_upgrade_approval

- Parameters: `env: Env`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::set_deployment_domain

- Parameters: `env: Env, deployment_domain: BytesN<32>,`
- Result: `Result<DeploymentIdentity, ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::set_issuer_proof_caps

- Parameters: `env: Env, issuer: Address, max_active: u32, max_lifetime: u32,`
- Result: `Result<(), ProofError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::set_issuer_registry

- Parameters: `env: Env, new_issuer_registry: Address,`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::set_pause_acknowledger

- Parameters: `env: Env, address: Address, enabled: bool,`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::set_protocol_config

- Parameters: `env: Env, new_protocol_config: Address,`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::unpause_scope

- Parameters: `env: Env, scope: PauseScope`
- Result: `Result<(), ProofError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::upgrade_contract

- Parameters: `env: Env, wasm_hash: BytesN<32>`
- Result: `()`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## proof-registry::withdraw_dispute

- Parameters: `env: Env, proof_id_hash: BytesN<32>`
- Result: `Result<(), ProofError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/proof-registry/src/lib.rs`

## protocol-config::accept_admin

- Parameters: `env: Env`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::acknowledge_pause

- Parameters: `env: Env, scope: PauseScope, acknowledgement_hash: BytesN<32>, actor: Address,`
- Result: `Result<PauseAcknowledgementRecord, ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::activate_successor

- Parameters: `env: Env`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::advance_migration

- Parameters: `env: Env, expected_cursor: u32, processed_items: u32,`
- Result: `Result<MigrationStatus, ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::approval_eligibility

- Parameters: `env: Env, proposal_id: BytesN<32>, signer: Address,`
- Result: `GovernanceApprovalEligibility`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::approve_critical_action

- Parameters: `env: Env, proposal_id: BytesN<32>, signer: Address,`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::approve_emergency_action

- Parameters: `env: Env, proposal_id: BytesN<32>, guardian: Address,`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::approve_proof_type

- Parameters: `env: Env, proof_type: BytesN<32>`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::approve_schema_at

- Parameters: `env: Env, version: u32, activates_at: u64,`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::approve_schema_version

- Parameters: `env: Env, version: u32`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::approve_schema_with_predecessor

- Parameters: `env: Env, version: u32, predecessor: u32,`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::approve_upgrade

- Parameters: `env: Env, wasm_hash: BytesN<32>, new_version: u32`
- Result: `()`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::begin_decommission_grace

- Parameters: `env: Env, grace_ledgers: u32,`
- Result: `Result<DecommissionState, ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::begin_layout_migration

- Parameters: `env: Env, target_layout: u32,`
- Result: `Result<StorageLayoutStatus, ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::begin_migration

- Parameters: `env: Env, target_contract_version: u32, total_items: u32,`
- Result: `Result<MigrationStatus, ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::cancel_admin_transfer

- Parameters: `env: Env`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::cancel_critical_action

- Parameters: `env: Env, proposal_id: BytesN<32>`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::cancel_emergency_action

- Parameters: `env: Env, proposal_id: BytesN<32>`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::cleanup_expired_admin_transfer

- Parameters: `env: Env`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::complete_layout_migration

- Parameters: `env: Env, target_layout: u32,`
- Result: `Result<StorageLayoutStatus, ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::deprecate_proof_type

- Parameters: `env: Env, proof_type: BytesN<32>`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::deprecate_schema_to

- Parameters: `env: Env, deprecated_version: u32, replacement_version: u32, compatibility: SchemaCompatibility,`
- Result: `Result<SchemaReplacementPointer, ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::deprecate_schema_version

- Parameters: `env: Env, version: u32`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::derive_critical_action_id

- Parameters: `env: Env, action: CriticalAction, nonce: u32`
- Result: `BytesN<32>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::execute_critical_action

- Parameters: `env: Env, proposal_id: BytesN<32>`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::execute_emergency_action

- Parameters: `env: Env, proposal_id: BytesN<32>,`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::finalize_decommission

- Parameters: `env: Env`
- Result: `Result<DecommissionState, ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_admin

- Parameters: `env: Env`
- Result: `Result<Address, ContractError>`
- Authorization: none
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_admin_transfer_status

- Parameters: `env: Env`
- Result: `AdminTransferStatus`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_approver_snapshot

- Parameters: `env: Env, proposal_id: BytesN<32>,`
- Result: `Option<GovernanceApproverSnapshot>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_capabilities

- Parameters: `env: Env`
- Result: `ContractCapabilities`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_config_history

- Parameters: `env: Env, cursor: u32, limit: u32`
- Result: `Vec<ConfigChangeSummary>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_config_history_cursor

- Parameters: `env: Env`
- Result: `u32`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_config_version

- Parameters: `env: Env`
- Result: `u32`
- Authorization: none
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_contract_version

- Parameters: `env: Env`
- Result: `u32`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_critical_action_domain

- Parameters: `env: Env, proposal_id: BytesN<32>,`
- Result: `Option<GovernanceActionDomain>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_critical_action_policy

- Parameters: `env: Env`
- Result: `CriticalActionPolicy`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_critical_action_proposal

- Parameters: `env: Env, proposal_id: BytesN<32>,`
- Result: `Option<CriticalActionProposal>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_critical_action_snapshot

- Parameters: `env: Env, proposal_id: BytesN<32>,`
- Result: `Option<GovernanceApprovalSnapshot>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_critical_policy_version

- Parameters: `env: Env`
- Result: `u32`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_decommission_state

- Parameters: `env: Env`
- Result: `DecommissionState`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_deployment_identity

- Parameters: `env: Env`
- Result: `Option<DeploymentIdentity>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_dispute_filing_policy

- Parameters: `env: Env`
- Result: `DisputeFilingPolicy`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_emergency_policy

- Parameters: `env: Env`
- Result: `EmergencyFastTrackPolicy`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_emergency_proposal

- Parameters: `env: Env, proposal_id: BytesN<32>,`
- Result: `Option<EmergencyActionProposal>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_genesis

- Parameters: `env: Env`
- Result: `Result<GenesisRecord, ContractError>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_migration_status

- Parameters: `env: Env`
- Result: `Option<MigrationStatus>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_min_schema_activation_notice

- Parameters: `env: Env`
- Result: `u64`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_nullifier_domain_versions

- Parameters: `env: Env`
- Result: `Vec<u32>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_pause_acknowledgement

- Parameters: `env: Env, acknowledgement_hash: BytesN<32>,`
- Result: `Option<PauseAcknowledgementRecord>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_pause_acknowledgements

- Parameters: `env: Env, cursor: u32, limit: u32,`
- Result: `Result<Vec<PauseAcknowledgementRecord>, ContractError>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_schema_activation_notice

- Parameters: `env: Env, version: u32`
- Result: `Option<SchemaActivationNotice>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_schema_activation_state

- Parameters: `env: Env, version: u32`
- Result: `SchemaActivationState`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_schema_lifecycle

- Parameters: `env: Env, version: u32`
- Result: `SchemaLifecyclePolicy`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_schema_lineage

- Parameters: `env: Env, version: u32`
- Result: `Vec<u32>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_schema_payload_limit

- Parameters: `env: Env, version: u32`
- Result: `u32`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_schema_policy

- Parameters: `env: Env, version: u32`
- Result: `SchemaPolicy`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_schema_policy_digest

- Parameters: `env: Env, version: u32,`
- Result: `Result<SchemaPolicyDigest, ContractError>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_schema_policy_digest_version

- Parameters: ``
- Result: `u32`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_schema_predecessor

- Parameters: `env: Env, version: u32`
- Result: `Option<u32>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_schema_rate_limit

- Parameters: `env: Env, version: u32`
- Result: `SchemaRateLimit`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_schema_replacement

- Parameters: `env: Env, deprecated_version: u32,`
- Result: `Option<SchemaReplacementPointer>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_schema_replacement_chain

- Parameters: `env: Env, version: u32`
- Result: `Vec<u32>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_storage_layout

- Parameters: `env: Env`
- Result: `StorageLayoutStatus`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::get_successor

- Parameters: `env: Env`
- Result: `Option<Address>`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::has_capability

- Parameters: `_env: Env, capability: u32`
- Result: `bool`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::initialize

- Parameters: `env: Env, admin: Address`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::interface_version

- Parameters: `_env: Env`
- Result: `InterfaceVersion`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::is_algorithm_supported

- Parameters: `env: Env, algorithm: u32`
- Result: `bool`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::is_decommissioned

- Parameters: `env: Env`
- Result: `bool`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::is_nullifier_domain_supported

- Parameters: `env: Env, domain_version: u32`
- Result: `bool`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::is_pause_acknowledger

- Parameters: `env: Env, address: Address`
- Result: `bool`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::is_paused

- Parameters: `env: Env`
- Result: `bool`
- Authorization: none
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::is_proof_type_approved

- Parameters: `env: Env, proof_type: BytesN<32>`
- Result: `bool`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::is_schema_version_approved

- Parameters: `env: Env, version: u32`
- Result: `bool`
- Authorization: none
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::is_scope_paused

- Parameters: `env: Env, scope: PauseScope`
- Result: `bool`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::is_upgrade_allowed

- Parameters: `env: Env, wasm_hash: BytesN<32>`
- Result: `bool`
- Authorization: current admin
- Storage effect: read-only
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::keepalive_instance

- Parameters: `env: Env`
- Result: `bool`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::keepalive_proof_type

- Parameters: `env: Env, proof_type: BytesN<32>`
- Result: `bool`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::keepalive_schema_version

- Parameters: `env: Env, version: u32`
- Result: `bool`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::nominate_admin

- Parameters: `env: Env, new_admin: Address`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::nominate_successor

- Parameters: `env: Env, successor: Address`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::pause

- Parameters: `env: Env`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::propose_critical_action

- Parameters: `env: Env, action: CriticalAction,`
- Result: `Result<BytesN<32>, ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::propose_emergency_action

- Parameters: `env: Env, action: EmergencyAction, reason_commitment: BytesN<32>, expires_at: u32, proposer: Address,`
- Result: `Result<BytesN<32>, ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::revoke_upgrade

- Parameters: `env: Env, wasm_hash: BytesN<32>`
- Result: `()`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::set_commitment_algorithm

- Parameters: `env: Env, algorithm: u32, supported: bool,`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::set_critical_action_policy

- Parameters: `env: Env, policy: CriticalActionPolicy,`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::set_deployment_domain

- Parameters: `env: Env, deployment_domain: BytesN<32>,`
- Result: `Result<DeploymentIdentity, ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::set_dispute_filing_policy

- Parameters: `env: Env, policy: DisputeFilingPolicy,`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::set_emergency_policy

- Parameters: `env: Env, policy: EmergencyFastTrackPolicy,`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::set_min_schema_activation_notice

- Parameters: `env: Env, minimum_notice_seconds: u64,`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::set_nullifier_domain_version

- Parameters: `env: Env, domain_version: u32, supported: bool,`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::set_pause_acknowledger

- Parameters: `env: Env, address: Address, enabled: bool,`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::set_schema_payload_limit

- Parameters: `env: Env, version: u32, max_size: u32,`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::set_schema_policy

- Parameters: `env: Env, version: u32, proof_types: Vec<u32>, max_validity_seconds: u64,`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::set_schema_rate_limit

- Parameters: `env: Env, version: u32, max_registrations: u32, window_ledgers: u32,`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::set_scoped_pause

- Parameters: `env: Env, scope: PauseScope, paused: bool,`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::unpause

- Parameters: `env: Env`
- Result: `Result<(), ContractError>`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

## protocol-config::upgrade_contract

- Parameters: `env: Env, wasm_hash: BytesN<32>`
- Result: `()`
- Authorization: current admin
- Storage effect: documented in lifecycle specification
- Event effect: none unless documented in lifecycle specification
- Failure atomicity: Soroban invocation rollback
- Source: `contracts/protocol-config/src/lib.rs`

<!-- END GENERATED -->
