use cosmwasm_std::Uint64;
use cw_storage_plus::{Item, Map, IndexedMap, IndexList, MultiIndex, Index};
use shared::types::{AdminTransferRecord, GovernanceRole, PauseTransitionRecord, PendingUpgradeApproval};

// Role Assignments: (Account, Role) -> ()
pub const GOVERNANCE_ROLES: Map<(&str, &str), ()> = Map::new("gov_roles");

// Monotonic Counters
pub const ADMIN_HISTORY_SEQ: Item<Uint64> = Item::new("admin_hist_seq");
pub const PAUSE_HISTORY_SEQ: Item<Uint64> = Item::new("pause_hist_seq");

// Bounded History Maps
pub const ADMIN_TRANSFER_HISTORY: Map<u64, AdminTransferRecord> = Map::new("admin_transfer_history");
pub const PAUSE_HISTORY: Map<u64, PauseTransitionRecord> = Map::new("pause_history");

// Pending Upgrades Indexing
pub struct UpgradeIndexes<'a> {
    pub target_hash: MultiIndex<'a, String, PendingUpgradeApproval, String>,
}

impl<'a> IndexList<PendingUpgradeApproval> for UpgradeIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<PendingUpgradeApproval>> + '_> {
        let v: Vec<&dyn Index<PendingUpgradeApproval>> = vec![&self.target_hash];
        Box::new(v.into_iter())
    }
}

pub fn pending_upgrades<'a>() -> IndexedMap<'a, &'a str, PendingUpgradeApproval, UpgradeIndexes<'a>> {
    let indexes = UpgradeIndexes {
        target_hash: MultiIndex::new(
            |_pk, u| u.target_hash.clone(),
            "pending_upgrades",
            "pending_upgrades__target",
        ),
    };
    IndexedMap::new("pending_upgrades", indexes)
}