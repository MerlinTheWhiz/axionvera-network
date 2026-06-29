#![no_std]

use soroban_sdk::{contracterror, contracttype, symbol_short, Address, Env, Symbol};

/// Current event schema version for state transitions.
pub const STATE_EVENT_VERSION: u32 = 1;

/// Protocol identifier used as Topic 1 for all state transition events.
pub const PROTOCOL: Symbol = symbol_short!("AxVault");

/// Action symbol used as Topic 2 for state transition events.
pub const ACT_STATE_TRANSITION: Symbol = symbol_short!("state_trn");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum StateError {
    InvalidTransition = 1001,
    AlreadyInState = 1002,
    Unauthorized = 1003,
}

// ===========================================================================
// 1. VAULTS STATE MACHINE
// ===========================================================================

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VaultState {
    Uninitialized = 0,
    Active = 1,
    Paused = 2,
    Locked = 3,
    Terminated = 4,
}

impl VaultState {
    /// Validates and returns the next state or returns a StateError if invalid.
    pub fn transition(&self, next: VaultState) -> Result<VaultState, StateError> {
        if *self == next {
            return Err(StateError::AlreadyInState);
        }
        match (*self, next) {
            (VaultState::Uninitialized, VaultState::Active) => Ok(next),
            (VaultState::Active, VaultState::Paused) => Ok(next),
            (VaultState::Active, VaultState::Locked) => Ok(next),
            (VaultState::Active, VaultState::Terminated) => Ok(next),
            (VaultState::Paused, VaultState::Active) => Ok(next),
            (VaultState::Paused, VaultState::Terminated) => Ok(next),
            (VaultState::Locked, VaultState::Active) => Ok(next),
            (VaultState::Locked, VaultState::Paused) => Ok(next),
            (VaultState::Locked, VaultState::Terminated) => Ok(next),
            _ => Err(StateError::InvalidTransition),
        }
    }
}

// ===========================================================================
// 2. STAKING STATE MACHINE
// ===========================================================================

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StakingState {
    Uninitialized = 0,
    Warmup = 1,
    Active = 2,
    Cooldown = 3,
    Unstaked = 4,
    Slashed = 5,
}

impl StakingState {
    pub fn transition(&self, next: StakingState) -> Result<StakingState, StateError> {
        if *self == next {
            return Err(StateError::AlreadyInState);
        }
        match (*self, next) {
            (StakingState::Uninitialized, StakingState::Warmup) => Ok(next),
            (StakingState::Warmup, StakingState::Active) => Ok(next),
            (StakingState::Warmup, StakingState::Unstaked) => Ok(next),
            (StakingState::Active, StakingState::Cooldown) => Ok(next),
            (StakingState::Active, StakingState::Slashed) => Ok(next),
            (StakingState::Cooldown, StakingState::Unstaked) => Ok(next),
            (StakingState::Cooldown, StakingState::Active) => Ok(next),
            (StakingState::Cooldown, StakingState::Slashed) => Ok(next),
            (StakingState::Unstaked, StakingState::Warmup) => Ok(next),
            _ => Err(StateError::InvalidTransition),
        }
    }
}

// ===========================================================================
// 3. REWARDS STATE MACHINE
// ===========================================================================

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RewardState {
    Idle = 0,
    Accruing = 1,
    ReadyForDistribution = 2,
    Distributing = 3,
    Paused = 4,
}

impl RewardState {
    pub fn transition(&self, next: RewardState) -> Result<RewardState, StateError> {
        if *self == next {
            return Err(StateError::AlreadyInState);
        }
        match (*self, next) {
            (RewardState::Idle, RewardState::Accruing) => Ok(next),
            (RewardState::Accruing, RewardState::ReadyForDistribution) => Ok(next),
            (RewardState::Accruing, RewardState::Paused) => Ok(next),
            (RewardState::ReadyForDistribution, RewardState::Distributing) => Ok(next),
            (RewardState::ReadyForDistribution, RewardState::Paused) => Ok(next),
            (RewardState::Distributing, RewardState::Idle) => Ok(next),
            (RewardState::Distributing, RewardState::Paused) => Ok(next),
            (RewardState::Paused, RewardState::Accruing) => Ok(next),
            (RewardState::Paused, RewardState::ReadyForDistribution) => Ok(next),
            (RewardState::Paused, RewardState::Distributing) => Ok(next),
            _ => Err(StateError::InvalidTransition),
        }
    }
}

// ===========================================================================
// 4. TREASURY STATE MACHINE
// ===========================================================================

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TreasuryState {
    Normal = 0,
    UnderReview = 1,
    Rebalancing = 2,
    EmergencyRestricted = 3,
    Insolvent = 4,
}

impl TreasuryState {
    pub fn transition(&self, next: TreasuryState) -> Result<TreasuryState, StateError> {
        if *self == next {
            return Err(StateError::AlreadyInState);
        }
        match (*self, next) {
            (TreasuryState::Normal, TreasuryState::UnderReview) => Ok(next),
            (TreasuryState::Normal, TreasuryState::Rebalancing) => Ok(next),
            (TreasuryState::Normal, TreasuryState::EmergencyRestricted) => Ok(next),
            (TreasuryState::UnderReview, TreasuryState::Normal) => Ok(next),
            (TreasuryState::UnderReview, TreasuryState::EmergencyRestricted) => Ok(next),
            (TreasuryState::Rebalancing, TreasuryState::Normal) => Ok(next),
            (TreasuryState::Rebalancing, TreasuryState::EmergencyRestricted) => Ok(next),
            (TreasuryState::EmergencyRestricted, TreasuryState::Normal) => Ok(next),
            (TreasuryState::EmergencyRestricted, TreasuryState::Insolvent) => Ok(next),
            _ => Err(StateError::InvalidTransition),
        }
    }
}

// ===========================================================================
// 5. GOVERNANCE STATE MACHINE
// ===========================================================================

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GovernanceState {
    Draft = 0,
    Active = 1,
    Defeated = 2,
    Succeeded = 3,
    Queued = 4,
    Executed = 5,
    Canceled = 6,
    Expired = 7,
}

impl GovernanceState {
    pub fn transition(&self, next: GovernanceState) -> Result<GovernanceState, StateError> {
        if *self == next {
            return Err(StateError::AlreadyInState);
        }
        match (*self, next) {
            (GovernanceState::Draft, GovernanceState::Active) => Ok(next),
            (GovernanceState::Draft, GovernanceState::Canceled) => Ok(next),
            (GovernanceState::Active, GovernanceState::Defeated) => Ok(next),
            (GovernanceState::Active, GovernanceState::Succeeded) => Ok(next),
            (GovernanceState::Active, GovernanceState::Canceled) => Ok(next),
            (GovernanceState::Succeeded, GovernanceState::Queued) => Ok(next),
            (GovernanceState::Succeeded, GovernanceState::Expired) => Ok(next),
            (GovernanceState::Queued, GovernanceState::Executed) => Ok(next),
            (GovernanceState::Queued, GovernanceState::Canceled) => Ok(next),
            (GovernanceState::Queued, GovernanceState::Expired) => Ok(next),
            _ => Err(StateError::InvalidTransition),
        }
    }
}

// ===========================================================================
// TRANSITION EVENTS & EMITTERS
// ===========================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateTransitionEvent {
    pub event_version: u32,
    pub module: Symbol,
    pub old_state: u32,
    pub new_state: u32,
    pub caller: Address,
    pub timestamp: u64,
}

pub fn emit_state_transition(
    e: &Env,
    module: Symbol,
    old_state: u32,
    new_state: u32,
    caller: Address,
) {
    let event = StateTransitionEvent {
        event_version: STATE_EVENT_VERSION,
        module,
        old_state,
        new_state,
        caller,
        timestamp: e.ledger().timestamp(),
    };
    e.events().publish((PROTOCOL, ACT_STATE_TRANSITION), event);
}
