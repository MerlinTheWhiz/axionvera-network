#![no_std]

use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol};

use axionvera_state::{
    self as state, GovernanceState, RewardState, StakingState, StateError, TreasuryState,
    VaultState,
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    VaultState,
    StakingState,
    RewardState,
    TreasuryState,
    GovernanceState(Symbol),
    ProtocolAdmin,
}

const INSTANCE_TTL_THRESHOLD: u32 = 518_400;
const INSTANCE_TTL_EXTEND_TO: u32 = 518_400;

pub fn initialize_admin(e: &Env, admin: &Address) -> Result<(), StateError> {
    if e.storage().instance().has(&DataKey::ProtocolAdmin) {
        return Err(StateError::AlreadyInState);
    }
    admin.require_auth();
    e.storage().instance().set(&DataKey::ProtocolAdmin, admin);
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
    Ok(())
}

fn require_admin(e: &Env, caller: &Address) -> Result<(), StateError> {
    caller.require_auth();
    match e
        .storage()
        .instance()
        .get::<_, Address>(&DataKey::ProtocolAdmin)
    {
        Some(admin) if admin == *caller => Ok(()),
        None => Ok(()),
        _ => Err(StateError::Unauthorized),
    }
}

macro_rules! transition_store {
    ($fn_set:ident, $fn_get:ident, $key:expr, $default:expr, $module:expr, $ty:ty) => {
        pub fn $fn_get(e: &Env) -> $ty {
            e.storage().instance().get(&$key).unwrap_or($default)
        }

        pub fn $fn_set(e: &Env, new_state: $ty, caller: Address) -> Result<$ty, StateError> {
            require_admin(e, &caller)?;
            let current = $fn_get(e);
            let next = current.transition(new_state)?;
            e.storage().instance().set(&$key, &next);
            e.storage()
                .instance()
                .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
            state::emit_state_transition(e, $module, current as u32, next as u32, caller);
            Ok(next)
        }
    };
}

transition_store!(
    set_vault_state,
    get_vault_state,
    DataKey::VaultState,
    VaultState::Uninitialized,
    symbol_short!("vault"),
    VaultState
);
transition_store!(
    set_staking_state,
    get_staking_state,
    DataKey::StakingState,
    StakingState::Uninitialized,
    symbol_short!("staking"),
    StakingState
);
transition_store!(
    set_reward_state,
    get_reward_state,
    DataKey::RewardState,
    RewardState::Idle,
    symbol_short!("rewards"),
    RewardState
);
transition_store!(
    set_treasury_state,
    get_treasury_state,
    DataKey::TreasuryState,
    TreasuryState::Normal,
    symbol_short!("treasury"),
    TreasuryState
);

pub fn get_governance_state(e: &Env, proposal_id: Symbol) -> GovernanceState {
    e.storage()
        .instance()
        .get(&DataKey::GovernanceState(proposal_id))
        .unwrap_or(GovernanceState::Draft)
}

pub fn set_governance_state(
    e: &Env,
    proposal_id: Symbol,
    new_state: GovernanceState,
    caller: Address,
) -> Result<GovernanceState, StateError> {
    require_admin(e, &caller)?;
    let current = get_governance_state(e, proposal_id.clone());
    let next = current.transition(new_state)?;
    e.storage()
        .instance()
        .set(&DataKey::GovernanceState(proposal_id), &next);
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
    state::emit_state_transition(
        e,
        symbol_short!("govern"),
        current as u32,
        next as u32,
        caller,
    );
    Ok(next)
}
