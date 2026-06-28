#![no_std]

use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol};
use axionvera_state::{
    emit_state_transition, GovernanceState, RewardState, StateError, StakingState, TreasuryState,
    VaultState,
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StateDataKey {
    VaultState,
    StakingState,
    RewardState,
    TreasuryState,
    GovernanceState(Symbol), // Keyed by proposal ID symbol
}

// ===========================================================================
// VAULTS STORAGE & TRANSITIONS
// ===========================================================================

pub fn get_vault_state(e: &Env) -> VaultState {
    e.storage()
        .instance()
        .get(&StateDataKey::VaultState)
        .unwrap_or(VaultState::Uninitialized)
}

pub fn set_vault_state(
    e: &Env,
    new_state: VaultState,
    caller: Address,
) -> Result<VaultState, StateError> {
    let current = get_vault_state(e);
    let validated = current.transition(new_state)?;
    e.storage()
        .instance()
        .set(&StateDataKey::VaultState, &validated);
    emit_state_transition(
        e,
        symbol_short!("vault"),
        current as u32,
        validated as u32,
        caller,
    );
    Ok(validated)
}

// ===========================================================================
// STAKING STORAGE & TRANSITIONS
// ===========================================================================

pub fn get_staking_state(e: &Env) -> StakingState {
    e.storage()
        .instance()
        .get(&StateDataKey::StakingState)
        .unwrap_or(StakingState::Uninitialized)
}

pub fn set_staking_state(
    e: &Env,
    new_state: StakingState,
    caller: Address,
) -> Result<StakingState, StateError> {
    let current = get_staking_state(e);
    let validated = current.transition(new_state)?;
    e.storage()
        .instance()
        .set(&StateDataKey::StakingState, &validated);
    emit_state_transition(
        e,
        symbol_short!("staking"),
        current as u32,
        validated as u32,
        caller,
    );
    Ok(validated)
}

// ===========================================================================
// REWARDS STORAGE & TRANSITIONS
// ===========================================================================

pub fn get_reward_state(e: &Env) -> RewardState {
    e.storage()
        .instance()
        .get(&StateDataKey::RewardState)
        .unwrap_or(RewardState::Idle)
}

pub fn set_reward_state(
    e: &Env,
    new_state: RewardState,
    caller: Address,
) -> Result<RewardState, StateError> {
    let current = get_reward_state(e);
    let validated = current.transition(new_state)?;
    e.storage()
        .instance()
        .set(&StateDataKey::RewardState, &validated);
    emit_state_transition(
        e,
        symbol_short!("rewards"),
        current as u32,
        validated as u32,
        caller,
    );
    Ok(validated)
}

// ===========================================================================
// TREASURY STORAGE & TRANSITIONS
// ===========================================================================

pub fn get_treasury_state(e: &Env) -> TreasuryState {
    e.storage()
        .instance()
        .get(&StateDataKey::TreasuryState)
        .unwrap_or(TreasuryState::Normal)
}

pub fn set_treasury_state(
    e: &Env,
    new_state: TreasuryState,
    caller: Address,
) -> Result<TreasuryState, StateError> {
    let current = get_treasury_state(e);
    let validated = current.transition(new_state)?;
    e.storage()
        .instance()
        .set(&StateDataKey::TreasuryState, &validated);
    emit_state_transition(
        e,
        symbol_short!("treasury"),
        current as u32,
        validated as u32,
        caller,
    );
    Ok(validated)
}

// ===========================================================================
// GOVERNANCE STORAGE & TRANSITIONS
// ===========================================================================

pub fn get_governance_state(e: &Env, proposal_id: Symbol) -> GovernanceState {
    e.storage()
        .persistent()
        .get(&StateDataKey::GovernanceState(proposal_id))
        .unwrap_or(GovernanceState::Draft)
}

pub fn set_governance_state(
    e: &Env,
    proposal_id: Symbol,
    new_state: GovernanceState,
    caller: Address,
) -> Result<GovernanceState, StateError> {
    let current = get_governance_state(e, proposal_id.clone());
    let validated = current.transition(new_state)?;
    e.storage()
        .persistent()
        .set(&StateDataKey::GovernanceState(proposal_id.clone()), &validated);
    emit_state_transition(
        e,
        symbol_short!("gov"),
        current as u32,
        validated as u32,
        caller,
    );
    Ok(validated)
}
