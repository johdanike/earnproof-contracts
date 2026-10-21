//! Shared query and response types for governance role enumeration.
//!
//! The issuer registry keeps a two-way index of governance role assignments:
//! one index by role and one by principal. A caller that wants to know who
//! holds a role, or which roles a principal holds, asks for one direction of
//! that index through a bounded page.
//!
//! Every page is bounded twice over. The caller asks for at most
//! [`MAX_QUERY_LIMIT`] entries, and the registry never serves more than that
//! even when a larger limit is requested, so the response size is a property
//! of the contract rather than of the request.

use soroban_sdk::{contracttype, Address, Vec};

use crate::GovernanceRole;

/// Maximum number of assignments one enumeration page may return.
///
/// A larger requested limit is clamped to this value rather than rejected, so
/// a client that asks for "everything" still receives a bounded page.
///
/// The bound is set by the ledger rather than by taste: an enumerated
/// assignment costs two reads, the index entry and the assignment it points
/// at, and a network invocation may touch a hundred ledger entries. A page of
/// thirty-two therefore stays well inside that limit.
pub const MAX_QUERY_LIMIT: u32 = 32;

/// Number of assignments returned when the caller does not ask for a page size.
pub const DEFAULT_QUERY_LIMIT: u32 = 10;

/// One governance role assignment, as returned by an enumeration query.
///
/// Expired assignments are never returned: an assignment that has lapsed
/// carries no authority, and listing it as live would misreport who can act.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleAssignmentResponse {
    /// The role the principal holds.
    pub role: GovernanceRole,
    /// The principal the role is held by.
    pub principal: Address,
    /// Ledger at which the assignment expires. `None` means the assignment
    /// does not expire.
    pub expires: Option<u32>,
    /// Ledger from which the assignment is in force. An assignment whose
    /// activation ledger is still in the future is granted but not yet live.
    pub activation_ledger: u32,
}

/// A bounded page of governance role assignments.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleAssignmentListResponse {
    /// The assignments in the page, in the order the query returns them.
    pub assignments: Vec<RoleAssignmentResponse>,
}
