//! Storage layout versions, migration targets, and compatibility state.
//!
//! The *contract version* says which code is deployed. The *storage layout
//! version* says which shape the persisted state is in. They move separately:
//! an upgrade can ship code that understands a newer layout while the stored
//! state still has the older one until a reviewed migration completes.
//!
//! Every contract exposes `get_storage_layout`, returning a
//! [`StorageLayoutStatus`]:
//!
//! * `active_layout` is the layout the stored state is in. It is read from
//!   instance storage. A deployment that predates this key reports
//!   [`LEGACY_STORAGE_LAYOUT`].
//! * `code_layout` is the layout the deployed code writes for new deployments.
//! * `migration_target` is the layout a started migration is moving to, or the
//!   next reviewed step when a migration is required but not started, or `0`.
//! * `compatibility` classifies the pair (see [`StorageLayoutCompatibility`]).
//!
//! `active_layout` changes only in `complete_storage_layout_migration`, only
//! by exactly one reviewed step from [`REVIEWED_LAYOUT_MIGRATIONS`], and only
//! after `begin_storage_layout_migration` recorded the same target. It never
//! moves on upgrade alone, so a status of `Current` cannot be reached without
//! a completed migration.

use soroban_sdk::contracttype;

/// Encoding version of [`StorageLayoutStatus`].
pub const STORAGE_LAYOUT_STATUS_VERSION: u32 = 1;

/// Layout reported by deployments that predate the layout key.
pub const LEGACY_STORAGE_LAYOUT: u32 = 1;

/// Layout written by the current `issuer-registry` code.
pub const ISSUER_REGISTRY_STORAGE_LAYOUT: u32 = 2;
/// Layout written by the current `proof-registry` code.
pub const PROOF_REGISTRY_STORAGE_LAYOUT: u32 = 2;
/// Layout written by the current `protocol-config` code.
pub const PROTOCOL_CONFIG_STORAGE_LAYOUT: u32 = 2;

/// Reviewed `(from, to)` layout steps. Layout 2 adds only new instance keys
/// (deployment domain, decommission grace record, layout version, pending
/// layout target); no existing entry is rewritten, so the step needs no data
/// batches.
pub const REVIEWED_LAYOUT_MIGRATIONS: [(u32, u32); 1] = [(1, 2)];

/// Classification of the stored layout against the deployed code.
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum StorageLayoutCompatibility {
    /// Stored layout equals the code layout.
    Current = 0,
    /// Stored layout is older and a reviewed step exists; not started.
    MigrationRequired = 1,
    /// A reviewed step was started and has not completed.
    MigrationInProgress = 2,
    /// Stored layout is newer than the code, or no reviewed path exists.
    Unsupported = 3,
}

/// Query result of `get_storage_layout`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageLayoutStatus {
    pub status_version: u32,
    pub active_layout: u32,
    pub code_layout: u32,
    pub migration_target: u32,
    pub compatibility: StorageLayoutCompatibility,
}

/// Whether `from -> to` is a reviewed single step.
pub fn is_reviewed_layout_migration(from: u32, to: u32) -> bool {
    REVIEWED_LAYOUT_MIGRATIONS
        .iter()
        .any(|(reviewed_from, reviewed_to)| *reviewed_from == from && *reviewed_to == to)
}

/// The next reviewed step from `active`, bounded by `code`.
pub fn next_layout_step(active: u32, code: u32) -> Option<u32> {
    REVIEWED_LAYOUT_MIGRATIONS
        .iter()
        .find(|(from, to)| *from == active && *to <= code)
        .map(|(_, to)| *to)
}

/// Computes the layout status. `pending_target` is the target recorded by a
/// started migration, if any.
pub fn storage_layout_status(
    active_layout: u32,
    code_layout: u32,
    pending_target: Option<u32>,
) -> StorageLayoutStatus {
    let (migration_target, compatibility) = if active_layout > code_layout {
        (0, StorageLayoutCompatibility::Unsupported)
    } else if let Some(target) = pending_target {
        (target, StorageLayoutCompatibility::MigrationInProgress)
    } else if active_layout == code_layout {
        (0, StorageLayoutCompatibility::Current)
    } else {
        match next_layout_step(active_layout, code_layout) {
            Some(target) => (target, StorageLayoutCompatibility::MigrationRequired),
            None => (0, StorageLayoutCompatibility::Unsupported),
        }
    };
    StorageLayoutStatus {
        status_version: STORAGE_LAYOUT_STATUS_VERSION,
        active_layout,
        code_layout,
        migration_target,
        compatibility,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_layout_has_no_target() {
        let status = storage_layout_status(2, 2, None);
        assert_eq!(status.compatibility, StorageLayoutCompatibility::Current);
        assert_eq!(status.migration_target, 0);
        assert_eq!(status.status_version, STORAGE_LAYOUT_STATUS_VERSION);
    }

    #[test]
    fn legacy_layout_requires_the_reviewed_step() {
        let status = storage_layout_status(LEGACY_STORAGE_LAYOUT, 2, None);
        assert_eq!(
            status.compatibility,
            StorageLayoutCompatibility::MigrationRequired
        );
        assert_eq!(status.migration_target, 2);
        assert_eq!(status.active_layout, 1);
    }

    #[test]
    fn started_migration_never_reports_current() {
        let status = storage_layout_status(1, 2, Some(2));
        assert_eq!(
            status.compatibility,
            StorageLayoutCompatibility::MigrationInProgress
        );
        assert_eq!(status.active_layout, 1);
        assert_eq!(status.migration_target, 2);
    }

    #[test]
    fn newer_or_unreviewed_layouts_are_unsupported() {
        assert_eq!(
            storage_layout_status(3, 2, None).compatibility,
            StorageLayoutCompatibility::Unsupported
        );
        assert_eq!(
            storage_layout_status(3, 2, Some(3)).compatibility,
            StorageLayoutCompatibility::Unsupported
        );
        assert_eq!(
            storage_layout_status(0, 2, None).compatibility,
            StorageLayoutCompatibility::Unsupported
        );
    }

    #[test]
    fn only_single_reviewed_steps_are_accepted() {
        assert!(is_reviewed_layout_migration(1, 2));
        assert!(!is_reviewed_layout_migration(2, 1));
        assert!(!is_reviewed_layout_migration(1, 3));
        assert!(!is_reviewed_layout_migration(2, 2));
        assert_eq!(next_layout_step(1, 2), Some(2));
        assert_eq!(next_layout_step(1, 1), None);
        assert_eq!(next_layout_step(2, 2), None);
    }

    #[test]
    fn contract_code_layouts_are_reachable_from_legacy() {
        for code in [
            ISSUER_REGISTRY_STORAGE_LAYOUT,
            PROOF_REGISTRY_STORAGE_LAYOUT,
            PROTOCOL_CONFIG_STORAGE_LAYOUT,
        ] {
            let mut layout = LEGACY_STORAGE_LAYOUT;
            while let Some(next) = next_layout_step(layout, code) {
                assert_eq!(next, layout + 1);
                layout = next;
            }
            assert_eq!(layout, code);
        }
    }
}
