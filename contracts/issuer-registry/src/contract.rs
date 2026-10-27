//! Bounded enumeration of governance role assignments.
//!
//! The two directions of the governance index are answered here: the
//! assignments that hold a given role, and the roles a given principal holds.
//! Both are read through the index rather than through storage scans, so the
//! work a page costs is bounded by the page size and by the index bound, not
//! by the size of the assignment history.
//!
//! An assignment that has expired is never returned. It carries no authority,
//! so listing it as live would misreport who can act, and a client that
//! trusted the listing would believe a lapsed grant still holds.

use earnproof_shared::{
    ContractError, GovernanceRole, GovernanceRoleAssignment, RoleAssignmentListResponse,
    RoleAssignmentResponse, DEFAULT_QUERY_LIMIT, MAX_QUERY_LIMIT,
};
use soroban_sdk::{Address, Env, Vec};

use crate::{state, DataKey};

/// The number of entries a page may hold.
///
/// A caller that asks for more than the maximum still receives a bounded page:
/// the request is clamped rather than rejected, so a client that asks for
/// "everything" cannot make the contract do unbounded work.
fn page_limit(limit: Option<u32>) -> u32 {
    limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT)
}

/// Reads the assignment for one principal, if it is still in force.
///
/// `live` distinguishes the two questions an enumeration can ask. A grant that
/// has been made but whose activation ledger is still in the future is
/// reported by the recent query and not by the active one, because it does not
/// authorize anything yet.
fn assignment_for(
    env: &Env,
    role: GovernanceRole,
    principal: &Address,
    ledger: u32,
    live: bool,
) -> Option<RoleAssignmentResponse> {
    let assignment: GovernanceRoleAssignment = env
        .storage()
        .persistent()
        .get(&DataKey::GovernanceAssignment(role, principal.clone()))?;
    if assignment.expiration_ledger.map(|expiry| ledger >= expiry).unwrap_or(false) {
        return None;
    }
    if live && !assignment.is_active_at(ledger) {
        return None;
    }
    Some(RoleAssignmentResponse {
        role: assignment.role,
        principal: assignment.address,
        expires: assignment.expiration_ledger,
        activation_ledger: assignment.activation_ledger,
    })
}

/// Returns a bounded page of the assignments that currently hold `role`.
///
/// The page is walked in grant order, oldest first, starting at the slot
/// after `start_after`. `start_after` names a principal rather than a
/// position, so the same cursor works whether the caller is walking the whole
/// index or resuming a page it fetched earlier.
///
/// A principal that is not indexed cannot be a cursor: the index is ordered by
/// grant rather than by address, so there is no position to resume from and
/// the page starts at the beginning of the index.
pub fn query_role_assignments(
    env: &Env,
    role: GovernanceRole,
    start_after: Option<Address>,
    limit: Option<u32>,
) -> Result<RoleAssignmentListResponse, ContractError> {
    let limit = page_limit(limit);
    let count = state::role_index_count(env, role);
    let start = match start_after {
        Some(principal) => match state::role_index_slot(env, role, &principal) {
            Some(slot) => slot + 1,
            None => 0,
        },
        None => 0,
    };
    let ledger = env.ledger().sequence();
    let mut assignments = Vec::new(env);
    let mut slot = start;
    while slot < count && assignments.len() < limit {
        if let Some(principal) = state::role_index_entry(env, role, slot) {
            if let Some(entry) = assignment_for(env, role, &principal, ledger, true) {
                assignments.push_back(entry);
            }
        }
        slot += 1;
    }
    Ok(RoleAssignmentListResponse { assignments })
}

/// Returns a bounded page of the most recently granted assignments for
/// `role`.
///
/// The page is walked in reverse grant order, newest first, so an
/// administrator sees the latest grants rather than the oldest. Assignments
/// whose activation ledger is still in the future are included: they have been
/// granted but are not in force yet, and hiding them would hide a pending
/// grant. Expired assignments are excluded.
pub fn query_recent_role_assignments(
    env: &Env,
    role: GovernanceRole,
    start_after: Option<Address>,
    limit: Option<u32>,
) -> Result<RoleAssignmentListResponse, ContractError> {
    let limit = page_limit(limit);
    let count = state::role_index_count(env, role);
    // The cursor names the principal the page resumes below, in the newest
    // first order the page is written in.
    let end = match start_after {
        Some(principal) => match state::role_index_slot(env, role, &principal) {
            Some(slot) => slot,
            None => count,
        },
        None => count,
    };
    let ledger = env.ledger().sequence();
    let mut assignments = Vec::new(env);
    let mut slot = end;
    while slot > 0 && assignments.len() < limit {
        slot -= 1;
        if let Some(principal) = state::role_index_entry(env, role, slot) {
            if let Some(entry) = assignment_for(env, role, &principal, ledger, false) {
                assignments.push_back(entry);
            }
        }
    }
    Ok(RoleAssignmentListResponse { assignments })
}

/// Returns a bounded page of the roles `principal` currently holds.
///
/// The page is walked in grant order, oldest first, starting at the role
/// after `start_after`. A role that is not indexed cannot be a cursor, and
/// the page then starts at the beginning of the index.
pub fn query_principal_roles(
    env: &Env,
    principal: &Address,
    start_after: Option<GovernanceRole>,
    limit: Option<u32>,
) -> Result<RoleAssignmentListResponse, ContractError> {
    let limit = page_limit(limit);
    let count = state::principal_index_count(env, principal);
    let start = match start_after {
        Some(role) => match state::principal_index_slot(env, principal, role) {
            Some(slot) => slot + 1,
            None => 0,
        },
        None => 0,
    };
    let ledger = env.ledger().sequence();
    let mut assignments = Vec::new(env);
    let mut slot = start;
    while slot < count && assignments.len() < limit {
        if let Some(role) = state::principal_index_entry(env, principal, slot) {
            if let Some(entry) = assignment_for(env, role, principal, ledger, true) {
                assignments.push_back(entry);
            }
        }
        slot += 1;
    }
    Ok(RoleAssignmentListResponse { assignments })
}
