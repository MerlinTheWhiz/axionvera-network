#![no_std]

use soroban_sdk::{contracterror, contracttype, Address, BytesN, Env, Vec};

/// Trait that all event emitters must implement.
/// Ensures each action emits a well-formed event with the standard two-topic pattern.
pub trait VaultEventEmitter {
    fn emit_initialize(e: &Env, admin: Address, deposit_token: Address, reward_token: Address);
    fn emit_deposit(e: &Env, user: Address, amount: i128);
    fn emit_withdraw(e: &Env, user: Address, amount: i128, remaining_balance: i128);
    fn emit_distribute(e: &Env, caller: Address, amount: i128);
    fn emit_claim_rewards(e: &Env, user: Address, amount: i128);
    fn emit_lock(e: &Env, user: Address, amount: i128, unlock_timestamp: u64);
    fn emit_unlock(e: &Env, user: Address, amount: i128);
    fn emit_admin_transfer_proposed(e: &Env, current_admin: Address, pending_admin: Address);
    fn emit_admin_transfer_accepted(e: &Env, previous_admin: Address, new_admin: Address);
    fn emit_upgrade(e: &Env, admin: Address, new_wasm_hash: BytesN<32>);
    fn emit_pause(e: &Env, admin: Address);
    fn emit_unpause(e: &Env, admin: Address);
    fn emit_asset_added(e: &Env, asset: Address);
    fn emit_asset_deposit(e: &Env, user: Address, asset: Address, amount: i128);
    fn emit_asset_withdraw(
        e: &Env,
        user: Address,
        asset: Address,
        amount: i128,
        remaining_balance: i128,
    );
    fn emit_asset_distribute(e: &Env, caller: Address, asset: Address, amount: i128);
    fn emit_asset_claim_rewards(e: &Env, user: Address, asset: Address, amount: i128);
}

pub const TREASURY_BPS_DENOMINATOR: u32 = 10_000;

/// A single recipient allocation in a treasury strategy.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllocationRule {
    pub recipient: Address,
    pub share_bps: u32,
}

/// Governance-controlled allocation strategy for protocol-owned assets.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllocationStrategy {
    pub id: BytesN<32>,
    pub rules: Vec<AllocationRule>,
}

/// A concrete transfer made during a treasury distribution.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllocationTransfer {
    pub recipient: Address,
    pub amount: i128,
}

/// Audit receipt for a completed treasury distribution.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreasuryDistributionReceipt {
    pub distribution_id: BytesN<32>,
    pub strategy_id: BytesN<32>,
    pub asset: Address,
    pub total_amount: i128,
    pub transfers: Vec<AllocationTransfer>,
    pub timestamp: u64,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum TreasuryError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    EmptyStrategy = 4,
    TooManyRules = 5,
    InvalidShare = 6,
    InvalidShareTotal = 7,
    DuplicateRecipient = 8,
    InvalidAmount = 9,
    StrategyNotFound = 10,
    DuplicateDistribution = 11,
    InsufficientBalance = 12,
    TransferFailed = 13,
}

/// Interface implemented by treasury allocation engines.
pub trait TreasuryAllocator {
    fn initialize(e: Env, admin: Address, asset: Address) -> Result<(), TreasuryError>;
    fn configure_strategy(
        e: Env,
        admin: Address,
        strategy: AllocationStrategy,
    ) -> Result<(), TreasuryError>;
    fn distribute(
        e: Env,
        admin: Address,
        distribution_id: BytesN<32>,
        strategy_id: BytesN<32>,
        amount: i128,
    ) -> Result<TreasuryDistributionReceipt, TreasuryError>;
    fn strategy(e: Env, strategy_id: BytesN<32>) -> Option<AllocationStrategy>;
    fn distribution_receipt(
        e: Env,
        distribution_id: BytesN<32>,
    ) -> Option<TreasuryDistributionReceipt>;
    fn recipient_distributed(e: Env, recipient: Address) -> i128;
    fn total_distributed(e: Env) -> i128;
}
