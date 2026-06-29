#![no_std]

use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol};

use axionvera_interfaces::{FeeConfig, FeeTotals, FeeType};
use axionvera_state::{
    emit_state_transition, GovernanceState, RewardState, StateError, StakingState, TreasuryState,
    VaultState,
};

/// Current storage schema version.
pub const CURRENT_VERSION: u32 = 1;

/// Returns whether a persisted storage version is supported by this build.
pub const fn is_compatible(version: u32) -> bool {
    version == CURRENT_VERSION
}

const INSTANCE_TTL_THRESHOLD: u32 = 518_400;
const INSTANCE_TTL_EXTEND_TO: u32 = 518_400;
const PERSISTENT_TTL_THRESHOLD: u32 = 518_400;
const PERSISTENT_TTL_EXTEND_TO: u32 = 518_400;

/// Keys used by the shared storage layer.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Version,
    VaultState,
    StakingState,
    RewardState,
    TreasuryState,
    GovernanceState(Symbol),
    FeeConfig,
    FeeTotals(FeeType),
}

fn bump_instance_ttl(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
}

fn bump_persistent_ttl(e: &Env, key: &DataKey) {
    e.storage()
        .persistent()
        .extend_ttl(key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

fn current_or_default<T: Clone>(value: Option<T>, default: T) -> T {
    value.unwrap_or(default)
}

pub fn get_vault_state(e: &Env) -> VaultState {
    let state = current_or_default(
        e.storage()
            .instance()
            .get::<_, VaultState>(&DataKey::VaultState),
        VaultState::Uninitialized,
    );
    bump_instance_ttl(e);
    state
}

pub fn set_vault_state(
    e: &Env,
    new_state: VaultState,
    caller: Address,
) -> Result<VaultState, StateError> {
    caller.require_auth();
    let current = get_vault_state(e);
    let next = current.transition(new_state)?;
    e.storage().instance().set(&DataKey::VaultState, &next);
    bump_instance_ttl(e);
    emit_state_transition(e, symbol_short!("vault"), current as u32, next as u32, caller);
    Ok(next)
}

pub fn get_staking_state(e: &Env) -> StakingState {
    let state = current_or_default(
        e.storage()
            .instance()
            .get::<_, StakingState>(&DataKey::StakingState),
        StakingState::Uninitialized,
    );
    bump_instance_ttl(e);
    state
}

pub fn set_staking_state(
    e: &Env,
    new_state: StakingState,
    caller: Address,
) -> Result<StakingState, StateError> {
    caller.require_auth();
    let current = get_staking_state(e);
    let next = current.transition(new_state)?;
    e.storage().instance().set(&DataKey::StakingState, &next);
    bump_instance_ttl(e);
    emit_state_transition(e, symbol_short!("stake"), current as u32, next as u32, caller);
    Ok(next)
}

pub fn get_reward_state(e: &Env) -> RewardState {
    let state = current_or_default(
        e.storage()
            .instance()
            .get::<_, RewardState>(&DataKey::RewardState),
        RewardState::Idle,
    );
    bump_instance_ttl(e);
    state
}

pub fn set_reward_state(
    e: &Env,
    new_state: RewardState,
    caller: Address,
) -> Result<RewardState, StateError> {
    caller.require_auth();
    let current = get_reward_state(e);
    let next = current.transition(new_state)?;
    e.storage().instance().set(&DataKey::RewardState, &next);
    bump_instance_ttl(e);
    emit_state_transition(e, symbol_short!("rewar"), current as u32, next as u32, caller);
    Ok(next)
}

pub fn get_treasury_state(e: &Env) -> TreasuryState {
    let state = current_or_default(
        e.storage()
            .instance()
            .get::<_, TreasuryState>(&DataKey::TreasuryState),
        TreasuryState::Normal,
    );
    bump_instance_ttl(e);
    state
}

pub fn set_treasury_state(
    e: &Env,
    new_state: TreasuryState,
    caller: Address,
) -> Result<TreasuryState, StateError> {
    caller.require_auth();
    let current = get_treasury_state(e);
    let next = current.transition(new_state)?;
    e.storage().instance().set(&DataKey::TreasuryState, &next);
    bump_instance_ttl(e);
    emit_state_transition(e, symbol_short!("treas"), current as u32, next as u32, caller);
    Ok(next)
}

pub fn get_governance_state(e: &Env, proposal_id: Symbol) -> GovernanceState {
    let state = current_or_default(
        e.storage()
            .persistent()
            .get::<_, GovernanceState>(&DataKey::GovernanceState(proposal_id.clone())),
        GovernanceState::Draft,
    );
    bump_persistent_ttl(e, &DataKey::GovernanceState(proposal_id));
    state
}

pub fn set_governance_state(
    e: &Env,
    proposal_id: Symbol,
    new_state: GovernanceState,
    caller: Address,
) -> Result<GovernanceState, StateError> {
    caller.require_auth();
    let current = get_governance_state(e, proposal_id.clone());
    let next = current.transition(new_state)?;
    let key = DataKey::GovernanceState(proposal_id);
    e.storage().persistent().set(&key, &next);
    bump_persistent_ttl(e, &key);
    emit_state_transition(e, symbol_short!("govern"), current as u32, next as u32, caller);
    Ok(next)
}

pub fn get_fee_config(e: &Env) -> Option<FeeConfig> {
    let config = e.storage().instance().get(&DataKey::FeeConfig);
    if config.is_some() {
        bump_instance_ttl(e);
    }
    config
}

pub fn set_fee_config(e: &Env, config: &FeeConfig) {
    e.storage().instance().set(&DataKey::FeeConfig, config);
    bump_instance_ttl(e);
}

pub fn clear_fee_config(e: &Env) {
    e.storage().instance().remove(&DataKey::FeeConfig);
    bump_instance_ttl(e);
}

pub fn get_fee_totals(e: &Env, fee_type: FeeType) -> FeeTotals {
    let key = DataKey::FeeTotals(fee_type);
    let totals = e
        .storage()
        .persistent()
        .get::<_, FeeTotals>(&key)
        .unwrap_or(FeeTotals {
            operation_count: 0,
            collected_amount: 0,
            treasury_amount: 0,
        });
    if totals.operation_count > 0 {
        bump_persistent_ttl(e, &key);
    }
    totals
}

pub fn record_fee_totals(
    e: &Env,
    fee_type: FeeType,
    collected_amount: i128,
    treasury_amount: i128,
) -> Result<FeeTotals, StateError> {
    let key = DataKey::FeeTotals(fee_type);
    let mut totals = get_fee_totals(e, fee_type);
    totals.operation_count = totals
        .operation_count
        .checked_add(1)
        .ok_or(StateError::InvalidTransition)?;
    totals.collected_amount = totals
        .collected_amount
        .checked_add(collected_amount)
        .ok_or(StateError::InvalidTransition)?;
    totals.treasury_amount = totals
        .treasury_amount
        .checked_add(treasury_amount)
        .ok_or(StateError::InvalidTransition)?;
    e.storage().persistent().set(&key, &totals);
    bump_persistent_ttl(e, &key);
    Ok(totals)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_helpers_work() {
        assert_eq!(CURRENT_VERSION, 1);
        assert!(is_compatible(1));
        assert!(!is_compatible(2));
    }
}
