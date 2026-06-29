use soroban_sdk::{contracttype, Address, Env, Map, Vec};

use crate::errors::{
    ArithmeticError, AuthorizationError, BalanceError, StateError, ValidationError, VaultError,
    ArithmeticError, AuthorizationError, BalanceError, DelegationError, StateError, ValidationError,
    VaultError,
    ArithmeticError, AuthorizationError, BalanceError, DelegationError, StateError,
    ValidationError, VaultError,
};

pub const PRECISION_FACTOR: i128 = 1_000_000_000;
const REWARD_INDEX_SCALE: i128 = PRECISION_FACTOR;
const LOCK_WEIGHT_SCALE: i128 = 10_000;

const INSTANCE_TTL_THRESHOLD: u32 = 518_400;
const INSTANCE_TTL_EXTEND_TO: u32 = 518_400;

const PERSISTENT_TTL_THRESHOLD: u32 = 518_400;
const PERSISTENT_TTL_EXTEND_TO: u32 = 518_400;

/// A point on the utilization-to-reward-multiplier curve.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultiplierPoint {
    /// Utilization in basis points (0-10000).
    pub utilization_bps: u32,
    /// Reward multiplier in basis points (e.g., 10000 = 1.0x, 15000 = 1.5x).
    pub multiplier_bps: u32,
}

/// A configured lock duration and its reward multiplier.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LockDurationModel {
    /// Duration in seconds the funds must remain locked.
    pub duration_seconds: u64,
    /// Reward multiplier in basis points applied while the lock is active.
    pub reward_multiplier_bps: u32,
}

/// A time-locked deposit entry for a user.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Lock {
    /// The amount of tokens locked.
    pub amount: i128,
    /// The configured lock duration in seconds.
    pub duration_seconds: u64,
    /// The timestamp at which the lock expires.
    pub unlock_timestamp: u64,
    /// Reward multiplier in basis points for this specific lock.
    pub reward_multiplier_bps: u32,
}

/// Keys used to store data in the contract's storage.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Flag indicating if the contract has been initialized.
    Initialized,
    /// Admin address
    Admin,
    /// Pending admin address (for two-step transfer)
    PendingAdmin,
    /// Deposit token address (legacy, kept for backwards compatibility)
    DepositToken,
    /// Reward token address
    RewardToken,
    /// Total deposits amount (legacy, kept for backwards compatibility)
    TotalDeposits,
    /// Global reward index (legacy, kept for backwards compatibility)
    RewardIndex,
    /// Vesting period in seconds
    VestingPeriod,
    /// Target deposit amount for utilization calculation
    TargetDeposits,
    /// Multiplier points for dynamic reward calculation
    UtilizationMultipliers,
    /// Total stake measured in weighted units for reward accounting
    WeightedTotalDeposits,
    /// Configured lock duration models
    LockDurationModels,
    /// Reentrancy guard flag
    ReentrancyGuard,
    /// Pause flag
    IsPaused,
    /// User balance (legacy, kept for backwards compatibility)
    UserBalance(Address),
    /// User liquid balance separated from locked funds
    UserLiquidBalance(Address),
    /// User lock entries
    UserLocks(Address),
    /// User's last synced reward index (legacy, kept for backwards compatibility)
    UserRewardIndex(Address),
    /// User's accrued but unvested rewards (legacy, kept for backwards compatibility)
    UserAccruedRewards(Address),
    /// User's last reward distribution timestamp (legacy, kept for backwards compatibility)
    UserLastRewardTimestamp(Address),
    /// Penalty rate in basis points for early withdrawals
    PenaltyRateBps,
    /// Total penalty amount collected by the vault
    TotalPenalties,
    /// Total penalty amount paid by a specific user
    UserPenaltyPaid(Address),
    /// Map of supported asset addresses
    SupportedAssets,
    /// Total deposits per asset
    AssetTotalDeposits(Address),
    /// Global reward index per asset
    AssetRewardIndex(Address),
    /// User balance per asset
    UserAssetBalance(Address, Address), // (user, asset)
    /// User's last synced reward index per asset
    UserAssetRewardIndex(Address, Address), // (user, asset)
    /// User's accrued but unvested rewards per asset
    UserAssetAccruedRewards(Address, Address), // (user, asset)
    /// User's last reward distribution timestamp per asset
    UserAssetLastRewardTimestamp(Address, Address), // (user, asset)
    // -----------------------------------------------------------------------
    // Delegation keys
    // -----------------------------------------------------------------------
    /// Delegation entry: (delegator, operator) -> Delegation
    Delegation(Address, Address),
    /// List of operator addresses for a delegator
    DelegationOperators(Address),
    /// Maximum number of delegations allowed per user
    MaxDelegationsPerUser,
    /// Legacy/simple delegate authorization record
    DelegatePermissions(Address, Address),
}

/// The global state of the vault contract.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaultState {
    /// The address allowed to perform administrative actions like reward distribution.
    pub admin: Address,
    /// The address of the token that users deposit into the vault.
    pub deposit_token: Address,
    /// The address of the token distributed as rewards.
    pub reward_token: Address,
    /// The total amount of deposit tokens currently held by the vault.
    pub total_deposits: i128,
    /// The total amount of stake measured in weighted units for reward accounting.
    pub weighted_total_deposits: i128,
    /// The global reward index that tracks cumulative rewards per unit of deposit.
    pub reward_index: i128,
    /// The vesting period in seconds.
    pub vesting_period: u64,
    /// The target deposit amount for calculating utilization.
    pub target_deposits: i128,
    /// The penalty rate applied to early locked withdrawals (bps).
    pub penalty_rate_bps: u32,
    /// Total penalty amount collected by the vault.
    pub total_penalties: i128,
    /// A list of points defining the utilization-to-reward multiplier curve.
    pub utilization_multipliers: soroban_sdk::Vec<MultiplierPoint>,
    /// The configured lock duration models.
    pub lock_duration_models: soroban_sdk::Vec<LockDurationModel>,
}

/// A tranche of rewards that vests linearly from `start_timestamp`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingSchedule {
    pub amount: i128,
    pub claimed: i128,
    pub start_timestamp: u64,
    pub duration: u64,
}

/// Snapshot of a user's position in the vault for a specific asset.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserPosition {
    /// The amount of deposit tokens the user has currently staked.
    pub balance: i128,
    /// The value of the global reward index at the time of the user's last interaction.
    pub reward_index: i128,
    /// The amount of rewards the user has earned but not yet vested/claimed.
    pub accrued_rewards: i128,
    /// The timestamp of the last reward distribution affecting this user.
    pub last_reward_timestamp: u64,
}

/// Snapshot of a user's position across multiple assets.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultiAssetPosition {
    /// Map of asset address to user position
    pub positions: Map<Address, UserPosition>,
}

/// Legacy delegate authorization used by delegate-specific entrypoints.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelegateAuthorization {
    pub owner: Address,
    pub delegate: Address,
    pub permissions: u32,
    pub created_at: u64,
    pub active: bool,
}

// ---------------------------------------------------------------------------
// Delegation permissions (bitmask)
// ---------------------------------------------------------------------------

/// Permission to deposit on behalf of the delegator.
pub const PERMISSION_DEPOSIT: u32 = 1 << 0;
/// Permission to withdraw from the delegator's balance.
pub const PERMISSION_WITHDRAW: u32 = 1 << 1;
/// Permission to lock the delegator's funds.
pub const PERMISSION_LOCK: u32 = 1 << 2;
/// Permission to unlock the delegator's expired locks.
pub const PERMISSION_UNLOCK: u32 = 1 << 3;
/// Permission to claim rewards on behalf of the delegator.
pub const PERMISSION_CLAIM: u32 = 1 << 4;

/// All user-action permissions combined.
pub const PERMISSION_ALL_USER: u32 = PERMISSION_DEPOSIT
    | PERMISSION_WITHDRAW
    | PERMISSION_LOCK
    | PERMISSION_UNLOCK
    | PERMISSION_CLAIM;

/// Default maximum number of delegations per user.
pub const DEFAULT_MAX_DELEGATIONS: u32 = 20;

/// A delegation entry granting an operator specific permissions on a vault owner's positions.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Delegation {
    /// The operator address that is authorized to act.
    pub operator: Address,
    /// Bitmask of allowed permissions (see PERMISSION_* constants).
    pub permissions: u32,
    /// Timestamp after which the delegation expires (0 = never).
    pub expires_at: u64,
}

// ---------------------------------------------------------------------------
// Helper struct for returning delegation info in view functions.
// ---------------------------------------------------------------------------
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelegationInfo {
    pub operator: Address,
    pub permissions: u32,
    pub expires_at: u64,
}

/// A helper struct for returning reward information in view functions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserRewardSnapshot {
    /// The current reward index applied to the snapshot.
    pub reward_index: i128,
    /// The total rewards (accrued + pending) for the user.
    pub rewards: i128,
    /// The amount of vested rewards available to claim.
    pub vested_rewards: i128,
    /// Number of active vesting schedules included in the snapshot.
    pub vesting_schedule_count: u32,
}

// ---------------------------------------------------------------------------
// Init
// ---------------------------------------------------------------------------

pub fn is_initialized(e: &Env) -> bool {
    e.storage()
        .instance()
        .get::<_, bool>(&DataKey::Initialized)
        .unwrap_or(false)
}

pub fn require_initialized(e: &Env) -> Result<(), VaultError> {
    if is_initialized(e) {
        Ok(())
    } else {
        Err(StateError::NotInitialized.into())
    }
}

pub fn require_not_paused(e: &Env) -> Result<(), VaultError> {
    if e.storage()
        .instance()
        .get::<_, bool>(&DataKey::IsPaused)
        .unwrap_or(false)
    {
        Err(AuthorizationError::Unauthorized.into())
    } else {
        Ok(())
    }
}

pub fn initialize_state(
    e: &Env,
    admin: &Address,
    deposit_token: &Address,
    reward_token: &Address,
    vesting_period: u64,
    target_deposits: i128,
    utilization_multipliers: &soroban_sdk::Vec<MultiplierPoint>,
) {
    e.storage().instance().set(&DataKey::Initialized, &true);
    e.storage().instance().set(&DataKey::Admin, admin);
    e.storage().instance().remove(&DataKey::PendingAdmin);
    e.storage()
        .instance()
        .set(&DataKey::DepositToken, deposit_token);
    e.storage()
        .instance()
        .set(&DataKey::RewardToken, reward_token);
    e.storage()
        .instance()
        .set(&DataKey::VestingPeriod, &vesting_period);
    e.storage().instance().set(&DataKey::TotalDeposits, &0_i128);
    e.storage()
        .instance()
        .set(&DataKey::WeightedTotalDeposits, &0_i128);
    e.storage().instance().set(&DataKey::RewardIndex, &0_i128);
    e.storage()
        .instance()
        .set(&DataKey::TargetDeposits, &target_deposits);
    e.storage().instance().set(&DataKey::PenaltyRateBps, &0_u32);
    e.storage()
        .instance()
        .set(&DataKey::TotalPenalties, &0_i128);
    e.storage()
        .instance()
        .set(&DataKey::UtilizationMultipliers, utilization_multipliers);
    e.storage().instance().set(
        &DataKey::LockDurationModels,
        &default_lock_duration_models(e),
    );
    e.storage()
        .instance()
        .set(&DataKey::ReentrancyGuard, &false);
    e.storage().instance().set(&DataKey::IsPaused, &false);

    // Register the legacy deposit token as the first multi-asset vault asset so
    // legacy and asset-scoped entrypoints share the same supported-asset registry.
    let mut supported_assets = Map::new(e);
    supported_assets.set(deposit_token.clone(), true);
    e.storage()
        .instance()
        .set(&DataKey::SupportedAssets, &supported_assets);
    e.storage()
        .instance()
        .set(&DataKey::AssetTotalDeposits(deposit_token.clone()), &0_i128);
    e.storage()
        .instance()
        .set(&DataKey::AssetRewardIndex(deposit_token.clone()), &0_i128);

    bump_instance_ttl(e);
}

// ---------------------------------------------------------------------------
// State (global)
// ---------------------------------------------------------------------------

pub fn get_state(e: &Env) -> Result<VaultState, VaultError> {
    require_initialized(e)?;
    let admin = e
        .storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(StateError::InvalidState)?;
    let deposit_token = e
        .storage()
        .instance()
        .get(&DataKey::DepositToken)
        .ok_or(StateError::InvalidState)?;
    let reward_token = e
        .storage()
        .instance()
        .get(&DataKey::RewardToken)
        .ok_or(StateError::InvalidState)?;
    let total_deposits = e
        .storage()
        .instance()
        .get(&DataKey::TotalDeposits)
        .unwrap_or(0_i128);
    let weighted_total_deposits = e
        .storage()
        .instance()
        .get(&DataKey::WeightedTotalDeposits)
        .unwrap_or(0_i128);
    let reward_index = e
        .storage()
        .instance()
        .get(&DataKey::RewardIndex)
        .unwrap_or(0_i128);
    let vesting_period = e
        .storage()
        .instance()
        .get(&DataKey::VestingPeriod)
        .unwrap_or(0_u64);
    let target_deposits = e
        .storage()
        .instance()
        .get(&DataKey::TargetDeposits)
        .unwrap_or(0_i128);
    let utilization_multipliers = e
        .storage()
        .instance()
        .get(&DataKey::UtilizationMultipliers)
        .unwrap_or_else(|| soroban_sdk::Vec::new(e));
    let lock_duration_models = e
        .storage()
        .instance()
        .get(&DataKey::LockDurationModels)
        .unwrap_or_else(|| default_lock_duration_models(e));
    let penalty_rate_bps = e
        .storage()
        .instance()
        .get(&DataKey::PenaltyRateBps)
        .unwrap_or(0_u32);
    let total_penalties = e
        .storage()
        .instance()
        .get(&DataKey::TotalPenalties)
        .unwrap_or(0_i128);
    bump_instance_ttl(e);
    Ok(VaultState {
        admin,
        deposit_token,
        reward_token,
        total_deposits,
        weighted_total_deposits,
        reward_index,
        vesting_period,
        target_deposits,
        penalty_rate_bps,
        total_penalties,
        utilization_multipliers,
        lock_duration_models,
    })
}

pub fn get_admin(e: &Env) -> Result<Address, VaultError> {
    require_initialized(e)?;
    let admin = e
        .storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(StateError::InvalidState)?;
    bump_instance_ttl(e);
    Ok(admin)
}

pub fn set_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::Admin, admin);
    bump_instance_ttl(e);
}

pub fn get_pending_admin(e: &Env) -> Result<Option<Address>, VaultError> {
    require_initialized(e)?;
    let pending = e.storage().instance().get(&DataKey::PendingAdmin);
    bump_instance_ttl(e);
    Ok(pending)
}

pub fn set_pending_admin(e: &Env, pending_admin: &Address) {
    e.storage()
        .instance()
        .set(&DataKey::PendingAdmin, pending_admin);
    bump_instance_ttl(e);
}

pub fn clear_pending_admin(e: &Env) {
    e.storage().instance().remove(&DataKey::PendingAdmin);
    bump_instance_ttl(e);
}

pub fn get_deposit_token(e: &Env) -> Result<Address, VaultError> {
    require_initialized(e)?;
    let deposit_token = e
        .storage()
        .instance()
        .get(&DataKey::DepositToken)
        .ok_or(StateError::InvalidState)?;
    bump_instance_ttl(e);
    Ok(deposit_token)
}

pub fn get_reward_token(e: &Env) -> Result<Address, VaultError> {
    require_initialized(e)?;
    let reward_token = e
        .storage()
        .instance()
        .get(&DataKey::RewardToken)
        .ok_or(StateError::InvalidState)?;
    bump_instance_ttl(e);
    Ok(reward_token)
}

pub fn get_total_deposits(e: &Env) -> Result<i128, VaultError> {
    require_initialized(e)?;
    let total = e
        .storage()
        .instance()
        .get(&DataKey::TotalDeposits)
        .unwrap_or(0_i128);
    bump_instance_ttl(e);
    Ok(total)
}

pub fn set_total_deposits(e: &Env, total: i128) {
    e.storage().instance().set(&DataKey::TotalDeposits, &total);
    bump_instance_ttl(e);
}

pub fn get_weighted_total_deposits(e: &Env) -> Result<i128, VaultError> {
    Ok(get_state(e)?.weighted_total_deposits)
}

pub fn set_weighted_total_deposits(e: &Env, total: i128) {
    e.storage()
        .instance()
        .set(&DataKey::WeightedTotalDeposits, &total);
    bump_instance_ttl(e);
}

pub fn get_reward_index(e: &Env) -> Result<i128, VaultError> {
    require_initialized(e)?;
    let index = e
        .storage()
        .instance()
        .get(&DataKey::RewardIndex)
        .unwrap_or(0_i128);
    bump_instance_ttl(e);
    Ok(index)
}

pub fn set_reward_index(e: &Env, index: i128) {
    e.storage().instance().set(&DataKey::RewardIndex, &index);
    bump_instance_ttl(e);
}

pub fn get_vesting_period(e: &Env) -> Result<u64, VaultError> {
    require_initialized(e)?;
    let period = e
        .storage()
        .instance()
        .get(&DataKey::VestingPeriod)
        .unwrap_or(0_u64);
    bump_instance_ttl(e);
    Ok(period)
}

pub fn set_paused(e: &Env, paused: bool) {
    e.storage().instance().set(&DataKey::IsPaused, &paused);
    bump_instance_ttl(e);
}

pub fn set_target_deposits(e: &Env, new_target: i128) {
    e.storage()
        .instance()
        .set(&DataKey::TargetDeposits, &new_target);
    bump_instance_ttl(e);
}

pub fn set_utilization_multipliers(e: &Env, multipliers: &soroban_sdk::Vec<MultiplierPoint>) {
    e.storage()
        .instance()
        .set(&DataKey::UtilizationMultipliers, multipliers);
    bump_instance_ttl(e);
}

pub fn get_lock_duration_models(e: &Env) -> soroban_sdk::Vec<LockDurationModel> {
    e.storage()
        .instance()
        .get(&DataKey::LockDurationModels)
        .unwrap_or_else(|| default_lock_duration_models(e))
}

pub fn set_lock_duration_models(e: &Env, models: &soroban_sdk::Vec<LockDurationModel>) {
    e.storage()
        .instance()
        .set(&DataKey::LockDurationModels, models);
    bump_instance_ttl(e);
}

pub fn get_penalty_rate_bps(e: &Env) -> Result<u32, VaultError> {
    require_initialized(e)?;
    let rate = e
        .storage()
        .instance()
        .get(&DataKey::PenaltyRateBps)
        .unwrap_or(0_u32);
    bump_instance_ttl(e);
    Ok(rate)
}

pub fn authorize_delegate(
    e: &Env,
    owner: &Address,
    delegate: &Address,
    permissions: u32,
) -> Result<(), VaultError> {
    require_initialized(e)?;
    let record = DelegateAuthorization {
        owner: owner.clone(),
        delegate: delegate.clone(),
        permissions,
        created_at: e.ledger().timestamp(),
        active: true,
    };
    e.storage().instance().set(
        &DataKey::DelegatePermissions(owner.clone(), delegate.clone()),
        &record,
    );
    bump_instance_ttl(e);
    Ok(())
}

pub fn revoke_delegate(e: &Env, owner: &Address, delegate: &Address) -> Result<(), VaultError> {
    require_initialized(e)?;
    e.storage().instance().remove(&DataKey::DelegatePermissions(
        owner.clone(),
        delegate.clone(),
    ));
    bump_instance_ttl(e);
    Ok(())
}

pub fn get_delegate_permissions(
    e: &Env,
    owner: &Address,
    delegate: &Address,
) -> Result<u32, VaultError> {
    require_initialized(e)?;
    let record =
        e.storage()
            .instance()
            .get::<_, DelegateAuthorization>(&DataKey::DelegatePermissions(
                owner.clone(),
                delegate.clone(),
            ));
    match record {
        Some(auth) if auth.active => {
            bump_instance_ttl(e);
            Ok(auth.permissions)
        }
        _ => Ok(0),
    }
}

pub fn require_delegate_permission(
    e: &Env,
    owner: &Address,
    delegate: &Address,
    permission: u32,
) -> Result<(), VaultError> {
    let record =
        e.storage()
            .instance()
            .get::<_, DelegateAuthorization>(&DataKey::DelegatePermissions(
                owner.clone(),
                delegate.clone(),
            ));
    match record {
        Some(auth) if auth.active && (auth.permissions & permission) != 0 => Ok(()),
        _ => Err(AuthorizationError::Unauthorized.into()),
    }
}

pub fn set_penalty_rate_bps(e: &Env, rate_bps: u32) {
    e.storage()
        .instance()
        .set(&DataKey::PenaltyRateBps, &rate_bps);
    bump_instance_ttl(e);
}

pub fn get_total_penalties(e: &Env) -> Result<i128, VaultError> {
    require_initialized(e)?;
    let total = e
        .storage()
        .instance()
        .get(&DataKey::TotalPenalties)
        .unwrap_or(0_i128);
    bump_instance_ttl(e);
    Ok(total)
}

pub fn set_total_penalties(e: &Env, amount: i128) {
    e.storage()
        .instance()
        .set(&DataKey::TotalPenalties, &amount);
    bump_instance_ttl(e);
}

pub fn get_user_penalty_paid(e: &Env, user: &Address) -> Result<i128, VaultError> {
    require_initialized(e)?;
    let total = e
        .storage()
        .persistent()
        .get(&DataKey::UserPenaltyPaid(user.clone()))
        .unwrap_or(0_i128);
    if total != 0 {
        bump_persistent_ttl(e, &DataKey::UserPenaltyPaid(user.clone()));
    }
    Ok(total)
}

pub fn set_user_penalty_paid(e: &Env, user: &Address, amount: i128) {
    let key = DataKey::UserPenaltyPaid(user.clone());
    if amount == 0 {
        e.storage().persistent().remove(&key);
    } else {
        e.storage().persistent().set(&key, &amount);
        bump_persistent_ttl(e, &key);
    }
}

pub fn increase_penalty_totals(e: &Env, user: &Address, amount: i128) -> Result<(), VaultError> {
    let current_user = get_user_penalty_paid(e, user)?;
    let next_user = current_user
        .checked_add(amount)
        .ok_or(ArithmeticError::Overflow)?;
    set_user_penalty_paid(e, user, next_user);

    let current_total = get_total_penalties(e)?;
    let next_total = current_total
        .checked_add(amount)
        .ok_or(ArithmeticError::Overflow)?;
    set_total_penalties(e, next_total);
    Ok(())
}

pub fn store_early_withdraw_locked(
    e: &Env,
    user: &Address,
    amount: i128,
) -> Result<(VaultState, UserPosition, i128, i128), VaultError> {
    let state = get_state(e)?;
    let penalty_rate_bps = state.penalty_rate_bps;
    let mut position = get_user_position_unchecked(e, user)?;

    accrue_position_rewards(e, &state, &mut position)?;

    let current_timestamp = e.ledger().timestamp();
    let locks = get_user_locks_unchecked(e, user);

    let mut remaining = amount;
    let mut penalty_total: i128 = 0;
    let mut next_locks = soroban_sdk::Vec::new(e);

    for lock in locks.iter() {
        if remaining == 0 {
            next_locks.push_back(lock);
            continue;
        }

        let withdraw_amount = if lock.amount <= remaining {
            lock.amount
        } else {
            remaining
        };

        let penalty = if lock.unlock_timestamp > current_timestamp {
            withdraw_amount
                .checked_mul(penalty_rate_bps as i128)
                .ok_or(ArithmeticError::Overflow)?
                .checked_div(10000)
                .ok_or(ArithmeticError::RewardCalculationFailed)?
        } else {
            0
        };

        penalty_total = penalty_total
            .checked_add(penalty)
            .ok_or(ArithmeticError::Overflow)?;

        let remaining_lock_amount = lock
            .amount
            .checked_sub(withdraw_amount)
            .ok_or(ArithmeticError::Overflow)?;
        if remaining_lock_amount > 0 {
            next_locks.push_back(Lock {
                amount: remaining_lock_amount,
                unlock_timestamp: lock.unlock_timestamp,
                reward_multiplier: lock.reward_multiplier,
            });
        }

        remaining = remaining
            .checked_sub(withdraw_amount)
            .ok_or(ArithmeticError::Overflow)?;
    }

    if remaining != 0 {
        return Err(BalanceError::InsufficientBalance.into());
    }

    let net_amount = amount
        .checked_sub(penalty_total)
        .ok_or(ArithmeticError::Overflow)?;

    position.balance = position
        .balance
        .checked_sub(amount)
        .ok_or(ArithmeticError::Overflow)?;

    let next_total = state
        .total_deposits
        .checked_sub(amount)
        .ok_or(ArithmeticError::Overflow)?;

    set_user_locks(e, user, &next_locks);
    set_user_position(e, user, &position);
    set_total_deposits(e, next_total);
    increase_penalty_totals(e, user, penalty_total)?;

    Ok((
        VaultState {
            total_deposits: next_total,
            ..state
        },
        position,
        net_amount,
        penalty_total,
    ))
}

// ---------------------------------------------------------------------------
// Reentrancy Guard
// ---------------------------------------------------------------------------

pub fn enter_non_reentrant(e: &Env) -> Result<(), VaultError> {
    if e.storage()
        .instance()
        .get::<_, bool>(&DataKey::ReentrancyGuard)
        .unwrap_or(false)
    {
        return Err(AuthorizationError::ReentrancyDetected.into());
    }
    e.storage().instance().set(&DataKey::ReentrancyGuard, &true);
    bump_instance_ttl(e);
    Ok(())
}

pub fn exit_non_reentrant(e: &Env) {
    e.storage()
        .instance()
        .set(&DataKey::ReentrancyGuard, &false);
    bump_instance_ttl(e);
}

// ---------------------------------------------------------------------------
// User Position
// ---------------------------------------------------------------------------

pub fn get_user_position(e: &Env, user: &Address) -> Result<UserPosition, VaultError> {
    require_initialized(e)?;
    get_user_position_unchecked(e, user)
}

pub fn get_user_position_unchecked(e: &Env, user: &Address) -> Result<UserPosition, VaultError> {
    let liquid_balance = get_liquid_balance_unchecked(e, user);
    let locks = get_user_locks_unchecked(e, user);
    let locked_balance: i128 = locks.iter().map(|lock: Lock| lock.amount).sum();
    let total_balance = liquid_balance
        .checked_add(locked_balance)
        .ok_or(ArithmeticError::Overflow)?;

    let reward_index_key = DataKey::UserRewardIndex(user.clone());
    let accrued_rewards_key = DataKey::UserAccruedRewards(user.clone());
    let last_reward_timestamp_key = DataKey::UserLastRewardTimestamp(user.clone());
    let reward_index = e
        .storage()
        .persistent()
        .get(&reward_index_key)
        .unwrap_or(0_i128);
    let accrued_rewards = e
        .storage()
        .persistent()
        .get(&accrued_rewards_key)
        .unwrap_or(0_i128);
    let last_reward_timestamp = e
        .storage()
        .persistent()
        .get(&last_reward_timestamp_key)
        .unwrap_or(0_u64);

    if reward_index != 0 {
        bump_persistent_ttl(e, &reward_index_key);
    }
    if accrued_rewards != 0 {
        bump_persistent_ttl(e, &accrued_rewards_key);
    }
    if last_reward_timestamp != 0 {
        bump_persistent_ttl(e, &last_reward_timestamp_key);
    }

    Ok(UserPosition {
        balance: total_balance,
        reward_index,
        accrued_rewards,
        last_reward_timestamp,
    })
}

pub fn set_user_position(e: &Env, user: &Address, position: &UserPosition) {
    let reward_index_key = DataKey::UserRewardIndex(user.clone());
    let accrued_rewards_key = DataKey::UserAccruedRewards(user.clone());
    let last_reward_timestamp_key = DataKey::UserLastRewardTimestamp(user.clone());

    if position.reward_index == 0 {
        e.storage().persistent().remove(&reward_index_key);
    } else {
        e.storage()
            .persistent()
            .set(&reward_index_key, &position.reward_index);
        bump_persistent_ttl(e, &reward_index_key);
    }

    if position.accrued_rewards == 0 {
        e.storage().persistent().remove(&accrued_rewards_key);
    } else {
        e.storage()
            .persistent()
            .set(&accrued_rewards_key, &position.accrued_rewards);
        bump_persistent_ttl(e, &accrued_rewards_key);
    }

    e.storage()
        .persistent()
        .set(&last_reward_timestamp_key, &position.last_reward_timestamp);
    bump_persistent_ttl(e, &last_reward_timestamp_key);
}

pub fn get_user_balance(e: &Env, user: &Address) -> Result<i128, VaultError> {
    let position = get_user_position(e, user)?;
    Ok(position.balance)
}

pub fn get_liquid_balance(e: &Env, user: &Address) -> Result<i128, VaultError> {
    require_initialized(e)?;
    Ok(get_liquid_balance_unchecked(e, user))
}

pub fn get_liquid_balance_unchecked(e: &Env, user: &Address) -> i128 {
    let key = DataKey::UserLiquidBalance(user.clone());
    let balance = e.storage().persistent().get(&key).unwrap_or(0_i128);
    if balance != 0 {
        bump_persistent_ttl(e, &key);
    }
    balance
}

fn set_liquid_balance(e: &Env, user: &Address, amount: i128) {
    let key = DataKey::UserLiquidBalance(user.clone());
    if amount == 0 {
        e.storage().persistent().remove(&key);
    } else {
        e.storage().persistent().set(&key, &amount);
        bump_persistent_ttl(e, &key);
    }
}

pub fn get_locked_balance(e: &Env, user: &Address) -> Result<i128, VaultError> {
    require_initialized(e)?;
    let locks = get_user_locks_unchecked(e, user);
    let locked_amount: i128 = locks
        .iter()
        .filter(|l| l.unlock_timestamp > e.ledger().timestamp())
        .map(|l| l.amount)
        .sum();
    Ok(locked_amount)
}

pub fn get_user_locks_unchecked(e: &Env, user: &Address) -> soroban_sdk::Vec<Lock> {
    let key = DataKey::UserLocks(user.clone());
    let locks = e
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| soroban_sdk::Vec::new(e));
    if !locks.is_empty() {
        bump_persistent_ttl(e, &key);
    }
    locks
}

fn set_user_locks(e: &Env, user: &Address, locks: &soroban_sdk::Vec<Lock>) {
    let key = DataKey::UserLocks(user.clone());
    if locks.is_empty() {
        e.storage().persistent().remove(&key);
    } else {
        e.storage().persistent().set(&key, locks);
        bump_persistent_ttl(e, &key);
    }
}

fn collect_expired_locks(
    e: &Env,
    user: &Address,
    limit: u32,
) -> Result<(i128, i128, soroban_sdk::Vec<Lock>), VaultError> {
    let locks = get_user_locks_unchecked(e, user);
    let mut unlocked_amount: i128 = 0;
    let mut weighted_reduction: i128 = 0;
    let mut new_locks = soroban_sdk::Vec::new(e);
    let mut processed_count = 0;
    let current_timestamp = e.ledger().timestamp();

    if limit == 0 {
        return Ok((0, 0, locks));
    }

    for lock in locks.iter() {
        if lock.unlock_timestamp <= current_timestamp && processed_count < limit {
            unlocked_amount = unlocked_amount
                .checked_add(lock.amount)
                .ok_or(ArithmeticError::Overflow)?;
            weighted_reduction = weighted_reduction
                .checked_add(checked_lock_bonus(lock.amount, lock.reward_multiplier_bps)?)
                .ok_or(ArithmeticError::Overflow)?;
            processed_count += 1;
        } else {
            new_locks.push_back(lock);
        }
    }

    Ok((unlocked_amount, weighted_reduction, new_locks))
}

// ---------------------------------------------------------------------------
// Deposit/Withdraw Logic
// ---------------------------------------------------------------------------

pub fn store_deposit(
    e: &Env,
    user: &Address,
    amount: i128,
) -> Result<(VaultState, UserPosition), VaultError> {
    let state = get_state(e)?;
    let mut position = get_user_position_unchecked(e, user)?;

    // Accrue rewards earned up to this point using the old balance.
    accrue_position_rewards(e, &state, user, &mut position)?;

    // Update total balance for reward calculation purposes in the returned position.
    position.balance = position
        .balance
        .checked_add(amount)
        .ok_or(ArithmeticError::Overflow)?;

    // Update liquid balance and total contract deposits.
    let new_liquid_balance = get_liquid_balance_unchecked(e, user)
        .checked_add(amount)
        .ok_or(ArithmeticError::Overflow)?;
    let next_total = state
        .total_deposits
        .checked_add(amount)
        .ok_or(ArithmeticError::Overflow)?;
    let next_weighted_total = state
        .weighted_total_deposits
        .checked_add(weighted_liquid_amount(amount)?)
        .ok_or(ArithmeticError::Overflow)?;

    set_liquid_balance(e, user, new_liquid_balance);
    set_total_deposits(e, next_total);
    set_weighted_total_deposits(e, next_weighted_total);
    set_user_position(e, user, &position);

    Ok((
        VaultState {
            total_deposits: next_total,
            weighted_total_deposits: next_weighted_total,
            ..state
        },
        position,
    ))
}

pub fn store_withdraw(
    e: &Env,
    user: &Address,
    amount: i128,
) -> Result<(VaultState, UserPosition), VaultError> {
    let state = get_state(e)?;
    let mut position = get_user_position_unchecked(e, user)?;

    // Accrue rewards earned up to this point using the old balance.
    accrue_position_rewards(e, &state, user, &mut position)?;

    // To prevent DoS, process a small batch of expired locks automatically.
    // If more locks are expired, the user must call `unlock_expired` manually.
    const WITHDRAW_UNLOCK_LIMIT: u32 = 5;
    let (unlockable_amount, _, _) = collect_expired_locks(e, user, WITHDRAW_UNLOCK_LIMIT)?;

    let liquid_balance = get_liquid_balance_unchecked(e, user);
    let available_balance = liquid_balance
        .checked_add(unlockable_amount)
        .ok_or(ArithmeticError::Overflow)?;
    if available_balance < amount {
        return Err(BalanceError::InsufficientBalance.into());
    }

    // Process any expired locks, moving them to the liquid balance.
    unlock_expired_locks(e, user, WITHDRAW_UNLOCK_LIMIT)?;

    // Update total balance for reward calculation purposes.
    position.balance = position
        .balance
        .checked_sub(amount)
        .ok_or(ArithmeticError::Overflow)?;

    // Update liquid balance and total contract deposits using the post-unlock
    // balance, since `unlock_expired_locks` may have already moved funds into
    // the liquid bucket.
    let liquid_balance = get_liquid_balance_unchecked(e, user);
    let new_liquid_balance = liquid_balance
        .checked_sub(amount)
        .ok_or(ArithmeticError::Overflow)?;
    let next_total = state
        .total_deposits
        .checked_sub(amount)
        .ok_or(ArithmeticError::Overflow)?;
    let weighted_total = get_weighted_total_deposits(e)?;
    let next_weighted_total = weighted_total
        .checked_sub(weighted_liquid_amount(amount)?)
        .ok_or(ArithmeticError::Overflow)?;

    set_liquid_balance(e, user, new_liquid_balance);
    set_total_deposits(e, next_total);
    set_weighted_total_deposits(e, next_weighted_total);
    set_user_position(e, user, &position);

    Ok((
        VaultState {
            total_deposits: next_total,
            weighted_total_deposits: next_weighted_total,
            ..state
        },
        position,
    ))
}

// ---------------------------------------------------------------------------
// Lock/Unlock Logic
// ---------------------------------------------------------------------------

pub fn store_lock(e: &Env, user: &Address, amount: i128, duration: u64) -> Result<(), VaultError> {
    let state = get_state(e)?;
    let duration_model = find_lock_duration_model(&state.lock_duration_models, duration)?
        .ok_or(VaultError::UnsupportedLockDuration)?;
    let liquid_balance = get_liquid_balance_unchecked(e, user);
    if liquid_balance < amount {
        return Err(BalanceError::InsufficientBalance.into());
    }

    let mut position = get_user_position_unchecked(e, user)?;

    // Accrue rewards before changing balance distribution.
    accrue_position_rewards(e, &state, user, &mut position)?;
    set_user_position(e, user, &position);

    // Move funds from liquid to a new lock
    let new_liquid_balance = liquid_balance
        .checked_sub(amount)
        .ok_or(ArithmeticError::Overflow)?;
    set_liquid_balance(e, user, new_liquid_balance);
    let next_weighted_total = state
        .weighted_total_deposits
        .checked_add(checked_lock_bonus(
            amount,
            duration_model.reward_multiplier_bps,
        )?)
        .ok_or(ArithmeticError::Overflow)?;
    set_weighted_total_deposits(e, next_weighted_total);

    let mut locks = get_user_locks_unchecked(e, user);
    locks.push_back(Lock {
        amount,
        duration_seconds: duration,
        unlock_timestamp: e
            .ledger()
            .timestamp()
            .checked_add(duration)
            .ok_or(ArithmeticError::Overflow)?,
        reward_multiplier_bps: duration_model.reward_multiplier_bps,
    });
    set_user_locks(e, user, &locks);

    // Note: Total balance and total_deposits do not change.
    Ok(())
}

pub fn unlock_expired_locks(e: &Env, user: &Address, limit: u32) -> Result<i128, VaultError> {
    let (unlocked_amount, weighted_reduction, new_locks) = collect_expired_locks(e, user, limit)?;

    if unlocked_amount > 0 {
        let liquid_balance = get_liquid_balance_unchecked(e, user);
        let new_liquid_balance = liquid_balance
            .checked_add(unlocked_amount)
            .ok_or(ArithmeticError::Overflow)?;
        set_liquid_balance(e, user, new_liquid_balance);
        set_user_locks(e, user, &new_locks);
        let weighted_total = get_weighted_total_deposits(e)?;
        let next_weighted_total = weighted_total
            .checked_sub(weighted_reduction)
            .ok_or(ArithmeticError::Overflow)?;
        set_weighted_total_deposits(e, next_weighted_total);
    }

    Ok(unlocked_amount)
}

// ---------------------------------------------------------------------------
// Reward Distribution
// ---------------------------------------------------------------------------

pub fn store_reward_distribution(e: &Env, amount: i128) -> Result<VaultState, VaultError> {
    let state = get_state(e)?;

    let multiplier_bps = calculate_utilization_multiplier(
        state.total_deposits,
        state.target_deposits,
        &state.utilization_multipliers,
    )?;

    // Apply the multiplier to the distributed amount.
    let effective_amount = (amount as u128)
        .checked_mul(multiplier_bps as u128)
        .ok_or(ArithmeticError::Overflow)?
        .checked_div(10000) // Convert from basis points
        .ok_or(ArithmeticError::RewardCalculationFailed)? as i128;

    let increment =
        checked_reward_index_increment(effective_amount, state.weighted_total_deposits)?;

    let next_reward_index = state
        .reward_index
        .checked_add(increment)
        .ok_or(ArithmeticError::Overflow)?;

    set_reward_index(e, next_reward_index);

    Ok(VaultState {
        reward_index: next_reward_index,
        ..state
    })
}

// ---------------------------------------------------------------------------
// Claim Rewards
// ---------------------------------------------------------------------------

pub fn calculate_vested_rewards(
    current_timestamp: u64,
    position: &UserPosition,
    vesting_period: u64,
) -> Result<i128, VaultError> {
    if vesting_period == 0 {
        return Ok(position.accrued_rewards);
    }

    if position.last_reward_timestamp == 0 {
        return Ok(0);
    }

    let schedule = VestingSchedule {
        amount: position.accrued_rewards,
        claimed: 0,
        start_timestamp: position.last_reward_timestamp,
        duration: vesting_period,
    };
    calculate_schedule_claimable(current_timestamp, &schedule)
}

fn vesting_schedules(e: &Env, key: &DataKey) -> Vec<VestingSchedule> {
    e.storage()
        .persistent()
        .get::<_, Vec<VestingSchedule>>(key)
        .unwrap_or_else(|| Vec::new(e))
}

fn set_vesting_schedules(e: &Env, key: &DataKey, schedules: &Vec<VestingSchedule>) {
    if schedules.is_empty() {
        e.storage().persistent().remove(key);
    } else {
        e.storage().persistent().set(key, schedules);
        bump_persistent_ttl(e, key);
    }
}

fn calculate_schedule_claimable(
    current_timestamp: u64,
    schedule: &VestingSchedule,
) -> Result<i128, VaultError> {
    if schedule.amount <= schedule.claimed {
        return Ok(0);
    }
    if schedule.duration == 0 {
        return schedule
            .amount
            .checked_sub(schedule.claimed)
            .ok_or(ArithmeticError::Overflow.into());
    }

    let elapsed = current_timestamp
        .checked_sub(schedule.start_timestamp)
        .unwrap_or(0);
    let vested = if elapsed >= schedule.duration {
        schedule.amount
    } else {
        (schedule.amount as u128)
            .checked_mul(elapsed as u128)
            .ok_or(ArithmeticError::Overflow)?
            .checked_div(schedule.duration as u128)
            .ok_or(ArithmeticError::RewardCalculationFailed)? as i128
    };

    vested
        .checked_sub(schedule.claimed)
        .ok_or(ArithmeticError::Overflow.into())
}

fn append_vesting_schedule(
    e: &Env,
    key: &DataKey,
    amount: i128,
    duration: u64,
) -> Result<(), VaultError> {
    if amount <= 0 {
        return Ok(());
    }

    let mut schedules = vesting_schedules(e, key);
    schedules.push_back(VestingSchedule {
        amount,
        claimed: 0,
        start_timestamp: e.ledger().timestamp(),
        duration,
    });
    set_vesting_schedules(e, key, &schedules);
    Ok(())
}

pub fn active_vesting_schedule_count(e: &Env, user: &Address) -> u32 {
    vesting_schedules(e, &DataKey::UserRewardVestingSchedules(user.clone())).len()
}

fn claim_from_schedules(
    e: &Env,
    key: &DataKey,
    current_timestamp: u64,
) -> Result<i128, VaultError> {
    let schedules = vesting_schedules(e, key);
    let mut next = Vec::new(e);
    let mut total = 0_i128;

    for mut schedule in schedules.iter() {
        let claimable = calculate_schedule_claimable(current_timestamp, &schedule)?;
        if claimable > 0 {
            schedule.claimed = schedule
                .claimed
                .checked_add(claimable)
                .ok_or(ArithmeticError::Overflow)?;
            total = total
                .checked_add(claimable)
                .ok_or(ArithmeticError::Overflow)?;
        }
        if schedule.claimed < schedule.amount {
            next.push_back(schedule);
        }
    }

    set_vesting_schedules(e, key, &next);
    Ok(total)
}

fn preview_schedules(
    e: &Env,
    key: &DataKey,
    current_timestamp: u64,
) -> Result<(i128, i128, u32), VaultError> {
    let schedules = vesting_schedules(e, key);
    let mut unclaimed = 0_i128;
    let mut claimable = 0_i128;
    for schedule in schedules.iter() {
        unclaimed = unclaimed
            .checked_add(
                schedule
                    .amount
                    .checked_sub(schedule.claimed)
                    .ok_or(ArithmeticError::Overflow)?,
            )
            .ok_or(ArithmeticError::Overflow)?;
        claimable = claimable
            .checked_add(calculate_schedule_claimable(current_timestamp, &schedule)?)
            .ok_or(ArithmeticError::Overflow)?;
    }
    Ok((unclaimed, claimable, schedules.len()))
}

pub fn store_claimable_rewards(e: &Env, user: &Address) -> Result<i128, VaultError> {
    // Release any expired locks before calculating claimable rewards so the
    // user does not continue to earn the lock bonus past expiry.
    let _ = unlock_expired_locks(e, user, 50)?;

    let state = get_state(e)?;
    let mut position = get_user_position_unchecked(e, user)?;

    let before_accrued = position.accrued_rewards;
    accrue_position_rewards(e, &state, &mut position)?;
    let newly_accrued = position
        .accrued_rewards
        .checked_sub(before_accrued)
        .ok_or(ArithmeticError::Overflow)?;
    let key = DataKey::UserRewardVestingSchedules(user.clone());
    append_vesting_schedule(e, &key, newly_accrued, state.vesting_period)?;

    let current_timestamp = e.ledger().timestamp();
    let vested = claim_from_schedules(e, &key, current_timestamp)?;
    position.accrued_rewards = position
        .accrued_rewards
        .checked_sub(vested)
        .ok_or(ArithmeticError::Overflow)?;
    set_user_position(e, user, &position);

    Ok(vested)
}

// ---------------------------------------------------------------------------
// Read-only reward preview
// ---------------------------------------------------------------------------

pub fn preview_user_rewards(e: &Env, user: &Address) -> Result<UserRewardSnapshot, VaultError> {
    require_initialized(e)?;
    let state = get_state(e)?;
    let mut position = get_user_position_unchecked(e, user)?;

    // Calculate accrued rewards without modifying state
    accrue_position_rewards(e, &state, user, &mut position)?;

    let current_timestamp = e.ledger().timestamp();
    let key = DataKey::UserRewardVestingSchedules(user.clone());
    let (scheduled_unclaimed, vested, vesting_schedule_count) =
        preview_schedules(e, &key, current_timestamp)?;

    let unscheduled = position
        .accrued_rewards
        .checked_sub(scheduled_unclaimed)
        .ok_or(ArithmeticError::Overflow)?;

    Ok(UserRewardSnapshot {
        reward_index: position.reward_index,
        rewards: position.accrued_rewards,
        vested_rewards: vested
            .checked_add(calculate_vested_rewards(
                current_timestamp,
                &UserPosition {
                    accrued_rewards: unscheduled,
                    ..position.clone()
                },
                state.vesting_period,
            )?)
            .ok_or(ArithmeticError::Overflow)?,
        vesting_schedule_count,
    })
}

pub fn pending_user_rewards_view(e: &Env, user: &Address) -> Result<i128, VaultError> {
    Ok(preview_user_rewards(e, user)?.rewards)
}

pub fn vested_user_rewards_view(e: &Env, user: &Address) -> Result<i128, VaultError> {
    Ok(preview_user_rewards(e, user)?.vested_rewards)
}

// ---------------------------------------------------------------------------
// Helper Functions
// ---------------------------------------------------------------------------

pub(crate) fn checked_reward_index_increment(
    amount: i128,
    total_deposits: i128,
) -> Result<i128, VaultError> {
    if total_deposits <= 0 {
        return Err(BalanceError::NoDeposits.into());
    }

    let scaled = amount
        .checked_mul(REWARD_INDEX_SCALE)
        .ok_or(ArithmeticError::Overflow)?;
    let increment = scaled
        .checked_div(total_deposits)
        .ok_or(ArithmeticError::RewardCalculationFailed)?;

    if increment <= 0 {
        return Err(ArithmeticError::ZeroRewardIncrement.into());
    }

    Ok(increment)
}

pub(crate) fn checked_accrued_rewards(balance: i128, delta: i128) -> Result<i128, VaultError> {
    balance
        .checked_mul(delta)
        .ok_or(ArithmeticError::Overflow)?
        .checked_div(REWARD_INDEX_SCALE)
        .ok_or(ArithmeticError::RewardCalculationFailed.into())
}

fn weighted_liquid_amount(amount: i128) -> Result<i128, VaultError> {
    amount
        .checked_mul(LOCK_WEIGHT_SCALE)
        .ok_or(ArithmeticError::Overflow.into())
}

fn checked_lock_bonus(amount: i128, multiplier_bps: u32) -> Result<i128, VaultError> {
    let multiplier = i128::from(multiplier_bps);
    let weighted_amount = amount
        .checked_mul(multiplier)
        .ok_or(ArithmeticError::Overflow)?;
    weighted_amount
        .checked_sub(weighted_liquid_amount(amount)?)
        .ok_or(ArithmeticError::Overflow.into())
}

fn get_user_effective_balance_units_unchecked(e: &Env, user: &Address) -> Result<i128, VaultError> {
    let liquid_balance = get_liquid_balance_unchecked(e, user);
    let mut weighted_balance = weighted_liquid_amount(liquid_balance)?;
    let current_timestamp = e.ledger().timestamp();
    let locks = get_user_locks_unchecked(e, user);
    for lock in locks.iter() {
        if lock.unlock_timestamp <= current_timestamp {
            continue;
        }
        weighted_balance = weighted_balance
            .checked_add(
                lock.amount
                    .checked_mul(i128::from(lock.reward_multiplier_bps))
                    .ok_or(ArithmeticError::Overflow)?,
            )
            .ok_or(ArithmeticError::Overflow)?;
    }
    Ok(weighted_balance)
}

fn find_lock_duration_model(
    models: &soroban_sdk::Vec<LockDurationModel>,
    duration_seconds: u64,
) -> Result<Option<LockDurationModel>, VaultError> {
    if models.is_empty() {
        return Ok(None);
    }

    for model in models.iter() {
        if model.duration_seconds == duration_seconds {
            return Ok(Some(model));
        }
    }

    Ok(None)
}

fn default_lock_duration_models(e: &Env) -> soroban_sdk::Vec<LockDurationModel> {
    let mut models = soroban_sdk::Vec::new(e);
    models.push_back(LockDurationModel {
        duration_seconds: 7 * 24 * 60 * 60,
        reward_multiplier_bps: 11_000,
    });
    models.push_back(LockDurationModel {
        duration_seconds: 30 * 24 * 60 * 60,
        reward_multiplier_bps: 12_500,
    });
    models.push_back(LockDurationModel {
        duration_seconds: 90 * 24 * 60 * 60,
        reward_multiplier_bps: 15_000,
    });
    models
}

pub fn validate_lock_duration_models(
    models: &soroban_sdk::Vec<LockDurationModel>,
) -> Result<(), VaultError> {
    if models.is_empty() {
        return Err(ValidationError::InvalidLockConfiguration.into());
    }

    let mut previous_duration = 0_u64;
    let mut previous_multiplier = 10_000_u32;
    for model in models.iter() {
        if model.duration_seconds == 0 || model.reward_multiplier_bps < 10_000 {
            return Err(ValidationError::InvalidLockConfiguration.into());
        }
        if model.duration_seconds <= previous_duration {
            return Err(ValidationError::InvalidLockConfiguration.into());
        }
        if model.reward_multiplier_bps < previous_multiplier {
            return Err(ValidationError::InvalidLockConfiguration.into());
        }
        previous_duration = model.duration_seconds;
        previous_multiplier = model.reward_multiplier_bps;
    }

    Ok(())
}

fn calculate_utilization_multiplier(
    total_deposits: i128,
    target_deposits: i128,
    multipliers: &soroban_sdk::Vec<MultiplierPoint>,
) -> Result<u32, VaultError> {
    // If no target is set or no multipliers are defined, default to 1.0x.
    if target_deposits <= 0 || multipliers.is_empty() {
        return Ok(10000);
    }

    let utilization_bps = total_deposits
        .checked_mul(10000)
        .ok_or(ArithmeticError::Overflow)?
        .checked_div(target_deposits)
        .ok_or(ArithmeticError::RewardCalculationFailed)? as u32;

    // The multiplier curve is defined by points. Find the first point that
    // the current utilization is less than or equal to.
    // The list of points is expected to be sorted by `utilization_bps`.
    let mut selected_multiplier = multipliers.last().unwrap().multiplier_bps;
    for point in multipliers.iter() {
        if utilization_bps <= point.utilization_bps {
            selected_multiplier = point.multiplier_bps;
            break;
        }
    }

    Ok(selected_multiplier)
}

fn accrue_position_rewards(
    e: &Env,
    state: &VaultState,
    user: &Address,
    position: &mut UserPosition,
) -> Result<(), VaultError> {
    if state.reward_index == position.reward_index || position.balance == 0 {
        position.reward_index = state.reward_index;
        return Ok(());
    }

    if position.balance > 0 {
        let delta = state
            .reward_index
            .checked_sub(position.reward_index)
            .ok_or(ArithmeticError::Overflow)?;
        let weighted_balance = get_user_effective_balance_units_unchecked(e, user)?;
        let accrued = checked_accrued_rewards(weighted_balance, delta)?;

        if accrued > 0 {
            position.accrued_rewards = position
                .accrued_rewards
                .checked_add(accrued)
                .ok_or(ArithmeticError::Overflow)?;
            // Update last reward timestamp whenever new rewards are accrued
            position.last_reward_timestamp = e.ledger().timestamp();
        }
    }

    position.reward_index = state.reward_index;
    Ok(())
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

// ---------------------------------------------------------------------------
// Multi-Asset Support Functions
// ---------------------------------------------------------------------------

/// Add a new supported asset to the vault
pub fn add_supported_asset(e: &Env, asset: &Address) -> Result<(), VaultError> {
    let mut assets = get_supported_assets(e);
    if !assets.contains_key(asset.clone()) {
        assets.set(asset.clone(), true);
        e.storage()
            .instance()
            .set(&DataKey::SupportedAssets, &assets);

        // Initialize asset-specific state
        e.storage()
            .instance()
            .set(&DataKey::AssetTotalDeposits(asset.clone()), &0_i128);
        e.storage()
            .instance()
            .set(&DataKey::AssetRewardIndex(asset.clone()), &0_i128);

        bump_instance_ttl(e);
    }
    Ok(())
}

/// Get all supported assets
pub fn get_supported_assets(e: &Env) -> Map<Address, bool> {
    e.storage()
        .instance()
        .get(&DataKey::SupportedAssets)
        .unwrap_or(Map::new(e))
}

/// Check if an asset is supported
pub fn is_asset_supported(e: &Env, asset: &Address) -> bool {
    let assets = get_supported_assets(e);
    assets.contains_key(asset.clone())
}

/// Get total deposits for a specific asset
pub fn get_asset_total_deposits(e: &Env, asset: &Address) -> Result<i128, VaultError> {
    if !is_asset_supported(e, asset) {
        return Err(ValidationError::InvalidAddress.into());
    }
    Ok(e.storage()
        .instance()
        .get(&DataKey::AssetTotalDeposits(asset.clone()))
        .unwrap_or(0_i128))
}

/// Set total deposits for a specific asset
pub fn set_asset_total_deposits(e: &Env, asset: &Address, total: i128) {
    e.storage()
        .instance()
        .set(&DataKey::AssetTotalDeposits(asset.clone()), &total);
    bump_instance_ttl(e);
}

/// Get reward index for a specific asset
pub fn get_asset_reward_index(e: &Env, asset: &Address) -> Result<i128, VaultError> {
    if !is_asset_supported(e, asset) {
        return Err(ValidationError::InvalidAddress.into());
    }
    Ok(e.storage()
        .instance()
        .get(&DataKey::AssetRewardIndex(asset.clone()))
        .unwrap_or(0_i128))
}

/// Set reward index for a specific asset
pub fn set_asset_reward_index(e: &Env, asset: &Address, index: i128) {
    e.storage()
        .instance()
        .set(&DataKey::AssetRewardIndex(asset.clone()), &index);
    bump_instance_ttl(e);
}

/// Get user position for a specific asset
pub fn get_user_asset_position(
    e: &Env,
    user: &Address,
    asset: &Address,
) -> Result<UserPosition, VaultError> {
    require_initialized(e)?;
    if !is_asset_supported(e, asset) {
        return Err(ValidationError::InvalidAddress.into());
    }
    Ok(get_user_asset_position_unchecked(e, user, asset))
}

pub fn get_user_asset_position_unchecked(e: &Env, user: &Address, asset: &Address) -> UserPosition {
    let balance_key = DataKey::UserAssetBalance(user.clone(), asset.clone());
    let reward_index_key = DataKey::UserAssetRewardIndex(user.clone(), asset.clone());
    let accrued_rewards_key = DataKey::UserAssetAccruedRewards(user.clone(), asset.clone());
    let last_reward_timestamp_key =
        DataKey::UserAssetLastRewardTimestamp(user.clone(), asset.clone());

    let balance = e.storage().persistent().get(&balance_key).unwrap_or(0_i128);
    let reward_index = e
        .storage()
        .persistent()
        .get(&reward_index_key)
        .unwrap_or(0_i128);
    let accrued_rewards = e
        .storage()
        .persistent()
        .get(&accrued_rewards_key)
        .unwrap_or(0_i128);
    let last_reward_timestamp = e
        .storage()
        .persistent()
        .get(&last_reward_timestamp_key)
        .unwrap_or(0_u64);

    if balance != 0 {
        bump_persistent_ttl(e, &balance_key);
    }
    if reward_index != 0 {
        bump_persistent_ttl(e, &reward_index_key);
    }
    if accrued_rewards != 0 {
        bump_persistent_ttl(e, &accrued_rewards_key);
    }
    if last_reward_timestamp != 0 {
        bump_persistent_ttl(e, &last_reward_timestamp_key);
    }

    UserPosition {
        balance,
        reward_index,
        accrued_rewards,
        last_reward_timestamp,
    }
}

/// Set user position for a specific asset
pub fn set_user_asset_position(e: &Env, user: &Address, asset: &Address, position: &UserPosition) {
    let balance_key = DataKey::UserAssetBalance(user.clone(), asset.clone());
    let reward_index_key = DataKey::UserAssetRewardIndex(user.clone(), asset.clone());
    let accrued_rewards_key = DataKey::UserAssetAccruedRewards(user.clone(), asset.clone());
    let last_reward_timestamp_key =
        DataKey::UserAssetLastRewardTimestamp(user.clone(), asset.clone());

    if position.balance == 0 {
        e.storage().persistent().remove(&balance_key);
    } else {
        e.storage()
            .persistent()
            .set(&balance_key, &position.balance);
        bump_persistent_ttl(e, &balance_key);
    }

    if position.reward_index == 0 {
        e.storage().persistent().remove(&reward_index_key);
    } else {
        e.storage()
            .persistent()
            .set(&reward_index_key, &position.reward_index);
        bump_persistent_ttl(e, &reward_index_key);
    }

    if position.accrued_rewards == 0 {
        e.storage().persistent().remove(&accrued_rewards_key);
    } else {
        e.storage()
            .persistent()
            .set(&accrued_rewards_key, &position.accrued_rewards);
        bump_persistent_ttl(e, &accrued_rewards_key);
    }

    e.storage()
        .persistent()
        .set(&last_reward_timestamp_key, &position.last_reward_timestamp);
    bump_persistent_ttl(e, &last_reward_timestamp_key);
}

/// Get user balance for a specific asset
pub fn get_user_asset_balance(
    e: &Env,
    user: &Address,
    asset: &Address,
) -> Result<i128, VaultError> {
    Ok(get_user_asset_position(e, user, asset)?.balance)
}

/// Store deposit for a specific asset
pub fn store_asset_deposit(
    e: &Env,
    user: &Address,
    asset: &Address,
    amount: i128,
) -> Result<UserPosition, VaultError> {
    if !is_asset_supported(e, asset) {
        return Err(ValidationError::InvalidAddress.into());
    }

    let mut position = get_user_asset_position_unchecked(e, user, asset);
    let asset_reward_index = get_asset_reward_index(e, asset)?;
    let asset_total = get_asset_total_deposits(e, asset)?;

    // Accrue rewards earned up to this point using the old balance.
    accrue_asset_position_rewards(e, asset_reward_index, &mut position)?;

    // Update balance and total deposits.
    position.balance = position
        .balance
        .checked_add(amount)
        .ok_or(ArithmeticError::Overflow)?;
    let next_total = asset_total
        .checked_add(amount)
        .ok_or(ArithmeticError::Overflow)?;

    // Persist changes.
    set_asset_total_deposits(e, asset, next_total);
    set_user_asset_position(e, user, asset, &position);

    Ok(position)
}

/// Store withdraw for a specific asset
pub fn store_asset_withdraw(
    e: &Env,
    user: &Address,
    asset: &Address,
    amount: i128,
) -> Result<UserPosition, VaultError> {
    if !is_asset_supported(e, asset) {
        return Err(ValidationError::InvalidAddress.into());
    }

    let mut position = get_user_asset_position_unchecked(e, user, asset);
    let asset_reward_index = get_asset_reward_index(e, asset)?;
    let asset_total = get_asset_total_deposits(e, asset)?;

    // Accrue rewards earned up to this point using the old balance.
    accrue_asset_position_rewards(e, asset_reward_index, &mut position)?;

    if position.balance < amount {
        return Err(BalanceError::InsufficientBalance.into());
    }

    // Update balance and total deposits.
    position.balance = position
        .balance
        .checked_sub(amount)
        .ok_or(ArithmeticError::Overflow)?;
    let next_total = asset_total
        .checked_sub(amount)
        .ok_or(ArithmeticError::Overflow)?;

    // Persist changes.
    set_asset_total_deposits(e, asset, next_total);
    set_user_asset_position(e, user, asset, &position);

    Ok(position)
}

/// Store reward distribution for a specific asset
pub fn store_asset_reward_distribution(
    e: &Env,
    asset: &Address,
    amount: i128,
) -> Result<i128, VaultError> {
    if !is_asset_supported(e, asset) {
        return Err(ValidationError::InvalidAddress.into());
    }

    let asset_total = get_asset_total_deposits(e, asset)?;
    let asset_reward_index = get_asset_reward_index(e, asset)?;

    let increment = checked_reward_index_increment(amount, asset_total)?;
    let next_reward_index = asset_reward_index
        .checked_add(increment)
        .ok_or(ArithmeticError::Overflow)?;

    set_asset_reward_index(e, asset, next_reward_index);

    Ok(next_reward_index)
}

/// Claim rewards for a specific asset
pub fn store_asset_claimable_rewards(
    e: &Env,
    user: &Address,
    asset: &Address,
) -> Result<i128, VaultError> {
    if !is_asset_supported(e, asset) {
        return Err(ValidationError::InvalidAddress.into());
    }

    let state = get_state(e)?;
    let vesting_period = get_vesting_period(e)?;
    let mut position = get_user_asset_position_unchecked(e, user, asset);
    let asset_reward_index = get_asset_reward_index(e, asset)?;

    // Accrue all rewards earned up to the current global index.
    accrue_asset_position_rewards(e, asset_reward_index, &mut position)?;

    // Calculate vested rewards
    let current_timestamp = e.ledger().timestamp();
    let vested = calculate_vested_rewards(current_timestamp, &position, vesting_period)?;

    // Update position with remaining accrued rewards
    position.accrued_rewards = position
        .accrued_rewards
        .checked_sub(vested)
        .ok_or(ArithmeticError::Overflow)?;

    set_user_asset_position(e, user, asset, &position);

    Ok(vested)
}

/// Preview user rewards for a specific asset
pub fn preview_user_asset_rewards(
    e: &Env,
    user: &Address,
    asset: &Address,
) -> Result<UserRewardSnapshot, VaultError> {
    require_initialized(e)?;
    if !is_asset_supported(e, asset) {
        return Err(ValidationError::InvalidAddress.into());
    }

    let state = get_state(e)?;
    let vesting_period = get_vesting_period(e)?;
    let mut position = get_user_asset_position_unchecked(e, user, asset);
    let asset_reward_index = get_asset_reward_index(e, asset)?;

    // Calculate accrued rewards without modifying state
    accrue_asset_position_rewards(e, asset_reward_index, &mut position)?;

    let current_timestamp = e.ledger().timestamp();
    let vested = calculate_vested_rewards(current_timestamp, &position, vesting_period)?;

    Ok(UserRewardSnapshot {
        reward_index: position.reward_index,
        rewards: position.accrued_rewards,
        vested_rewards: vested,
        vesting_schedule_count: 0,
    })
}

pub fn pending_user_asset_rewards_view(
    e: &Env,
    user: &Address,
    asset: &Address,
) -> Result<i128, VaultError> {
    Ok(preview_user_asset_rewards(e, user, asset)?.rewards)
}

pub fn vested_user_asset_rewards_view(
    e: &Env,
    user: &Address,
    asset: &Address,
) -> Result<i128, VaultError> {
    Ok(preview_user_asset_rewards(e, user, asset)?.vested_rewards)
}

fn accrue_asset_position_rewards(
    e: &Env,
    asset_reward_index: i128,
    position: &mut UserPosition,
) -> Result<(), VaultError> {
    if asset_reward_index == position.reward_index || position.balance == 0 {
        position.reward_index = asset_reward_index;
        return Ok(());
    }

    if position.balance > 0 {
        let delta = asset_reward_index
            .checked_sub(position.reward_index)
            .ok_or(ArithmeticError::Overflow)?;
        let accrued = checked_accrued_rewards(position.balance, delta)?;

        if accrued > 0 {
            position.accrued_rewards = position
                .accrued_rewards
                .checked_add(accrued)
                .ok_or(ArithmeticError::Overflow)?;
            // Update last reward timestamp whenever new rewards are accrued
            position.last_reward_timestamp = e.ledger().timestamp();
        }
    }

    position.reward_index = asset_reward_index;
    Ok(())
}

// ---------------------------------------------------------------------------
// Delegation Storage
// ---------------------------------------------------------------------------

/// Get or create the maximum delegations per user setting.
pub fn get_max_delegations(e: &Env) -> u32 {
    e.storage()
        .instance()
        .get(&DataKey::MaxDelegationsPerUser)
        .unwrap_or(DEFAULT_MAX_DELEGATIONS)
}

/// Set the maximum delegations per user (admin function).
pub fn set_max_delegations(e: &Env, max: u32) {
    e.storage()
        .instance()
        .set(&DataKey::MaxDelegationsPerUser, &max);
    bump_instance_ttl(e);
}

/// Get the delegation entry for a (delegator, operator) pair.
pub fn get_delegation(e: &Env, delegator: &Address, operator: &Address) -> Option<Delegation> {
    let key = DataKey::Delegation(delegator.clone(), operator.clone());
    let result = e.storage().persistent().get::<_, Delegation>(&key);
    if result.is_some() {
        bump_persistent_ttl(e, &key);
    }
    result
}

/// Store or update a delegation entry.
pub fn set_delegation(
    e: &Env,
    delegator: &Address,
    operator: &Address,
    permissions: u32,
    expires_at: u64,
) {
    let key = DataKey::Delegation(delegator.clone(), operator.clone());
    e.storage().persistent().set(
        &key,
        &Delegation {
            operator: operator.clone(),
            permissions,
            expires_at,
        },
    );
    bump_persistent_ttl(e, &key);

    // Ensure the operator appears in the delegator's operator list.
    let mut operators = get_delegation_operators(e, delegator);
    if !operators.contains(operator.clone()) {
        operators.push_back(operator.clone());
        e.storage()
            .persistent()
            .set(&DataKey::DelegationOperators(delegator.clone()), &operators);
        bump_persistent_ttl(e, &DataKey::DelegationOperators(delegator.clone()));
    }
}

/// Remove a delegation entry and clean up the operator list.
pub fn remove_delegation(e: &Env, delegator: &Address, operator: &Address) {
    let key = DataKey::Delegation(delegator.clone(), operator.clone());
    e.storage().persistent().remove(&key);

    // Remove operator from the delegator's operator list.
    let mut operators = get_delegation_operators(e, delegator);
    if let Some(pos) = operators.first_index_of(operator.clone()) {
        operators.remove(pos as u32);
    }
    if operators.is_empty() {
        e.storage()
            .persistent()
            .remove(&DataKey::DelegationOperators(delegator.clone()));
    } else {
        e.storage()
            .persistent()
            .set(&DataKey::DelegationOperators(delegator.clone()), &operators);
        bump_persistent_ttl(e, &DataKey::DelegationOperators(delegator.clone()));
    }
}

/// Get the list of operators a delegator has granted permissions to.
pub fn get_delegation_operators(e: &Env, delegator: &Address) -> soroban_sdk::Vec<Address> {
    let key = DataKey::DelegationOperators(delegator.clone());
    let result = e
        .storage()
        .persistent()
        .get::<_, soroban_sdk::Vec<Address>>(&key)
        .unwrap_or_else(|| soroban_sdk::Vec::new(e));
    if !result.is_empty() {
        bump_persistent_ttl(e, &key);
    }
    result
}

/// Count how many delegations a delegator has.
pub fn delegation_count(e: &Env, delegator: &Address) -> u32 {
    get_delegation_operators(e, delegator).len()
}

/// Check whether an operator has a specific permission for a delegator.
/// Returns Ok(()) if the delegation exists, is not expired, and includes the permission.
pub fn check_delegation_permission(
    e: &Env,
    delegator: &Address,
    operator: &Address,
    permission: u32,
) -> Result<(), VaultError> {
    let delegation = get_delegation(e, delegator, operator).ok_or(DelegationError::NotFound)?;

    // Check expiration.
    let current_ts = e.ledger().timestamp();
    if delegation.expires_at != 0 && current_ts >= delegation.expires_at {
        return Err(DelegationError::Expired.into());
    }

    // Check the permission bit.
    if delegation.permissions & permission == 0 {
        return Err(DelegationError::InsufficientPermissions.into());
    }

    Ok(())
}

/// Verify that the operator is authorized to act on behalf of the user.
/// If `user == operator`, the user's own auth is required.
/// Otherwise, the operator must have the given permission via a delegation.
pub fn authorize_for_user(
    e: &Env,
    user: &Address,
    operator: &Address,
    permission: u32,
) -> Result<(), VaultError> {
    if user == operator {
        user.require_auth();
    } else {
        operator.require_auth();
        check_delegation_permission(e, user, operator, permission)?;
    }
    Ok(())
}

#[cfg(test)]
mod vesting_schedule_tests {
    use super::*;

    #[test]
    fn schedule_claimable_is_linear_and_partial_claim_aware() {
        let schedule = VestingSchedule {
            amount: 1_000,
            claimed: 250,
            start_timestamp: 100,
            duration: 100,
        };

        assert_eq!(calculate_schedule_claimable(150, &schedule).unwrap(), 250);
        assert_eq!(calculate_schedule_claimable(200, &schedule).unwrap(), 750);
    }

    #[test]
    fn zero_duration_schedule_is_immediately_claimable() {
        let schedule = VestingSchedule {
            amount: 1_000,
            claimed: 400,
            start_timestamp: 100,
            duration: 0,
        };

        assert_eq!(calculate_schedule_claimable(100, &schedule).unwrap(), 600);
    }
}
