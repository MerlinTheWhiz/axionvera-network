use soroban_sdk::contracterror;

/// All errors that can be returned by the config contract.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ConfigError {
    /// `initialize` was called on an already-initialised contract.
    AlreadyInitialized = 1,
    /// A state-reading function was called before `initialize`.
    NotInitialized = 2,
    /// Caller does not have admin authority.
    Unauthorized = 3,
    /// `penalty_rate_bps` exceeds 10 000 (100 %).
    InvalidPenaltyRate = 4,
    /// `vesting_period` exceeds the one-year ceiling.
    InvalidVestingPeriod = 5,
    /// `target_deposits` is zero or negative.
    InvalidTargetDeposits = 6,
    /// `min_reward_distribution` is zero or negative.
    InvalidMinRewardDistribution = 7,
    /// `max_unlock_limit` is outside [1, 100].
    InvalidMaxUnlockLimit = 8,
    /// `withdraw_unlock_limit` is outside [1, 50].
    InvalidWithdrawUnlockLimit = 9,
    /// `max_assets` is outside [1, 50].
    InvalidMaxAssets = 10,
    /// `accept_admin` called when no transfer is pending.
    NoPendingAdmin = 11,
    /// An admin-write function was called while the contract is paused.
    ContractPaused = 12,
}
