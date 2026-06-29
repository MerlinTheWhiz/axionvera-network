#![no_std]

use soroban_sdk::{contracttype, Address, Env, Symbol};
use axionvera_state::{
    VaultState, StakingState, RewardState, TreasuryState, GovernanceState,
    StateError, emit_state_transition
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    VaultState,
    StakingState,
    RewardState,
    TreasuryState,
    GovernanceState(Symbol),
}

pub fn get_vault_state(e: &Env) -> VaultState {
    e.storage().instance().get(&DataKey::VaultState).unwrap_or(VaultState::Uninitialized)
}

pub fn set_vault_state(e: &Env, new_state: VaultState, caller: Address) -> Result<VaultState, StateError> {
    let current = get_vault_state(e);
    let next = current.transition(new_state)?;
    e.storage().instance().set(&DataKey::VaultState, &next);
    emit_state_transition(e, Symbol::new(e, "vault"), current as u32, next as u32, caller);
    Ok(next)
}

pub fn get_staking_state(e: &Env) -> StakingState {
    e.storage().instance().get(&DataKey::StakingState).unwrap_or(StakingState::Uninitialized)
}

pub fn set_staking_state(e: &Env, new_state: StakingState, caller: Address) -> Result<StakingState, StateError> {
    let current = get_staking_state(e);
    let next = current.transition(new_state)?;
    e.storage().instance().set(&DataKey::StakingState, &next);
    emit_state_transition(e, Symbol::new(e, "staking"), current as u32, next as u32, caller);
    Ok(next)
}

pub fn get_reward_state(e: &Env) -> RewardState {
    e.storage().instance().get(&DataKey::RewardState).unwrap_or(RewardState::Idle)
}

pub fn set_reward_state(e: &Env, new_state: RewardState, caller: Address) -> Result<RewardState, StateError> {
    let current = get_reward_state(e);
    let next = current.transition(new_state)?;
    e.storage().instance().set(&DataKey::RewardState, &next);
    emit_state_transition(e, Symbol::new(e, "reward"), current as u32, next as u32, caller);
    Ok(next)
}

pub fn get_treasury_state(e: &Env) -> TreasuryState {
    e.storage().instance().get(&DataKey::TreasuryState).unwrap_or(TreasuryState::Normal)
}

pub fn set_treasury_state(e: &Env, new_state: TreasuryState, caller: Address) -> Result<TreasuryState, StateError> {
    let current = get_treasury_state(e);
    let next = current.transition(new_state)?;
    e.storage().instance().set(&DataKey::TreasuryState, &next);
    emit_state_transition(e, Symbol::new(e, "treasury"), current as u32, next as u32, caller);
    Ok(next)
}

pub fn get_governance_state(e: &Env, proposal_id: Symbol) -> GovernanceState {
    e.storage().persistent().get(&DataKey::GovernanceState(proposal_id)).unwrap_or(GovernanceState::Draft)
}

pub fn set_governance_state(e: &Env, proposal_id: Symbol, new_state: GovernanceState, caller: Address) -> Result<GovernanceState, StateError> {
    let current = get_governance_state(e, proposal_id.clone());
    let next = current.transition(new_state)?;
    e.storage().persistent().set(&DataKey::GovernanceState(proposal_id.clone()), &next);
    emit_state_transition(e, Symbol::new(e, "governance"), current as u32, next as u32, caller);
    Ok(next)
}
