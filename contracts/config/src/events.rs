use soroban_sdk::{Address, Env};

use axionvera_events::{
    self, ConfigAdminTransferAcceptedEvent, ConfigAdminTransferProposedEvent,
    ConfigInitializedEvent, ConfigPausedEvent, ConfigUnpausedEvent,
    MaxAssetsUpdatedEvent, MaxUnlockLimitUpdatedEvent,
    MinRewardDistributionUpdatedEvent, PenaltyRateUpdatedEvent,
    TargetDepositsUpdatedEvent, VestingPeriodUpdatedEvent,
    WithdrawUnlockLimitUpdatedEvent, EVENT_VERSION, PROTOCOL_CONFIG,
    ACT_CFG_ADM_A, ACT_CFG_ADM_P, ACT_CFG_INIT, ACT_CFG_MA_UPD,
    ACT_CFG_MR_UPD, ACT_CFG_MU_UPD, ACT_CFG_PAUSE, ACT_CFG_PR_UPD,
    ACT_CFG_TD_UPD, ACT_CFG_UNPAU, ACT_CFG_VP_UPD, ACT_CFG_WU_UPD,
};

use crate::types::ProtocolConfig;

pub fn emit_initialized(e: &Env, admin: Address, config: ProtocolConfig) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL_CONFIG, ACT_CFG_INIT),
        ConfigInitializedEvent {
            event_version: EVENT_VERSION,
            admin,
            penalty_rate_bps: config.penalty_rate_bps,
            vesting_period: config.vesting_period,
            target_deposits: config.target_deposits,
            min_reward_distribution: config.min_reward_distribution,
            max_unlock_limit: config.max_unlock_limit,
            withdraw_unlock_limit: config.withdraw_unlock_limit,
            max_assets: config.max_assets,
            timestamp: ts,
        },
    );
}

pub fn emit_penalty_rate_updated(e: &Env, admin: Address, old: u32, new: u32) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL_CONFIG, ACT_CFG_PR_UPD),
        PenaltyRateUpdatedEvent {
            event_version: EVENT_VERSION,
            admin,
            old_rate_bps: old,
            new_rate_bps: new,
            timestamp: ts,
        },
    );
}

pub fn emit_vesting_period_updated(e: &Env, admin: Address, old: u64, new: u64) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL_CONFIG, ACT_CFG_VP_UPD),
        VestingPeriodUpdatedEvent {
            event_version: EVENT_VERSION,
            admin,
            old_period: old,
            new_period: new,
            timestamp: ts,
        },
    );
}

pub fn emit_target_deposits_updated(e: &Env, admin: Address, old: i128, new: i128) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL_CONFIG, ACT_CFG_TD_UPD),
        TargetDepositsUpdatedEvent {
            event_version: EVENT_VERSION,
            admin,
            old_amount: old,
            new_amount: new,
            timestamp: ts,
        },
    );
}

pub fn emit_min_reward_distribution_updated(e: &Env, admin: Address, old: i128, new: i128) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL_CONFIG, ACT_CFG_MR_UPD),
        MinRewardDistributionUpdatedEvent {
            event_version: EVENT_VERSION,
            admin,
            old_amount: old,
            new_amount: new,
            timestamp: ts,
        },
    );
}

pub fn emit_max_unlock_limit_updated(e: &Env, admin: Address, old: u32, new: u32) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL_CONFIG, ACT_CFG_MU_UPD),
        MaxUnlockLimitUpdatedEvent {
            event_version: EVENT_VERSION,
            admin,
            old_limit: old,
            new_limit: new,
            timestamp: ts,
        },
    );
}

pub fn emit_withdraw_unlock_limit_updated(e: &Env, admin: Address, old: u32, new: u32) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL_CONFIG, ACT_CFG_WU_UPD),
        WithdrawUnlockLimitUpdatedEvent {
            event_version: EVENT_VERSION,
            admin,
            old_limit: old,
            new_limit: new,
            timestamp: ts,
        },
    );
}

pub fn emit_max_assets_updated(e: &Env, admin: Address, old: u32, new: u32) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL_CONFIG, ACT_CFG_MA_UPD),
        MaxAssetsUpdatedEvent {
            event_version: EVENT_VERSION,
            admin,
            old_max: old,
            new_max: new,
            timestamp: ts,
        },
    );
}

pub fn emit_admin_transfer_proposed(e: &Env, current_admin: Address, pending_admin: Address) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL_CONFIG, ACT_CFG_ADM_P),
        ConfigAdminTransferProposedEvent {
            event_version: EVENT_VERSION,
            current_admin,
            pending_admin,
            timestamp: ts,
        },
    );
}

pub fn emit_admin_transfer_accepted(e: &Env, previous_admin: Address, new_admin: Address) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL_CONFIG, ACT_CFG_ADM_A),
        ConfigAdminTransferAcceptedEvent {
            event_version: EVENT_VERSION,
            previous_admin,
            new_admin,
            timestamp: ts,
        },
    );
}

pub fn emit_paused(e: &Env, admin: Address) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL_CONFIG, ACT_CFG_PAUSE),
        ConfigPausedEvent {
            event_version: EVENT_VERSION,
            admin,
            timestamp: ts,
        },
    );
}

pub fn emit_unpaused(e: &Env, admin: Address) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL_CONFIG, ACT_CFG_UNPAU),
        ConfigUnpausedEvent {
            event_version: EVENT_VERSION,
            admin,
            timestamp: ts,
        },
    );
}
