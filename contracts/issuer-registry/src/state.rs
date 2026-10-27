//! Two-way index over governance role assignments.
//!
//! A governance assignment is stored once, under
//! [`DataKey::GovernanceAssignment`], keyed by role and principal. That single
//! record answers "does this principal hold this role", but it cannot answer
//! "who holds this role" or "which roles does this principal hold": Soroban
//! cannot scan storage, so a question about the other side of the pair has no
//! answer without an index.
//!
//! This module keeps the two secondary indexes that make those questions
//! answerable. One is indexed by role and lists the principals that hold it;
//! the other is indexed by principal and lists the roles it holds. Both are
//! finite ordered lists with an explicit maximum count, and both are written
//! in the same call as the assignment itself, so a grant, a replacement, an
//! expiry, or a removal can never leave one index updated and the other not.
//!
//! The lists are ordered by grant, oldest first. That order is what makes a
//! grant replaceable without a scan of unbounded state: the entry for a
//! principal is found by walking a list whose length is bounded by
//! [`MAX_GOVERNANCE_ROLE_ASSIGNMENTS_PER_ROLE`] and
//! [`MAX_GOVERNANCE_ROLES_PER_PRINCIPAL`].

use earnproof_shared::{
    ContractError, GovernanceRole, MAX_GOVERNANCE_ROLE_ASSIGNMENTS_PER_ROLE,
    MAX_GOVERNANCE_ROLES_PER_PRINCIPAL, TTL_EXTEND_TO_LEDGERS, TTL_THRESHOLD_LEDGERS,
};
use soroban_sdk::{Address, Env};

use crate::DataKey;

/// Number of principals indexed under `role`.
pub fn role_index_count(env: &Env, role: GovernanceRole) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::RoleAssignmentIndexCount(role))
        .unwrap_or(0)
}

/// The principal indexed at `slot` under `role`, if that slot is populated.
pub fn role_index_entry(env: &Env, role: GovernanceRole, slot: u32) -> Option<Address> {
    env.storage()
        .persistent()
        .get(&DataKey::RoleAssignmentIndex(role, slot))
}

/// Number of governance roles indexed for `principal`.
pub fn principal_index_count(env: &Env, principal: &Address) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::PrincipalRoleIndexCount(principal.clone()))
        .unwrap_or(0)
}

/// The role indexed at `slot` for `principal`, if that slot is populated.
pub fn principal_index_entry(env: &Env, principal: &Address, slot: u32) -> Option<GovernanceRole> {
    env.storage()
        .persistent()
        .get(&DataKey::PrincipalRoleIndex(principal.clone(), slot))
}

/// The slot `principal` occupies in the index for `role`.
pub fn role_index_slot(env: &Env, role: GovernanceRole, principal: &Address) -> Option<u32> {
    for slot in 0..role_index_count(env, role) {
        if role_index_entry(env, role, slot).as_ref() == Some(principal) {
            return Some(slot);
        }
    }
    None
}

/// The slot `role` occupies in the index for `principal`.
pub fn principal_index_slot(env: &Env, principal: &Address, role: GovernanceRole) -> Option<u32> {
    for slot in 0..principal_index_count(env, principal) {
        if principal_index_entry(env, principal, slot) == Some(role) {
            return Some(slot);
        }
    }
    None
}

/// Drops `principal` from the index for `role` and closes the gap it leaves.
///
/// The tail is shifted down so the occupied slots stay contiguous, and the
/// count falls by one. The list is bounded, so the shift costs a number of
/// writes proportional to the bound rather than to the size of the history.
fn remove_from_role_index(env: &Env, role: GovernanceRole, principal: &Address) {
    let count = role_index_count(env, role);
    let Some(slot) = role_index_slot(env, role, principal) else {
        return;
    };
    let mut cursor = slot + 1;
    while cursor < count {
        let source = DataKey::RoleAssignmentIndex(role, cursor);
        let target = DataKey::RoleAssignmentIndex(role, cursor - 1);
        match env.storage().persistent().get::<_, Address>(&source) {
            Some(entry) => env.storage().persistent().set(&target, &entry),
            None => env.storage().persistent().remove(&target),
        };
        cursor += 1;
    }
    env.storage()
        .persistent()
        .remove(&DataKey::RoleAssignmentIndex(role, count - 1));
    env.storage()
        .persistent()
        .set(&DataKey::RoleAssignmentIndexCount(role), &(count - 1));
}

/// Drops `role` from the index for `principal` and closes the gap it leaves.
fn remove_from_principal_index(env: &Env, principal: &Address, role: GovernanceRole) {
    let count = principal_index_count(env, principal);
    let Some(slot) = principal_index_slot(env, principal, role) else {
        return;
    };
    let mut cursor = slot + 1;
    while cursor < count {
        let source = DataKey::PrincipalRoleIndex(principal.clone(), cursor);
        let target = DataKey::PrincipalRoleIndex(principal.clone(), cursor - 1);
        match env.storage().persistent().get::<_, GovernanceRole>(&source) {
            Some(entry) => env.storage().persistent().set(&target, &entry),
            None => env.storage().persistent().remove(&target),
        };
        cursor += 1;
    }
    env.storage()
        .persistent()
        .remove(&DataKey::PrincipalRoleIndex(principal.clone(), count - 1));
    env.storage().persistent().set(
        &DataKey::PrincipalRoleIndexCount(principal.clone()),
        &(count - 1),
    );
}

/// Indexes `principal` as a holder of `role`.
///
/// A pair that is already indexed is a replacement: the old entries are
/// dropped and the pair is appended afresh, so a principal is never indexed
/// twice for one role and the newest grant is the newest entry. A pair that
/// is not indexed is appended, provided both indexes have room: the role may
/// not hold more than [`MAX_GOVERNANCE_ROLE_ASSIGNMENTS_PER_ROLE`]
/// principals, and a principal may not hold more than
/// [`MAX_GOVERNANCE_ROLES_PER_PRINCIPAL`] roles.
///
/// The caller writes the assignment record in the same invocation, so a
/// failure here leaves no half-indexed state behind.
pub fn index_assignment(
    env: &Env,
    role: GovernanceRole,
    principal: &Address,
) -> Result<(), ContractError> {
    if role_index_slot(env, role, principal).is_some() {
        remove_from_role_index(env, role, principal);
        remove_from_principal_index(env, principal, role);
    }

    let role_count = role_index_count(env, role);
    if role_count >= MAX_GOVERNANCE_ROLE_ASSIGNMENTS_PER_ROLE {
        return Err(ContractError::InvalidInput);
    }
    let principal_count = principal_index_count(env, principal);
    if principal_count >= MAX_GOVERNANCE_ROLES_PER_PRINCIPAL {
        return Err(ContractError::InvalidInput);
    }

    let role_key = DataKey::RoleAssignmentIndex(role, role_count);
    env.storage().persistent().set(&role_key, principal);
    extend_ttl(env, &role_key);
    let role_count_key = DataKey::RoleAssignmentIndexCount(role);
    env.storage()
        .persistent()
        .set(&role_count_key, &(role_count + 1));
    extend_ttl(env, &role_count_key);

    let principal_key = DataKey::PrincipalRoleIndex(principal.clone(), principal_count);
    env.storage().persistent().set(&principal_key, &role);
    extend_ttl(env, &principal_key);
    let principal_count_key = DataKey::PrincipalRoleIndexCount(principal.clone());
    env.storage()
        .persistent()
        .set(&principal_count_key, &(principal_count + 1));
    extend_ttl(env, &principal_count_key);

    Ok(())
}

/// Removes `principal` from the index for `role`.
///
/// The assignment record itself is left alone: the caller decides whether the
/// assignment is being removed outright or only dropped from the index.
pub fn unindex_assignment(env: &Env, role: GovernanceRole, principal: &Address) {
    remove_from_role_index(env, role, principal);
    remove_from_principal_index(env, principal, role);
}

/// Keeps an index entry alive for as long as the assignment it points at.
fn extend_ttl(env: &Env, key: &DataKey) {
    env.storage()
        .persistent()
        .extend_ttl(key, TTL_THRESHOLD_LEDGERS, TTL_EXTEND_TO_LEDGERS);
}
