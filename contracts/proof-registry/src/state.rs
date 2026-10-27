use cosmwasm_std::Uint64;
use cw_storage_plus::{Item, Index, IndexList, IndexedMap, MultiIndex};
use shared::types::{Proof, ProofStatus};

// Global sequence for registry state transitions
pub const REGISTRY_SEQUENCE: Item<Uint64> = Item::new("registry_sequence");
pub const REVOCATION_COMMITMENT: Item<String> = Item::new("revocation_commitment");

// Dispute indices for bounded queries without full table scans
pub struct ProofIndexes<'a> {
    pub open_disputes: MultiIndex<'a, u8, Proof, String>,
    pub closed_disputes: MultiIndex<'a, u8, Proof, String>,
}

impl<'a> IndexList<Proof> for ProofIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Proof>> + '_> {
        let v: Vec<&dyn Index<Proof>> = vec![&self.open_disputes, &self.closed_disputes];
        Box::new(v.into_iter())
    }
}

pub fn proofs<'a>() -> IndexedMap<'a, &'a str, Proof, ProofIndexes<'a>> {
    let indexes = ProofIndexes {
        open_disputes: MultiIndex::new(
            |_pk, p| if p.status == ProofStatus::Disputed { 1 } else { 0 },
            "proofs",
            "proofs__open_disputes",
        ),
        closed_disputes: MultiIndex::new(
            |_pk, p| if p.status == ProofStatus::Resolved { 1 } else { 0 },
            "proofs",
            "proofs__closed_disputes",
        ),
    };
    IndexedMap::new("proofs", indexes)
}