//! Bounded enumeration of governance role assignments.
//!
//! The registry keeps a two-way index of governance role assignments: one
//! index by role, one by principal. These tests pin the behaviour an
//! administrator relies on when they ask who holds a role:
//!
//! * an expired assignment is never listed and never authorizes anything;
//! * the page is bounded, and paging through it visits every entry once;
//! * a repeated grant replaces the assignment rather than duplicating it;
//! * a removal drops the assignment from both indexes;
//! * an assignment granted before the indexes existed can be migrated into
//!   them; and
//! * the indexes refuse to grow past their documented bounds.
//!
//! The harness authorizes every call with a matching-mode auth entry, so an
//! enumeration that is signed by the wrong identity is rejected by the host
//! rather than silently admitted. That is what makes "enumeration confers no
//! authority" an executable claim.

use earnproof_shared::{
    GovernanceRole, GovernanceRoleAssignment, RoleAssignmentListResponse,
    DEFAULT_QUERY_LIMIT, MAX_GOVERNANCE_ROLE_ASSIGNMENTS_PER_ROLE, MAX_GOVERNANCE_ROLES_PER_PRINCIPAL,
    MAX_QUERY_LIMIT,
};
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::{Address, IntoVal, Symbol};

use crate::harness::{authorize, hash, Deployment};

const ROLE: GovernanceRole = GovernanceRole::IssuerManagement;
const OTHER_ROLE: GovernanceRole = GovernanceRole::SchemaManagement;

/// Grants `role` to `principal` as the administrator.
///
/// The proposal identifier is derived from `proposal`, so two grants in the
/// same test never share one.
fn grant(
    deployment: &Deployment,
    role: GovernanceRole,
    principal: &Address,
    activation_ledger: u32,
    expiration_ledger: Option<u32>,
    proposal: u8,
) {
    let proposal_id = hash(&deployment.env, proposal);
    authorize(
        &deployment.env,
        &deployment.admin,
        &deployment.issuers_address,
        "grant_governance_role",
        (
            &proposal_id,
            &role,
            principal,
            &activation_ledger,
            &expiration_ledger,
        )
            .into_val(&deployment.env),
    );
    deployment.issuers.grant_governance_role(
        &proposal_id,
        &role,
        principal,
        &activation_ledger,
        &expiration_ledger,
    );
}

/// Reads a page of the assignments that currently hold `role`.
fn role_page(
    deployment: &Deployment,
    role: GovernanceRole,
    start_after: Option<Address>,
    limit: Option<u32>,
) -> RoleAssignmentListResponse {
    authorize(
        &deployment.env,
        &deployment.admin,
        &deployment.issuers_address,
        "get_role_assignments",
        (&role, &start_after, &limit).into_val(&deployment.env),
    );
    deployment
        .issuers
        .get_role_assignments(&role, &start_after, &limit)
}

/// Reads a page of the most recently granted assignments for `role`.
fn recent_page(
    deployment: &Deployment,
    role: GovernanceRole,
    start_after: Option<Address>,
    limit: Option<u32>,
) -> RoleAssignmentListResponse {
    authorize(
        &deployment.env,
        &deployment.admin,
        &deployment.issuers_address,
        "get_recent_role_assignments",
        (&role, &start_after, &limit).into_val(&deployment.env),
    );
    deployment
        .issuers
        .get_recent_role_assignments(&role, &start_after, &limit)
}

/// Reads a page of the roles `principal` currently holds.
fn principal_page(
    deployment: &Deployment,
    principal: &Address,
    start_after: Option<GovernanceRole>,
    limit: Option<u32>,
) -> RoleAssignmentListResponse {
    authorize(
        &deployment.env,
        &deployment.admin,
        &deployment.issuers_address,
        "get_principal_roles",
        (principal, &start_after, &limit).into_val(&deployment.env),
    );
    deployment
        .issuers
        .get_principal_roles(principal, &start_after, &limit)
}

fn principals(page: &RoleAssignmentListResponse) -> std::vec::Vec<Address> {
    page.assignments
        .iter()
        .map(|entry| entry.principal.clone())
        .collect()
}

#[test]
fn a_grant_is_visible_from_both_directions() {
    let deployment = Deployment::new();
    let holder = Address::generate(&deployment.env);
    let now = deployment.env.ledger().sequence();

    grant(&deployment, ROLE, &holder, now, None, 0xA1);

    let by_role = role_page(&deployment, ROLE, None, None);
    assert_eq!(principals(&by_role), std::vec![holder.clone()]);
    assert_eq!(by_role.assignments.get(0).unwrap().role, ROLE);
    assert_eq!(by_role.assignments.get(0).unwrap().expires, None);

    let by_principal = principal_page(&deployment, &holder, None, None);
    assert_eq!(principals(&by_principal), std::vec![holder]);
}

#[test]
fn a_grant_for_one_role_does_not_appear_under_another() {
    let deployment = Deployment::new();
    let holder = Address::generate(&deployment.env);
    let now = deployment.env.ledger().sequence();

    grant(&deployment, ROLE, &holder, now, None, 0xA1);

    assert!(role_page(&deployment, OTHER_ROLE, None, None)
        .assignments
        .is_empty());
}

#[test]
fn pagination_visits_every_assignment_once() {
    let deployment = Deployment::new();
    let now = deployment.env.ledger().sequence();
    let holders: std::vec::Vec<Address> = (0..5)
        .map(|_| Address::generate(&deployment.env))
        .collect();
    for (index, holder) in holders.iter().enumerate() {
        grant(
            &deployment,
            ROLE,
            holder,
            now,
            None,
            0xB0 + index as u8,
        );
    }

    let mut seen: std::vec::Vec<Address> = std::vec::Vec::new();
    let mut cursor: Option<Address> = None;
    loop {
        let page = role_page(&deployment, ROLE, cursor.clone(), Some(2));
        if page.assignments.is_empty() {
            break;
        }
        assert!(page.assignments.len() <= 2, "a page never exceeds the limit");
        for entry in page.assignments.iter() {
            assert!(
                !seen.contains(&entry.principal),
                "an assignment was listed twice"
            );
            seen.push(entry.principal.clone());
        }
        cursor = Some(page.assignments.get(page.assignments.len() - 1).unwrap().principal.clone());
    }
    assert_eq!(seen, holders);
}

#[test]
fn a_repeated_grant_replaces_the_assignment_instead_of_duplicating_it() {
    let deployment = Deployment::new();
    let holder = Address::generate(&deployment.env);
    let now = deployment.env.ledger().sequence();

    grant(&deployment, ROLE, &holder, now, None, 0xC1);
    grant(
        &deployment,
        ROLE,
        &holder,
        now,
        Some(now + 500),
        0xC2,
    );

    let by_role = role_page(&deployment, ROLE, None, None);
    assert_eq!(by_role.assignments.len(), 1);
    assert_eq!(
        by_role.assignments.get(0).unwrap().expires,
        Some(now + 500),
        "the replacement carries the new expiry"
    );

    let by_principal = principal_page(&deployment, &holder, None, None);
    assert_eq!(by_principal.assignments.len(), 1);
}

#[test]
fn the_recent_query_returns_the_newest_grant_first() {
    let deployment = Deployment::new();
    let now = deployment.env.ledger().sequence();
    let first = Address::generate(&deployment.env);
    let second = Address::generate(&deployment.env);

    grant(&deployment, ROLE, &first, now, None, 0xD1);
    grant(&deployment, ROLE, &second, now, None, 0xD2);

    let recent = recent_page(&deployment, ROLE, None, None);
    assert_eq!(principals(&recent), std::vec![second, first]);
}

#[test]
fn a_pending_grant_is_listed_as_recent_but_not_as_active() {
    let deployment = Deployment::new();
    let holder = Address::generate(&deployment.env);
    let now = deployment.env.ledger().sequence();

    // A role activation delay puts the grant in the future, so it has been
    // made but is not in force.
    authorize(
        &deployment.env,
        &deployment.admin,
        &deployment.issuers_address,
        "set_role_activation_delay",
        (&ROLE, &10_u32).into_val(&deployment.env),
    );
    deployment
        .issuers
        .set_role_activation_delay(&ROLE, &10);

    grant(&deployment, ROLE, &holder, now, None, 0xE1);

    assert!(
        role_page(&deployment, ROLE, None, None).assignments.is_empty(),
        "a grant that is not in force is not an active assignment"
    );
    assert_eq!(
        recent_page(&deployment, ROLE, None, None).assignments.len(),
        1,
        "a grant that is not in force is still a recent grant"
    );
}

#[test]
fn an_assignment_is_active_up_to_the_ledger_before_its_expiry() {
    let deployment = Deployment::new();
    let holder = Address::generate(&deployment.env);
    let now = deployment.env.ledger().sequence();
    let expiry = now + 10;

    grant(&deployment, ROLE, &holder, now, Some(expiry), 0xF1);

    deployment.env.ledger().set_sequence_number(expiry - 1);
    assert_eq!(
        role_page(&deployment, ROLE, None, None).assignments.len(),
        1,
        "the assignment is live on the ledger before its expiry"
    );

    deployment.env.ledger().set_sequence_number(expiry);
    assert!(
        role_page(&deployment, ROLE, None, None).assignments.is_empty(),
        "the assignment lapses on its expiry ledger"
    );
    assert!(
        recent_page(&deployment, ROLE, None, None).assignments.is_empty(),
        "an expired assignment is not listed as a recent grant either"
    );
}

#[test]
fn an_expired_assignment_authorizes_nothing() {
    let deployment = Deployment::new();
    let holder = Address::generate(&deployment.env);
    let now = deployment.env.ledger().sequence();
    let expiry = now + 10;

    grant(&deployment, ROLE, &holder, now, Some(expiry), 0x11);
    deployment.env.ledger().set_sequence_number(expiry);

    // The holder is named in the registry, and the role is not in force. The
    // call the role would authorize must still be rejected.
    assert_eq!(
        deployment
            .issuers
            .governance_role_state(&ROLE, &holder),
        earnproof_shared::GovernanceRoleState::Expired
    );
    let outcome = deployment
        .issuers
        .try_suspend_issuer_by_role(&hash(&deployment.env, 0x12), &deployment.issuer_id, &hash(&deployment.env, 0x13), &holder);
    assert!(outcome.is_err(), "an expired role authorizes nothing");
}

#[test]
fn a_removal_drops_the_assignment_from_both_indexes() {
    let deployment = Deployment::new();
    let holder = Address::generate(&deployment.env);
    let now = deployment.env.ledger().sequence();

    grant(&deployment, ROLE, &holder, now, None, 0x21);

    authorize(
        &deployment.env,
        &deployment.admin,
        &deployment.issuers_address,
        "remove_governance_role",
        (&ROLE, &holder).into_val(&deployment.env),
    );
    deployment.issuers.remove_governance_role(&ROLE, &holder);

    assert!(role_page(&deployment, ROLE, None, None)
        .assignments
        .is_empty());
    assert!(principal_page(&deployment, &holder, None, None)
        .assignments
        .is_empty());
}

#[test]
fn an_expired_assignment_is_swept_out_of_the_indexes() {
    let deployment = Deployment::new();
    let holder = Address::generate(&deployment.env);
    let now = deployment.env.ledger().sequence();

    grant(&deployment, ROLE, &holder, now, Some(now + 5), 0x31);
    deployment.env.ledger().set_sequence_number(now + 5);

    authorize(
        &deployment.env,
        &deployment.admin,
        &deployment.issuers_address,
        "expire_role_assignments",
        (&ROLE,).into_val(&deployment.env),
    );
    let expired = deployment.issuers.expire_role_assignments(&ROLE);
    assert_eq!(expired, 1);

    // The assignment record is kept: the historical fact that the role was
    // held does not change because the index was swept.
    assert_eq!(
        deployment
            .issuers
            .governance_role_state(&ROLE, &holder),
        earnproof_shared::GovernanceRoleState::Expired
    );
}

#[test]
fn an_assignment_predating_the_indexes_is_migrated_into_them() {
    let deployment = Deployment::new();
    let holder = Address::generate(&deployment.env);
    let now = deployment.env.ledger().sequence();

    // Write the assignment record directly, the way a contract that predates
    // the indexes would have left it.
    let assignment = GovernanceRoleAssignment {
        role: ROLE,
        address: holder.clone(),
        activation_ledger: now,
        expiration_ledger: None,
        proposal_id: hash(&deployment.env, 0x41),
    };
    let key = (
        Symbol::new(&deployment.env, "GovernanceAssignment"),
        ROLE,
        holder.clone(),
    );
    deployment.env.as_contract(&deployment.issuers_address, || {
        deployment
            .env
            .storage()
            .persistent()
            .set(&key, &assignment);
    });

    assert!(
        role_page(&deployment, ROLE, None, None).assignments.is_empty(),
        "an unindexed assignment is not enumerated"
    );

    authorize(
        &deployment.env,
        &deployment.admin,
        &deployment.issuers_address,
        "migrate_role_assignment",
        (&ROLE, &holder).into_val(&deployment.env),
    );
    assert!(deployment.issuers.migrate_role_assignment(&ROLE, &holder));

    assert_eq!(role_page(&deployment, ROLE, None, None).assignments.len(), 1);

    // The migration is idempotent: a second call changes nothing.
    authorize(
        &deployment.env,
        &deployment.admin,
        &deployment.issuers_address,
        "migrate_role_assignment",
        (&ROLE, &holder).into_val(&deployment.env),
    );
    assert!(!deployment.issuers.migrate_role_assignment(&ROLE, &holder));
    assert_eq!(role_page(&deployment, ROLE, None, None).assignments.len(), 1);
}

#[test]
fn the_role_index_refuses_to_grow_past_its_bound() {
    let deployment = Deployment::new();
    let now = deployment.env.ledger().sequence();

    for index in 0..MAX_GOVERNANCE_ROLE_ASSIGNMENTS_PER_ROLE {
        let holder = Address::generate(&deployment.env);
        grant(&deployment, ROLE, &holder, now, None, 0x10 + index as u8);
    }

    let overflow = Address::generate(&deployment.env);
    let proposal_id = hash(&deployment.env, 0x7F);
    authorize(
        &deployment.env,
        &deployment.admin,
        &deployment.issuers_address,
        "grant_governance_role",
        (&proposal_id, &ROLE, &overflow, &now, &None::<u32>).into_val(&deployment.env),
    );
    let outcome = deployment.issuers.try_grant_governance_role(
        &proposal_id,
        &ROLE,
        &overflow,
        &now,
        &None,
    );
    assert!(outcome.is_err(), "the role index is bounded");

    // The rejected grant left no trace: the assignment was not written.
    assert_eq!(
        deployment.issuers.governance_role_state(&ROLE, &overflow),
        earnproof_shared::GovernanceRoleState::Missing
    );
}

#[test]
fn the_principal_index_refuses_to_grow_past_its_bound() {
    let deployment = Deployment::new();
    let now = deployment.env.ledger().sequence();
    let holder = Address::generate(&deployment.env);

    // `Recovery` is never grantable, so the grantable roles are the other
    // six. The principal bound is below that, so the grant is refused once
    // the principal holds enough roles.
    let roles = [
        GovernanceRole::ProtocolPause,
        GovernanceRole::SchemaManagement,
        GovernanceRole::IssuerManagement,
        GovernanceRole::ProofAdministration,
        GovernanceRole::DependencyManagement,
        GovernanceRole::UpgradeManagement,
    ];
    for (index, role) in roles.iter().enumerate() {
        let proposal_id = hash(&deployment.env, 0x50 + index as u8);
        authorize(
            &deployment.env,
            &deployment.admin,
            &deployment.issuers_address,
            "grant_governance_role",
            (&proposal_id, role, &holder, &now, &None::<u32>).into_val(&deployment.env),
        );
        let outcome = deployment.issuers.try_grant_governance_role(
            &proposal_id,
            role,
            &holder,
            &now,
            &None,
        );
        if index < MAX_GOVERNANCE_ROLES_PER_PRINCIPAL as usize {
            assert!(outcome.is_ok(), "role {index} should fit");
        } else {
            assert!(outcome.is_err(), "the principal index is bounded");
        }
    }
}

#[test]
fn a_page_is_bounded_and_defaults_to_the_documented_size() {
    let deployment = Deployment::new();
    // The default test budget caps the ledger entries one invocation may
    // touch. A page of fifty assignments reads fifty index entries and fifty
    // assignment records, which is past that cap but well inside the
    // protocol's own limits.
    deployment.env.cost_estimate().budget().reset_unlimited();
    let now = deployment.env.ledger().sequence();
    for index in 0..(MAX_QUERY_LIMIT + 5) {
        let holder = Address::generate(&deployment.env);
        grant(&deployment, ROLE, &holder, now, None, 0x10 + index as u8);
    }

    let page = role_page(&deployment, ROLE, None, Some(u32::MAX));
    assert_eq!(page.assignments.len(), MAX_QUERY_LIMIT);

    let default_page = role_page(&deployment, ROLE, None, None);
    assert_eq!(
        default_page.assignments.len(),
        DEFAULT_QUERY_LIMIT,
        "a caller that does not ask for a page size gets the default one"
    );
}

#[test]
fn enumeration_requires_the_administrator() {
    let deployment = Deployment::new();
    let now = deployment.env.ledger().sequence();
    let holder = Address::generate(&deployment.env);
    grant(&deployment, ROLE, &holder, now, None, 0x61);

    // An address that holds no authority cannot enumerate, even when it
    // signs the call itself.
    authorize(
        &deployment.env,
        &deployment.attacker(),
        &deployment.issuers_address,
        "get_role_assignments",
        (&ROLE, &None::<Address>, &None::<u32>).into_val(&deployment.env),
    );
    let outcome = deployment
        .issuers
        .try_get_role_assignments(&ROLE, &None, &None);
    assert!(
        outcome.is_err(),
        "enumeration is refused without the administrator's authority"
    );
}

#[test]
fn enumerating_an_assignment_does_not_confer_its_authority() {
    let deployment = Deployment::new();
    let now = deployment.env.ledger().sequence();
    let holder = Address::generate(&deployment.env);
    grant(&deployment, ROLE, &holder, now, None, 0x71);

    // The holder is listed, so the listing tells an administrator that the
    // address exists. It does not let that address act: the call still
    // demands the holder's own signature.
    assert_eq!(role_page(&deployment, ROLE, None, None).assignments.len(), 1);

    let outcome = deployment.issuers.try_suspend_issuer_by_role(
        &hash(&deployment.env, 0x72),
        &deployment.issuer_id,
        &hash(&deployment.env, 0x73),
        &holder,
    );
    assert!(
        outcome.is_err(),
        "listing an assignment does not authorize the holder"
    );
}

#[test]
fn a_cursor_names_a_principal_rather_than_a_position() {
    let deployment = Deployment::new();
    let now = deployment.env.ledger().sequence();
    let first = Address::generate(&deployment.env);
    let second = Address::generate(&deployment.env);
    let third = Address::generate(&deployment.env);
    grant(&deployment, ROLE, &first, now, None, 0x81);
    grant(&deployment, ROLE, &second, now, None, 0x82);
    grant(&deployment, ROLE, &third, now, None, 0x83);

    let page = role_page(&deployment, ROLE, Some(first.clone()), None);
    assert_eq!(principals(&page), std::vec![second, third]);
}

#[test]
fn the_principal_query_pages_over_the_roles_a_principal_holds() {
    let deployment = Deployment::new();
    let now = deployment.env.ledger().sequence();
    let holder = Address::generate(&deployment.env);

    grant(&deployment, ROLE, &holder, now, None, 0x91);
    grant(&deployment, OTHER_ROLE, &holder, now, None, 0x92);

    let page = principal_page(&deployment, &holder, None, Some(1));
    assert_eq!(page.assignments.len(), 1);

    let rest = principal_page(
        &deployment,
        &holder,
        Some(page.assignments.get(0).unwrap().role),
        None,
    );
    assert_eq!(rest.assignments.len(), 1);
    assert_eq!(rest.assignments.get(0).unwrap().role, OTHER_ROLE);
}

#[test]
fn an_unindexed_principal_holds_no_roles() {
    let deployment = Deployment::new();
    let stranger = Address::generate(&deployment.env);

    let page = principal_page(&deployment, &stranger, None, None);
    assert!(page.assignments.is_empty());
}
