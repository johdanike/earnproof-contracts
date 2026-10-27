use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint64};
use shared::types::{GovernanceRole, PauseTransitionRecord};
use crate::state::{GOVERNANCE_ROLES, PAUSE_HISTORY, PAUSE_HISTORY_SEQ};

// Centralized role authorization guard
pub fn assert_role(deps: &DepsMut, account: &str, role: GovernanceRole) -> StdResult<()> {
    let role_str = serde_json::to_string(&role).unwrap();
    if GOVERNANCE_ROLES.has(deps.storage, (account, &role_str)) {
        Ok(())
    } else {
        Err(StdError::generic_err(format!("Unauthorized: Requires {:?} role", role)))
    }
}

// Example: Scoped Pause with History Recording (#154, #244)
pub fn execute_pause(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    scope: String,
    incident_commitment: Option<String>,
) -> StdResult<Response> {
    // 1. Authorization: Must possess the Pause role
    assert_role(&deps, info.sender.as_str(), GovernanceRole::Pause)?;

    // 2. State Mutation (Apply pause to protocol config...)
    
    // 3. Record Bounded History
    let mut seq = PAUSE_HISTORY_SEQ.load(deps.storage).unwrap_or(Uint64::zero());
    seq = seq.checked_add(Uint64::one())?;
    PAUSE_HISTORY_SEQ.save(deps.storage, &seq)?;

    let record = PauseTransitionRecord {
        sequence: seq,
        scope: scope.clone(),
        is_paused: true,
        actor: info.sender.clone(),
        ledger_height: Uint64::from(env.block.height),
        timestamp: env.block.time,
        incident_commitment: incident_commitment.clone(),
    };
    
    PAUSE_HISTORY.save(deps.storage, seq.u64(), &record)?;

    Ok(Response::new()
        .add_attribute("action", "pause")
        .add_attribute("scope", scope)
        .add_attribute("sequence", seq.to_string()))
}