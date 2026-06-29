#![no_std]

use soroban_sdk::{contracttype, Env, Vec, Symbol};
use axionvera_state::{VaultState, StakingState, RewardState, TreasuryState};
use axionvera_accounting::{self, ResourceTotals};
use axionvera_storage::{get_vault_state, get_staking_state, get_reward_state, get_treasury_state};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotMetadata {
    pub id: u64,
    pub timestamp: u64,
    pub ledger: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolSnapshot {
    pub metadata: SnapshotMetadata,
    pub vault_state: VaultState,
    pub staking_state: StakingState,
    pub reward_state: RewardState,
    pub treasury_state: TreasuryState,
    pub total_usage: ResourceTotals,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    SnapshotCount,
    Snapshot(u64),
    LatestSnapshotId,
    LastSnapshotTimestamp,
}

pub const MIN_SNAPSHOT_INTERVAL: u64 = 3600; // 1 hour

pub fn take_snapshot(e: &Env, _proposal_id_for_gov: Option<Symbol>) -> Result<ProtocolSnapshot, SnapshotError> {
    let now = e.ledger().timestamp();
    let last_timestamp = e.storage().instance().get(&DataKey::LastSnapshotTimestamp).unwrap_or(0);

    if now < last_timestamp + MIN_SNAPSHOT_INTERVAL {
        return Err(SnapshotError::IntervalNotMet);
    }

    let mut id = e.storage().instance().get(&DataKey::SnapshotCount).unwrap_or(0u64);
    id += 1;

    let metadata = SnapshotMetadata {
        id,
        timestamp: now,
        ledger: e.ledger().sequence(),
    };

    let snapshot = ProtocolSnapshot {
        metadata: metadata.clone(),
        vault_state: get_vault_state(e),
        staking_state: get_staking_state(e),
        reward_state: get_reward_state(e),
        treasury_state: get_treasury_state(e),
        total_usage: axionvera_accounting::get_total_usage(e),
    };

    e.storage().persistent().set(&DataKey::Snapshot(id), &snapshot);
    e.storage().instance().set(&DataKey::SnapshotCount, &id);
    e.storage().instance().set(&DataKey::LatestSnapshotId, &id);
    e.storage().instance().set(&DataKey::LastSnapshotTimestamp, &now);

    // Extend TTL for persistent storage
    e.storage().persistent().extend_ttl(&DataKey::Snapshot(id), 518400, 518400);

    Ok(snapshot)
}

pub fn get_snapshot(e: &Env, id: u64) -> Option<ProtocolSnapshot> {
    e.storage().persistent().get(&DataKey::Snapshot(id))
}

pub fn get_latest_snapshot(e: &Env) -> Option<ProtocolSnapshot> {
    let id = e.storage().instance().get(&DataKey::LatestSnapshotId).unwrap_or(0);
    if id == 0 {
        None
    } else {
        get_snapshot(e, id)
    }
}

pub fn get_snapshot_history(e: &Env, limit: u32) -> Vec<ProtocolSnapshot> {
    let count = e.storage().instance().get(&DataKey::SnapshotCount).unwrap_or(0);
    let mut history = Vec::new(e);
    let start = if count > limit as u64 { count - limit as u64 + 1 } else { 1 };

    for i in (start..=count).rev() {
        if let Some(snapshot) = get_snapshot(e, i) {
            history.push_back(snapshot);
        }
    }
    history
}

#[soroban_sdk::contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SnapshotError {
    IntervalNotMet = 1,
}
mod test;
