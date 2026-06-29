#![no_std]

use soroban_sdk::{contracttype, Address, Env, Map, Symbol, Vec};

use axionvera_events as events;
use axionvera_state::{
    GovernanceState, RewardState, StateError, StakingState, TreasuryState, VaultState,
};
use axionvera_storage::{
    get_governance_state, get_reward_state, get_staking_state, get_treasury_state, get_vault_state,
    set_governance_state, set_reward_state, set_staking_state, set_treasury_state, set_vault_state,
};
use axionvera_snapshots as snapshots;

/// Maximum number of event log entries stored per user index.
const MAX_EVENTS_PER_USER: u32 = 50;

/// Maximum number of event log entries stored globally.
const MAX_GLOBAL_EVENTS: u32 = 200;

/// A lightweight on-chain event log entry for indexing.
/// Stores only key metadata; the full event is emitted via Soroban topics.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventLogEntry {
    pub action: soroban_sdk::Symbol,
    pub user: Option<Address>,
    pub asset: Option<Address>,
    pub amount: i128,
    pub timestamp: u64,
    pub ledger: u32,
}

/// Index a newly emitted event by appending to the global event log.
pub fn index_event(
    e: &Env,
    action: soroban_sdk::Symbol,
    user: Option<Address>,
    asset: Option<Address>,
    amount: i128,
) {
    let entry = EventLogEntry {
        action,
        user: user.clone(),
        asset,
        amount,
        timestamp: e.ledger().timestamp(),
        ledger: e.ledger().sequence(),
    };

    // Append to the global event log
    let mut global_log: Vec<EventLogEntry> = e
        .storage()
        .instance()
        .get(&events::DataKey::EventLog)
        .unwrap_or_else(|| Vec::new(e));
    global_log.push_back(entry.clone());
    // Trim oldest entries when the log exceeds capacity
    while global_log.len() > MAX_GLOBAL_EVENTS {
        _ = global_log.remove(0);
    }
    e.storage()
        .instance()
        .set(&events::DataKey::EventLog, &global_log);

    // Append to the per-user event log
    if let Some(user_addr) = user {
        let mut user_log: Vec<EventLogEntry> = e
            .storage()
            .persistent()
            .get(&events::DataKey::UserEventLog(user_addr.clone()))
            .unwrap_or_else(|| Vec::new(e));
        user_log.push_back(entry);
        while user_log.len() > MAX_EVENTS_PER_USER {
            _ = user_log.remove(0);
        }
        e.storage()
            .persistent()
            .set(&events::DataKey::UserEventLog(user_addr), &user_log);
    }

    e.storage().instance().extend_ttl(518400, 518400);
}

/// Retrieve the global event log.
pub fn get_global_event_log(e: &Env) -> Vec<EventLogEntry> {
    e.storage()
        .instance()
        .get(&events::DataKey::EventLog)
        .unwrap_or_else(|| Vec::new(e))
}

/// Retrieve the event log for a specific user.
pub fn get_user_event_log(e: &Env, user: &Address) -> Vec<EventLogEntry> {
    e.storage()
        .persistent()
        .get(&events::DataKey::UserEventLog(user.clone()))
        .unwrap_or_else(|| Vec::new(e))
}

/// Maintain a set of unique users who have interacted with the contract.
pub fn record_interacting_user(e: &Env, user: &Address) {
    let mut users: Map<Address, bool> = e
        .storage()
        .instance()
        .get(&events::DataKey::InteractingUsers)
        .unwrap_or_else(|| Map::new(e));
    if !users.contains_key(user.clone()) {
        users.set(user.clone(), true);
        e.storage()
            .instance()
            .set(&events::DataKey::InteractingUsers, &users);
    }
}

/// Retrieve the set of all users who have interacted with the contract.
pub fn get_interacting_users(e: &Env) -> Vec<Address> {
    let users: Map<Address, bool> = e
        .storage()
        .instance()
        .get(&events::DataKey::InteractingUsers)
        .unwrap_or_else(|| Map::new(e));
    users.keys()
}

// ===========================================================================
// STATE MACHINE CORE INTEGRATION FACADE
// ===========================================================================

/// Transition Vault state
pub fn transition_vault_state(
    e: &Env,
    new_state: VaultState,
    caller: Address,
) -> Result<VaultState, StateError> {
    set_vault_state(e, new_state, caller)
}

/// Get current Vault state
pub fn current_vault_state(e: &Env) -> VaultState {
    get_vault_state(e)
}

/// Transition Staking state
pub fn transition_staking_state(
    e: &Env,
    new_state: StakingState,
    caller: Address,
) -> Result<StakingState, StateError> {
    set_staking_state(e, new_state, caller)
}

/// Get current Staking state
pub fn current_staking_state(e: &Env) -> StakingState {
    get_staking_state(e)
}

/// Transition Reward state
pub fn transition_reward_state(
    e: &Env,
    new_state: RewardState,
    caller: Address,
) -> Result<RewardState, StateError> {
    set_reward_state(e, new_state, caller)
}

/// Get current Reward state
pub fn current_reward_state(e: &Env) -> RewardState {
    get_reward_state(e)
}

/// Transition Treasury state
pub fn transition_treasury_state(
    e: &Env,
    new_state: TreasuryState,
    caller: Address,
) -> Result<TreasuryState, StateError> {
    set_treasury_state(e, new_state, caller)
}

/// Get current Treasury state
pub fn current_treasury_state(e: &Env) -> TreasuryState {
    get_treasury_state(e)
}

/// Transition Governance state
pub fn transition_governance_state(
    e: &Env,
    proposal_id: Symbol,
    new_state: GovernanceState,
    caller: Address,
) -> Result<GovernanceState, StateError> {
    set_governance_state(e, proposal_id, new_state, caller)
}

/// Get current Governance state
pub fn current_governance_state(e: &Env, proposal_id: Symbol) -> GovernanceState {
    get_governance_state(e, proposal_id)
}

// ===========================================================================
// SNAPSHOT INTEGRATION
// ===========================================================================

/// Take a protocol snapshot.
pub fn take_snapshot(e: &Env) -> Result<snapshots::ProtocolSnapshot, snapshots::SnapshotError> {
    snapshots::take_snapshot(e, None)
}

/// Get a protocol snapshot by ID.
pub fn get_snapshot(e: &Env, id: u64) -> Option<snapshots::ProtocolSnapshot> {
    snapshots::get_snapshot(e, id)
}

/// Get the latest protocol snapshot.
pub fn get_latest_snapshot(e: &Env) -> Option<snapshots::ProtocolSnapshot> {
    snapshots::get_latest_snapshot(e)
}

/// Get the protocol snapshot history.
pub fn get_snapshot_history(e: &Env, limit: u32) -> Vec<snapshots::ProtocolSnapshot> {
    snapshots::get_snapshot_history(e, limit)
}
