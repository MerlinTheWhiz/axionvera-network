use soroban_sdk::contracttype;

// ---------------------------------------------------------------------------
// Validation bounds — enforced on every write, documented for integrators
// ---------------------------------------------------------------------------

/// Maximum penalty rate in basis points (100% = 10 000).
pub const MAX_PENALTY_RATE_BPS: u32 = 10_000;

/// Maximum vesting period in seconds (capped at one year).
pub const MAX_VESTING_PERIOD: u64 = 31_536_000;

/// Target deposit amount must be at least 1 stroop.
pub const MIN_TARGET_DEPOSITS: i128 = 1;

/// Minimum reward distribution must be at least 1 stroop.
pub const MIN_REWARD_DISTRIBUTION_FLOOR: i128 = 1;

/// Upper ceiling on the per-call unlock limit (prevents budget exhaustion).
pub const MAX_UNLOCK_LIMIT_CEILING: u32 = 100;

/// Lower bound on the per-call unlock limit.
pub const MIN_UNLOCK_LIMIT: u32 = 1;

/// Upper ceiling on the auto-unlock count per withdraw call.
pub const MAX_WITHDRAW_UNLOCK_LIMIT: u32 = 50;

/// Lower bound on the auto-unlock count per withdraw call.
pub const MIN_WITHDRAW_UNLOCK_LIMIT: u32 = 1;

/// Maximum number of supported assets.
pub const MAX_ASSETS_CEILING: u32 = 50;

/// Minimum number of supported assets.
pub const MIN_ASSETS: u32 = 1;

// ---------------------------------------------------------------------------
// ProtocolConfig — the single source of truth for all protocol parameters
// ---------------------------------------------------------------------------

/// All configurable protocol parameters, stored as a single on-chain record.
///
/// Every field has explicit validation bounds defined above. Callers should
/// retrieve this struct via `get_config()` and pass it to `initialize()`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolConfig {
    /// Early-withdrawal penalty charged against locked funds (0–10 000 bps).
    pub penalty_rate_bps: u32,

    /// Seconds after reward accrual before rewards become claimable (0 = immediate).
    pub vesting_period: u64,

    /// Target total vault TVL used in utilization-curve calculations (≥ 1 stroop).
    pub target_deposits: i128,

    /// Minimum reward batch that may be distributed in a single call (≥ 1 stroop).
    pub min_reward_distribution: i128,

    /// Maximum number of expired locks to process in a single `unlock_expired` call.
    pub max_unlock_limit: u32,

    /// Number of expired locks automatically processed during `withdraw`.
    pub withdraw_unlock_limit: u32,

    /// Maximum number of distinct assets the vault may support.
    pub max_assets: u32,
}
