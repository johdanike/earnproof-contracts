use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint64};

// --- Sequence & Proof State (#256) ---
#[cw_serde]
pub struct Proof {
    pub id: String,
    pub issuer: String,
    pub status: ProofStatus,
    pub sequence: Uint64, // Monotonic lifecycle sequence
}

#[cw_serde]
pub enum ProofStatus {
    Active,
    Disputed,
    Resolved,
    Revoked,
    Archived,
}

// --- Checkpoint & Digest Definitions (#254, #255) ---
#[cw_serde]
pub struct CheckpointDigestResponse {
    pub version: u32,
    pub sequence: Uint64,
    pub ledger_height: Uint64,
    pub component_counters: ComponentCounters,
    pub digest: String, // Deterministic hash of aggregate state
}

#[cw_serde]
pub struct ComponentCounters {
    pub issuers: Uint64,
    pub schemas: Uint64,
    pub proofs: Uint64,
}

#[cw_serde]
pub struct RevocationCheckpointResponse {
    pub version: u32,
    pub sequence: Uint64,
    pub ledger_height: Uint64,
    pub algorithm: String,
    pub root_commitment: String,
}

// --- Dispute Queries (#257) ---
#[cw_serde]
pub enum DisputeQueryMsg {
    OpenDisputes {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    ClosedDisputes {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}


// --- Operation-Scoped Governance Roles (#154) ---
#[cw_serde]
pub enum GovernanceRole {
    Pause,
    Schema,
    Issuer,
    Revocation,
    Upgrade,
    Recovery, // Explicit recovery hierarchy
}

// --- Admin Transfer History (#245) ---
#[cw_serde]
pub struct AdminTransferRecord {
    pub sequence: Uint64,
    pub old_admin: Addr,
    pub proposed_admin: Addr,
    pub actor: Addr,
    pub ledger_height: Uint64,
    pub timestamp: Timestamp,
    pub terminal_reason: String, // "Accepted", "Cancelled", "Expired"
}

// --- Scoped Pause Transition History (#244) ---
#[cw_serde]
pub struct PauseTransitionRecord {
    pub sequence: Uint64,
    pub scope: String, // "Global", "IssuerRegistry", etc.
    pub is_paused: bool,
    pub actor: Addr,
    pub ledger_height: Uint64,
    pub timestamp: Timestamp,
    pub incident_commitment: Option<String>,
}

// --- Pending Upgrade Approvals (#243) ---
#[cw_serde]
pub struct PendingUpgradeApproval {
    pub proposal_id: String,
    pub target_hash: String,
    pub version: String,
    pub expires_at: Timestamp,
    pub approval_count: u32,
}

// --- Bounded Queries ---
#[cw_serde]
pub enum GovernanceQueryMsg {
    AdminTransferHistory {
        start_after: Option<Uint64>,
        limit: Option<u32>,
    },
    PauseHistory {
        scope: Option<String>,
        start_after: Option<Uint64>,
        limit: Option<u32>,
    },
    PendingUpgrades {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    HasRole {
        account: String,
        role: GovernanceRole,
    },
}
