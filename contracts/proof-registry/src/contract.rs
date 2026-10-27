use cosmwasm_std::{DepsMut, Env, Response, StdError, StdResult, Uint64};
use sha2::{Sha256, Digest};
use crate::state::{proofs, REGISTRY_SEQUENCE, REVOCATION_COMMITMENT};

// Helper: Increment proof sequence safely
pub fn increment_proof_sequence(proof: &mut Proof) -> StdResult<()> {
    proof.sequence = proof.sequence.checked_add(Uint64::one())
        .map_err(|_| StdError::generic_err("Proof sequence overflow"))?;
    Ok(())
}

// Helper: Update global revocation commitment deterministically
pub fn update_revocation_commitment(deps: DepsMut, env: &Env, proof_id: &str) -> StdResult<()> {
    let current_root = REVOCATION_COMMITMENT.load(deps.storage).unwrap_or_else(|_| "".to_string());
    
    // Deterministic chaining: Hash(current_root || proof_id || block_height)
    let mut hasher = Sha256::new();
    hasher.update(current_root.as_bytes());
    hasher.update(proof_id.as_bytes());
    hasher.update(&env.block.height.to_be_bytes());
    
    let next_root = hex::encode(hasher.finalize());
    REVOCATION_COMMITMENT.save(deps.storage, &next_root)?;
    Ok(())
}