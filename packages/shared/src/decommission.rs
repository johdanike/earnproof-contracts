//! Bounded read-only grace period before final decommission.
//!
//! `activate_successor` decommissions a contract immediately. A retiring
//! contract often needs a window in which clients can still read from it and
//! learn the successor address, while no new state can be written. The grace
//! phases are:
//!
//! ```text
//! Active --begin_decommission_grace--> Grace --finalize_decommission--> Finalized
//! Active --activate_successor (legacy, immediate)-------------------> Finalized
//! ```
//!
//! * `Active`: every operation the contract normally allows.
//! * `Grace`: read-only. Every entrypoint that is rejected after final
//!   decommission is already rejected, plus the successor cannot change and
//!   the grace window cannot be extended, shortened, or cancelled. Reads,
//!   TTL keepalives, and `finalize_decommission` remain available.
//! * `Finalized`: the existing decommissioned state. Terminal.
//!
//! Permissions only narrow: no transition returns to an earlier phase, and a
//! finalized contract cannot be reopened by any entrypoint, upgrade, or
//! migration (the decommissioned flag and the grace record are never cleared).

use soroban_sdk::{contracttype, Address};

/// Encoding version of [`DecommissionState`].
pub const DECOMMISSION_STATE_VERSION: u32 = 1;

/// Shortest grace window, in ledgers.
pub const MIN_DECOMMISSION_GRACE_LEDGERS: u32 = 1;

/// Longest grace window, in ledgers (~30 days at 5 s per ledger).
pub const MAX_DECOMMISSION_GRACE_LEDGERS: u32 = 518_400;

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum DecommissionPhase {
    Active = 0,
    Grace = 1,
    Finalized = 2,
}

/// Query result of `get_decommission_state`. Ledger fields are `0` when the
/// phase never passed through the matching transition (for example a legacy
/// immediate decommission has no grace window).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecommissionState {
    pub state_version: u32,
    pub phase: DecommissionPhase,
    pub successor: Option<Address>,
    pub grace_started_ledger: u32,
    pub grace_ends_ledger: u32,
    pub finalized_ledger: u32,
}

/// Exclusive-start, inclusive-end rule: finalization is allowed at
/// `grace_ends_ledger` and later. Returns `None` for an out-of-range window or
/// arithmetic overflow.
pub fn grace_end_ledger(started_ledger: u32, grace_ledgers: u32) -> Option<u32> {
    if !(MIN_DECOMMISSION_GRACE_LEDGERS..=MAX_DECOMMISSION_GRACE_LEDGERS).contains(&grace_ledgers) {
        return None;
    }
    started_ledger.checked_add(grace_ledgers)
}

/// Only forward transitions are allowed.
pub fn is_decommission_transition_allowed(from: DecommissionPhase, to: DecommissionPhase) -> bool {
    matches!(
        (from, to),
        (DecommissionPhase::Active, DecommissionPhase::Grace)
            | (DecommissionPhase::Grace, DecommissionPhase::Finalized)
            | (DecommissionPhase::Active, DecommissionPhase::Finalized)
    )
}

/// Whether state-changing operations are permitted in `phase`.
pub fn decommission_phase_permits_mutation(phase: DecommissionPhase) -> bool {
    phase == DecommissionPhase::Active
}

/// Whether a grace record may be finalized at `current_ledger`.
pub fn can_finalize_decommission(state: &DecommissionState, current_ledger: u32) -> bool {
    state.phase == DecommissionPhase::Grace && current_ledger >= state.grace_ends_ledger
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grace(ends: u32) -> DecommissionState {
        DecommissionState {
            state_version: DECOMMISSION_STATE_VERSION,
            phase: DecommissionPhase::Grace,
            successor: None,
            grace_started_ledger: 10,
            grace_ends_ledger: ends,
            finalized_ledger: 0,
        }
    }

    #[test]
    fn grace_window_bounds_are_exact() {
        assert_eq!(grace_end_ledger(10, 0), None);
        assert_eq!(grace_end_ledger(10, 1), Some(11));
        assert_eq!(
            grace_end_ledger(10, MAX_DECOMMISSION_GRACE_LEDGERS),
            Some(10 + MAX_DECOMMISSION_GRACE_LEDGERS)
        );
        assert_eq!(
            grace_end_ledger(10, MAX_DECOMMISSION_GRACE_LEDGERS + 1),
            None
        );
        assert_eq!(grace_end_ledger(u32::MAX, 1), None);
    }

    #[test]
    fn transitions_only_move_forward() {
        use DecommissionPhase::*;
        assert!(is_decommission_transition_allowed(Active, Grace));
        assert!(is_decommission_transition_allowed(Grace, Finalized));
        assert!(is_decommission_transition_allowed(Active, Finalized));
        for (from, to) in [
            (Grace, Active),
            (Finalized, Active),
            (Finalized, Grace),
            (Grace, Grace),
            (Finalized, Finalized),
            (Active, Active),
        ] {
            assert!(!is_decommission_transition_allowed(from, to));
        }
    }

    #[test]
    fn only_active_permits_mutation() {
        assert!(decommission_phase_permits_mutation(
            DecommissionPhase::Active
        ));
        assert!(!decommission_phase_permits_mutation(
            DecommissionPhase::Grace
        ));
        assert!(!decommission_phase_permits_mutation(
            DecommissionPhase::Finalized
        ));
    }

    #[test]
    fn finalization_boundary_is_inclusive() {
        let state = grace(20);
        assert!(!can_finalize_decommission(&state, 19));
        assert!(can_finalize_decommission(&state, 20));
        assert!(can_finalize_decommission(&state, 21));
        let mut finalized = state.clone();
        finalized.phase = DecommissionPhase::Finalized;
        assert!(!can_finalize_decommission(&finalized, 30));
    }
}
