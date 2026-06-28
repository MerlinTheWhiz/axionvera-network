#![no_std]

pub mod errors;
mod events;
mod storage;
pub mod types;
#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, Env};

use crate::errors::ConfigError;
use crate::types::{
    ProtocolConfig, MAX_ASSETS_CEILING, MAX_PENALTY_RATE_BPS, MAX_UNLOCK_LIMIT_CEILING,
    MAX_VESTING_PERIOD, MAX_WITHDRAW_UNLOCK_LIMIT, MIN_ASSETS, MIN_REWARD_DISTRIBUTION_FLOOR,
    MIN_TARGET_DEPOSITS, MIN_UNLOCK_LIMIT, MIN_WITHDRAW_UNLOCK_LIMIT,
};

#[contract]
pub struct ConfigContract;

#[contractimpl]
impl ConfigContract {
    /// Returns the contract version.
    pub fn version() -> u32 {
        1
    }

    // -----------------------------------------------------------------------
    // Lifecycle
    // -----------------------------------------------------------------------

    /// One-time initializer. Sets the admin and stores the initial config.
    ///
    /// Requires `admin` authorization. All config fields are validated before
    /// storage. Emits `ConfigInitializedEvent`.
    pub fn initialize(
        e: Env,
        admin: Address,
        config: ProtocolConfig,
    ) -> Result<(), ConfigError> {
        if storage::is_initialized(&e) {
            return Err(ConfigError::AlreadyInitialized);
        }
        admin.require_auth();
        validate_config(&config)?;
        storage::initialize(&e, &admin, &config);
        events::emit_initialized(&e, admin, config);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Read
    // -----------------------------------------------------------------------

    /// Returns the full `ProtocolConfig` record.
    pub fn get_config(e: Env) -> Result<ProtocolConfig, ConfigError> {
        storage::require_initialized(&e)?;
        storage::get_config(&e)
    }

    /// Returns the current admin address.
    pub fn admin(e: Env) -> Result<Address, ConfigError> {
        storage::require_initialized(&e)?;
        storage::get_admin(&e)
    }

    /// Returns the pending admin address, if a transfer is in progress.
    pub fn pending_admin(e: Env) -> Result<Option<Address>, ConfigError> {
        storage::require_initialized(&e)?;
        Ok(storage::get_pending_admin(&e))
    }

    /// Returns whether the contract is currently paused.
    pub fn is_paused(e: Env) -> bool {
        storage::get_is_paused(&e)
    }

    // -----------------------------------------------------------------------
    // Parameter setters (admin only, writes blocked when paused)
    // -----------------------------------------------------------------------

    /// Updates `penalty_rate_bps`. Valid range: 0–10 000.
    ///
    /// Emits `PenaltyRateUpdatedEvent` with old and new values.
    pub fn set_penalty_rate(e: Env, new_rate_bps: u32) -> Result<(), ConfigError> {
        storage::require_initialized(&e)?;
        storage::require_not_paused(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();
        if new_rate_bps > MAX_PENALTY_RATE_BPS {
            return Err(ConfigError::InvalidPenaltyRate);
        }
        let mut config = storage::get_config(&e)?;
        let old = config.penalty_rate_bps;
        config.penalty_rate_bps = new_rate_bps;
        storage::set_config(&e, &config);
        events::emit_penalty_rate_updated(&e, admin, old, new_rate_bps);
        Ok(())
    }

    /// Updates `vesting_period` in seconds. Valid range: 0–31 536 000 (1 year).
    ///
    /// Emits `VestingPeriodUpdatedEvent` with old and new values.
    pub fn set_vesting_period(e: Env, new_period: u64) -> Result<(), ConfigError> {
        storage::require_initialized(&e)?;
        storage::require_not_paused(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();
        if new_period > MAX_VESTING_PERIOD {
            return Err(ConfigError::InvalidVestingPeriod);
        }
        let mut config = storage::get_config(&e)?;
        let old = config.vesting_period;
        config.vesting_period = new_period;
        storage::set_config(&e, &config);
        events::emit_vesting_period_updated(&e, admin, old, new_period);
        Ok(())
    }

    /// Updates `target_deposits`. Must be ≥ 1 stroop.
    ///
    /// Emits `TargetDepositsUpdatedEvent` with old and new values.
    pub fn set_target_deposits(e: Env, new_amount: i128) -> Result<(), ConfigError> {
        storage::require_initialized(&e)?;
        storage::require_not_paused(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();
        if new_amount < MIN_TARGET_DEPOSITS {
            return Err(ConfigError::InvalidTargetDeposits);
        }
        let mut config = storage::get_config(&e)?;
        let old = config.target_deposits;
        config.target_deposits = new_amount;
        storage::set_config(&e, &config);
        events::emit_target_deposits_updated(&e, admin, old, new_amount);
        Ok(())
    }

    /// Updates `min_reward_distribution`. Must be ≥ 1 stroop.
    ///
    /// Emits `MinRewardDistributionUpdatedEvent` with old and new values.
    pub fn set_min_reward_distribution(e: Env, new_amount: i128) -> Result<(), ConfigError> {
        storage::require_initialized(&e)?;
        storage::require_not_paused(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();
        if new_amount < MIN_REWARD_DISTRIBUTION_FLOOR {
            return Err(ConfigError::InvalidMinRewardDistribution);
        }
        let mut config = storage::get_config(&e)?;
        let old = config.min_reward_distribution;
        config.min_reward_distribution = new_amount;
        storage::set_config(&e, &config);
        events::emit_min_reward_distribution_updated(&e, admin, old, new_amount);
        Ok(())
    }

    /// Updates `max_unlock_limit`. Valid range: 1–100.
    ///
    /// Emits `MaxUnlockLimitUpdatedEvent` with old and new values.
    pub fn set_max_unlock_limit(e: Env, new_limit: u32) -> Result<(), ConfigError> {
        storage::require_initialized(&e)?;
        storage::require_not_paused(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();
        if new_limit < MIN_UNLOCK_LIMIT || new_limit > MAX_UNLOCK_LIMIT_CEILING {
            return Err(ConfigError::InvalidMaxUnlockLimit);
        }
        let mut config = storage::get_config(&e)?;
        let old = config.max_unlock_limit;
        config.max_unlock_limit = new_limit;
        storage::set_config(&e, &config);
        events::emit_max_unlock_limit_updated(&e, admin, old, new_limit);
        Ok(())
    }

    /// Updates `withdraw_unlock_limit`. Valid range: 1–50.
    ///
    /// Emits `WithdrawUnlockLimitUpdatedEvent` with old and new values.
    pub fn set_withdraw_unlock_limit(e: Env, new_limit: u32) -> Result<(), ConfigError> {
        storage::require_initialized(&e)?;
        storage::require_not_paused(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();
        if new_limit < MIN_WITHDRAW_UNLOCK_LIMIT || new_limit > MAX_WITHDRAW_UNLOCK_LIMIT {
            return Err(ConfigError::InvalidWithdrawUnlockLimit);
        }
        let mut config = storage::get_config(&e)?;
        let old = config.withdraw_unlock_limit;
        config.withdraw_unlock_limit = new_limit;
        storage::set_config(&e, &config);
        events::emit_withdraw_unlock_limit_updated(&e, admin, old, new_limit);
        Ok(())
    }

    /// Updates `max_assets`. Valid range: 1–50.
    ///
    /// Emits `MaxAssetsUpdatedEvent` with old and new values.
    pub fn set_max_assets(e: Env, new_max: u32) -> Result<(), ConfigError> {
        storage::require_initialized(&e)?;
        storage::require_not_paused(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();
        if new_max < MIN_ASSETS || new_max > MAX_ASSETS_CEILING {
            return Err(ConfigError::InvalidMaxAssets);
        }
        let mut config = storage::get_config(&e)?;
        let old = config.max_assets;
        config.max_assets = new_max;
        storage::set_config(&e, &config);
        events::emit_max_assets_updated(&e, admin, old, new_max);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Admin transfer (two-step)
    // -----------------------------------------------------------------------

    /// Proposes a new admin. The transfer is not final until `accept_admin` is
    /// called by the proposed address. Emits `ConfigAdminTransferProposedEvent`.
    pub fn propose_new_admin(e: Env, new_admin: Address) -> Result<(), ConfigError> {
        storage::require_initialized(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();
        storage::set_pending_admin(&e, &new_admin);
        events::emit_admin_transfer_proposed(&e, admin, new_admin);
        Ok(())
    }

    /// Finalises an in-progress admin transfer. Must be called by the pending
    /// admin. Emits `ConfigAdminTransferAcceptedEvent`.
    pub fn accept_admin(e: Env, new_admin: Address) -> Result<(), ConfigError> {
        storage::require_initialized(&e)?;
        new_admin.require_auth();
        let previous_admin = storage::get_admin(&e)?;
        let pending = storage::get_pending_admin(&e).ok_or(ConfigError::NoPendingAdmin)?;
        if pending != new_admin {
            return Err(ConfigError::Unauthorized);
        }
        storage::set_admin(&e, &new_admin);
        storage::clear_pending_admin(&e);
        events::emit_admin_transfer_accepted(&e, previous_admin, new_admin);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Emergency controls
    // -----------------------------------------------------------------------

    /// Pauses all parameter-write operations. Reads remain available.
    /// Emits `ConfigPausedEvent`.
    pub fn pause_contract(e: Env) -> Result<(), ConfigError> {
        storage::require_initialized(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();
        storage::set_paused(&e, true);
        events::emit_paused(&e, admin);
        Ok(())
    }

    /// Resumes parameter-write operations after a pause.
    /// Emits `ConfigUnpausedEvent`.
    pub fn unpause_contract(e: Env) -> Result<(), ConfigError> {
        storage::require_initialized(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();
        storage::set_paused(&e, false);
        events::emit_unpaused(&e, admin);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Validation helpers
// ---------------------------------------------------------------------------

fn validate_config(config: &ProtocolConfig) -> Result<(), ConfigError> {
    if config.penalty_rate_bps > MAX_PENALTY_RATE_BPS {
        return Err(ConfigError::InvalidPenaltyRate);
    }
    if config.vesting_period > MAX_VESTING_PERIOD {
        return Err(ConfigError::InvalidVestingPeriod);
    }
    if config.target_deposits < MIN_TARGET_DEPOSITS {
        return Err(ConfigError::InvalidTargetDeposits);
    }
    if config.min_reward_distribution < MIN_REWARD_DISTRIBUTION_FLOOR {
        return Err(ConfigError::InvalidMinRewardDistribution);
    }
    if config.max_unlock_limit < MIN_UNLOCK_LIMIT || config.max_unlock_limit > MAX_UNLOCK_LIMIT_CEILING {
        return Err(ConfigError::InvalidMaxUnlockLimit);
    }
    if config.withdraw_unlock_limit < MIN_WITHDRAW_UNLOCK_LIMIT
        || config.withdraw_unlock_limit > MAX_WITHDRAW_UNLOCK_LIMIT
    {
        return Err(ConfigError::InvalidWithdrawUnlockLimit);
    }
    if config.max_assets < MIN_ASSETS || config.max_assets > MAX_ASSETS_CEILING {
        return Err(ConfigError::InvalidMaxAssets);
    }
    Ok(())
}
