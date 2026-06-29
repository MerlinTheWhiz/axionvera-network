#![no_std]

#[cfg(test)]
extern crate std;

use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol};

use axionvera_events::{self, AccountingEvent, ACT_ACCOUNTING, EVENT_VERSION, PROTOCOL};

const INSTANCE_TTL_THRESHOLD: u32 = 518_400;
const INSTANCE_TTL_EXTEND_TO: u32 = 518_400;
const PERSISTENT_TTL_THRESHOLD: u32 = 518_400;
const PERSISTENT_TTL_EXTEND_TO: u32 = 518_400;

/// High-level protocol area that consumed resources.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccountingCategory {
    Vault,
    Rewards,
    Treasury,
    Governance,
}

/// Deterministic operation buckets used for protocol accounting.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccountingOperation {
    Initialize,
    VaultDeposit,
    VaultWithdraw,
    VaultLock,
    VaultUnlock,
    VaultEarlyWithdraw,
    AssetAdded,
    AssetDeposit,
    AssetWithdraw,
    RewardDistribute,
    RewardClaim,
    AssetRewardDistribute,
    AssetRewardClaim,
    TreasuryPenalty,
    GovernanceAdminPropose,
    GovernanceAdminAccept,
    GovernancePause,
    GovernanceUnpause,
    GovernanceUpgrade,
    GovernanceSetParameter,
}

/// Storage keys owned by the accounting engine.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    AccountingTotals,
    CategoryTotals(AccountingCategory),
    OperationTotals(AccountingOperation),
    AssetTotals(Address),
}

/// Deterministic resource estimates for a protocol action.
///
/// Soroban does not expose a portable on-chain budget snapshot for contracts to
/// persist. The accounting engine therefore records deterministic, operation-
/// class estimates that are stable across validators and replayable in audits.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OperationResources {
    pub storage_reads: u32,
    pub storage_writes: u32,
    pub events_emitted: u32,
    pub token_transfers: u32,
}

/// A normalized accounting entry emitted by protocol operations.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountingEntry {
    pub category: AccountingCategory,
    pub operation: AccountingOperation,
    pub actor: Option<Address>,
    pub asset: Option<Address>,
    pub amount_in: i128,
    pub amount_out: i128,
    pub amount_processed: i128,
    pub resources: OperationResources,
}

/// Aggregated resource usage for a category, operation, asset, or the protocol.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceTotals {
    pub operation_count: u64,
    pub amount_in: i128,
    pub amount_out: i128,
    pub amount_processed: i128,
    pub net_amount: i128,
    pub storage_reads: u64,
    pub storage_writes: u64,
    pub events_emitted: u64,
    pub token_transfers: u64,
}

/// Deterministic report generated from fixed accounting buckets.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccountingReport {
    pub event_version: u32,
    pub generated_at: u64,
    pub ledger: u32,
    pub total: ResourceTotals,
    pub vault: ResourceTotals,
    pub rewards: ResourceTotals,
    pub treasury: ResourceTotals,
    pub governance: ResourceTotals,
    pub deterministic_checksum: i128,
    pub consistent: bool,
}

/// Errors that can occur while recording accounting entries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccountingError {
    NegativeAmount,
    Overflow,
    InconsistentTotals,
}

impl AccountingCategory {
    pub fn symbol(self) -> Symbol {
        match self {
            AccountingCategory::Vault => symbol_short!("vault"),
            AccountingCategory::Rewards => symbol_short!("rewards"),
            AccountingCategory::Treasury => symbol_short!("treasury"),
            AccountingCategory::Governance => symbol_short!("govern"),
        }
    }
}

impl AccountingOperation {
    pub fn symbol(self) -> Symbol {
        match self {
            AccountingOperation::Initialize => symbol_short!("init"),
            AccountingOperation::VaultDeposit => symbol_short!("deposit"),
            AccountingOperation::VaultWithdraw => symbol_short!("withdraw"),
            AccountingOperation::VaultLock => symbol_short!("lock"),
            AccountingOperation::VaultUnlock => symbol_short!("unlock"),
            AccountingOperation::VaultEarlyWithdraw => symbol_short!("early_wd"),
            AccountingOperation::AssetAdded => symbol_short!("asset_add"),
            AccountingOperation::AssetDeposit => symbol_short!("asset_dep"),
            AccountingOperation::AssetWithdraw => symbol_short!("asset_wd"),
            AccountingOperation::RewardDistribute => symbol_short!("rw_dist"),
            AccountingOperation::RewardClaim => symbol_short!("rw_claim"),
            AccountingOperation::AssetRewardDistribute => symbol_short!("ast_dist"),
            AccountingOperation::AssetRewardClaim => symbol_short!("asset_clm"),
            AccountingOperation::TreasuryPenalty => symbol_short!("penalty"),
            AccountingOperation::GovernanceAdminPropose => symbol_short!("adm_prop"),
            AccountingOperation::GovernanceAdminAccept => symbol_short!("adm_acpt"),
            AccountingOperation::GovernancePause => symbol_short!("pause"),
            AccountingOperation::GovernanceUnpause => symbol_short!("unpause"),
            AccountingOperation::GovernanceUpgrade => symbol_short!("upgrade"),
            AccountingOperation::GovernanceSetParameter => symbol_short!("set_param"),
        }
    }
}

impl OperationResources {
    pub const fn new(
        storage_reads: u32,
        storage_writes: u32,
        events_emitted: u32,
        token_transfers: u32,
    ) -> Self {
        Self {
            storage_reads,
            storage_writes,
            events_emitted,
            token_transfers,
        }
    }
}

impl ResourceTotals {
    pub const fn zero() -> Self {
        Self {
            operation_count: 0,
            amount_in: 0,
            amount_out: 0,
            amount_processed: 0,
            net_amount: 0,
            storage_reads: 0,
            storage_writes: 0,
            events_emitted: 0,
            token_transfers: 0,
        }
    }
}

/// Record a normalized accounting entry, update all deterministic aggregates,
/// and emit a structured accounting event.
pub fn record_operation(e: &Env, entry: AccountingEntry) -> Result<(), AccountingError> {
    validate_entry(&entry)?;

    update_bucket(e, DataKey::AccountingTotals, &entry)?;
    update_bucket(e, DataKey::CategoryTotals(entry.category), &entry)?;
    update_bucket(e, DataKey::OperationTotals(entry.operation), &entry)?;

    if let Some(asset) = entry.asset.clone() {
        update_bucket(e, DataKey::AssetTotals(asset), &entry)?;
    }

    emit_accounting_event(e, &entry);
    bump_instance_ttl(e);
    Ok(())
}

/// Read the protocol-wide aggregate totals.
pub fn get_total_usage(e: &Env) -> ResourceTotals {
    get_bucket(e, &DataKey::AccountingTotals)
}

/// Read aggregate totals for a high-level accounting category.
pub fn get_category_usage(e: &Env, category: AccountingCategory) -> ResourceTotals {
    get_bucket(e, &DataKey::CategoryTotals(category))
}

/// Read aggregate totals for a deterministic operation bucket.
pub fn get_operation_usage(e: &Env, operation: AccountingOperation) -> ResourceTotals {
    get_bucket(e, &DataKey::OperationTotals(operation))
}

/// Read aggregate totals for a specific asset address.
pub fn get_asset_usage(e: &Env, asset: &Address) -> ResourceTotals {
    get_bucket(e, &DataKey::AssetTotals(asset.clone()))
}

/// Generate a deterministic accounting report from the fixed category buckets.
pub fn accounting_report(e: &Env) -> AccountingReport {
    let total = get_total_usage(e);
    let vault = get_category_usage(e, AccountingCategory::Vault);
    let rewards = get_category_usage(e, AccountingCategory::Rewards);
    let treasury = get_category_usage(e, AccountingCategory::Treasury);
    let governance = get_category_usage(e, AccountingCategory::Governance);
    let consistent = totals_equal(total, sum_categories(vault, rewards, treasury, governance));
    let deterministic_checksum = checksum(total, vault, rewards, treasury, governance);

    AccountingReport {
        event_version: EVENT_VERSION,
        generated_at: e.ledger().timestamp(),
        ledger: e.ledger().sequence(),
        total,
        vault,
        rewards,
        treasury,
        governance,
        deterministic_checksum,
        consistent,
    }
}

/// Validate that the protocol total exactly equals the sum of all fixed
/// category buckets. This gives auditors a cheap consistency check.
pub fn validate_accounting(e: &Env) -> bool {
    accounting_report(e).consistent
}

fn validate_entry(entry: &AccountingEntry) -> Result<(), AccountingError> {
    if entry.amount_in < 0 || entry.amount_out < 0 || entry.amount_processed < 0 {
        return Err(AccountingError::NegativeAmount);
    }
    Ok(())
}

fn get_bucket(e: &Env, key: &DataKey) -> ResourceTotals {
    let totals = e
        .storage()
        .persistent()
        .get::<_, ResourceTotals>(key)
        .unwrap_or_else(ResourceTotals::zero);
    if totals.operation_count > 0 {
        bump_persistent_ttl(e, key);
    }
    totals
}

fn update_bucket(e: &Env, key: DataKey, entry: &AccountingEntry) -> Result<(), AccountingError> {
    let mut totals = get_bucket(e, &key);
    apply_entry(&mut totals, entry)?;
    e.storage().persistent().set(&key, &totals);
    bump_persistent_ttl(e, &key);
    Ok(())
}

fn apply_entry(totals: &mut ResourceTotals, entry: &AccountingEntry) -> Result<(), AccountingError> {
    totals.operation_count = checked_add_u64(totals.operation_count, 1)?;
    totals.amount_in = checked_add_i128(totals.amount_in, entry.amount_in)?;
    totals.amount_out = checked_add_i128(totals.amount_out, entry.amount_out)?;
    totals.amount_processed = checked_add_i128(totals.amount_processed, entry.amount_processed)?;
    totals.net_amount = checked_sub_i128(totals.amount_in, totals.amount_out)?;
    totals.storage_reads = checked_add_u64(
        totals.storage_reads,
        entry.resources.storage_reads as u64,
    )?;
    totals.storage_writes = checked_add_u64(
        totals.storage_writes,
        entry.resources.storage_writes as u64,
    )?;
    totals.events_emitted = checked_add_u64(
        totals.events_emitted,
        entry.resources.events_emitted as u64,
    )?;
    totals.token_transfers = checked_add_u64(
        totals.token_transfers,
        entry.resources.token_transfers as u64,
    )?;
    Ok(())
}

fn sum_categories(
    vault: ResourceTotals,
    rewards: ResourceTotals,
    treasury: ResourceTotals,
    governance: ResourceTotals,
) -> ResourceTotals {
    let mut totals = ResourceTotals::zero();
    // Saturation is used only for report validation. `record_operation` uses
    // checked arithmetic, so persisted buckets cannot overflow through normal
    // protocol paths.
    add_saturating(&mut totals, vault);
    add_saturating(&mut totals, rewards);
    add_saturating(&mut totals, treasury);
    add_saturating(&mut totals, governance);
    totals.net_amount = totals.amount_in.saturating_sub(totals.amount_out);
    totals
}

fn add_saturating(target: &mut ResourceTotals, source: ResourceTotals) {
    target.operation_count = target.operation_count.saturating_add(source.operation_count);
    target.amount_in = target.amount_in.saturating_add(source.amount_in);
    target.amount_out = target.amount_out.saturating_add(source.amount_out);
    target.amount_processed = target.amount_processed.saturating_add(source.amount_processed);
    target.storage_reads = target.storage_reads.saturating_add(source.storage_reads);
    target.storage_writes = target.storage_writes.saturating_add(source.storage_writes);
    target.events_emitted = target.events_emitted.saturating_add(source.events_emitted);
    target.token_transfers = target.token_transfers.saturating_add(source.token_transfers);
}

fn totals_equal(a: ResourceTotals, b: ResourceTotals) -> bool {
    a.operation_count == b.operation_count
        && a.amount_in == b.amount_in
        && a.amount_out == b.amount_out
        && a.amount_processed == b.amount_processed
        && a.net_amount == b.net_amount
        && a.storage_reads == b.storage_reads
        && a.storage_writes == b.storage_writes
        && a.events_emitted == b.events_emitted
        && a.token_transfers == b.token_transfers
}

fn checksum(
    total: ResourceTotals,
    vault: ResourceTotals,
    rewards: ResourceTotals,
    treasury: ResourceTotals,
    governance: ResourceTotals,
) -> i128 {
    // A deterministic, order-independent checksum used to prove that report
    // generation is stable for the same persisted buckets.
    totals_checksum(total, 1)
        .saturating_add(totals_checksum(vault, 3))
        .saturating_add(totals_checksum(rewards, 5))
        .saturating_add(totals_checksum(treasury, 7))
        .saturating_add(totals_checksum(governance, 11))
}

fn totals_checksum(totals: ResourceTotals, weight: i128) -> i128 {
    (totals.operation_count as i128)
        .saturating_mul(weight)
        .saturating_add(totals.amount_in.saturating_mul(weight + 1))
        .saturating_add(totals.amount_out.saturating_mul(weight + 2))
        .saturating_add(totals.amount_processed.saturating_mul(weight + 3))
        .saturating_add(totals.net_amount.saturating_mul(weight + 4))
        .saturating_add((totals.storage_reads as i128).saturating_mul(weight + 5))
        .saturating_add((totals.storage_writes as i128).saturating_mul(weight + 6))
        .saturating_add((totals.events_emitted as i128).saturating_mul(weight + 7))
        .saturating_add((totals.token_transfers as i128).saturating_mul(weight + 8))
}

fn emit_accounting_event(e: &Env, entry: &AccountingEntry) {
    e.events().publish(
        (PROTOCOL, ACT_ACCOUNTING),
        AccountingEvent {
            event_version: EVENT_VERSION,
            category: entry.category.symbol(),
            operation: entry.operation.symbol(),
            actor: entry.actor.clone(),
            asset: entry.asset.clone(),
            amount_in: entry.amount_in,
            amount_out: entry.amount_out,
            amount_processed: entry.amount_processed,
            storage_reads: entry.resources.storage_reads,
            storage_writes: entry.resources.storage_writes,
            events_emitted: entry.resources.events_emitted,
            token_transfers: entry.resources.token_transfers,
            timestamp: axionvera_events::ledger_timestamp(e),
            ledger: e.ledger().sequence(),
        },
    );
}

fn checked_add_i128(left: i128, right: i128) -> Result<i128, AccountingError> {
    left.checked_add(right).ok_or(AccountingError::Overflow)
}

fn checked_sub_i128(left: i128, right: i128) -> Result<i128, AccountingError> {
    left.checked_sub(right).ok_or(AccountingError::Overflow)
}

fn checked_add_u64(left: u64, right: u64) -> Result<u64, AccountingError> {
    left.checked_add(right).ok_or(AccountingError::Overflow)
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

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};

    #[soroban_sdk::contract]
    pub struct AccountingHarness;

    #[soroban_sdk::contractimpl]
    impl AccountingHarness {
        pub fn noop() {}
    }

    fn entry(
        category: AccountingCategory,
        operation: AccountingOperation,
        asset: Option<Address>,
        amount_in: i128,
        amount_out: i128,
        amount_processed: i128,
    ) -> AccountingEntry {
        AccountingEntry {
            category,
            operation,
            actor: None,
            asset,
            amount_in,
            amount_out,
            amount_processed,
            resources: OperationResources::new(2, 3, 2, 1),
        }
    }

    #[test]
    fn empty_report_is_deterministic_and_consistent() {
        let e = Env::default();
        e.ledger().set_timestamp(42);
        let contract_id = e.register(AccountingHarness, ());

        e.as_contract(&contract_id, || {
            let report = accounting_report(&e);
            assert_eq!(report.total, ResourceTotals::zero());
            assert!(report.consistent);
            assert_eq!(report.generated_at, 42);
        });
    }

    #[test]
    fn records_vault_deposit_across_total_category_operation_and_asset() {
        let e = Env::default();
        let asset = Address::generate(&e);
        let contract_id = e.register(AccountingHarness, ());

        e.as_contract(&contract_id, || {
            record_operation(
                &e,
                entry(
                    AccountingCategory::Vault,
                    AccountingOperation::VaultDeposit,
                    Some(asset.clone()),
                    500,
                    0,
                    500,
                ),
            )
            .unwrap();

            let expected = ResourceTotals {
                operation_count: 1,
                amount_in: 500,
                amount_out: 0,
                amount_processed: 500,
                net_amount: 500,
                storage_reads: 2,
                storage_writes: 3,
                events_emitted: 2,
                token_transfers: 1,
            };

            assert_eq!(get_total_usage(&e), expected);
            assert_eq!(get_category_usage(&e, AccountingCategory::Vault), expected);
            assert_eq!(get_operation_usage(&e, AccountingOperation::VaultDeposit), expected);
            assert_eq!(get_asset_usage(&e, &asset), expected);
            assert!(validate_accounting(&e));
        });
    }

    #[test]
    fn aggregates_rewards_treasury_and_governance_deterministically() {
        let e = Env::default();
        let reward_asset = Address::generate(&e);
        let contract_id = e.register(AccountingHarness, ());

        e.as_contract(&contract_id, || {
            record_operation(
                &e,
                entry(
                    AccountingCategory::Rewards,
                    AccountingOperation::RewardDistribute,
                    Some(reward_asset.clone()),
                    1_000,
                    0,
                    1_000,
                ),
            )
            .unwrap();
            record_operation(
                &e,
                entry(
                    AccountingCategory::Rewards,
                    AccountingOperation::RewardClaim,
                    Some(reward_asset),
                    0,
                    250,
                    250,
                ),
            )
            .unwrap();
            record_operation(
                &e,
                entry(
                    AccountingCategory::Treasury,
                    AccountingOperation::TreasuryPenalty,
                    None,
                    25,
                    0,
                    25,
                ),
            )
            .unwrap();
            record_operation(
                &e,
                entry(
                    AccountingCategory::Governance,
                    AccountingOperation::GovernancePause,
                    None,
                    0,
                    0,
                    0,
                ),
            )
            .unwrap();

            let report = accounting_report(&e);
            assert!(report.consistent);
            assert_eq!(report.total.operation_count, 4);
            assert_eq!(report.rewards.amount_in, 1_000);
            assert_eq!(report.rewards.amount_out, 250);
            assert_eq!(report.rewards.net_amount, 750);
            assert_eq!(report.treasury.amount_in, 25);
            assert_eq!(report.governance.operation_count, 1);

            let same_report = accounting_report(&e);
            assert_eq!(report.deterministic_checksum, same_report.deterministic_checksum);
        });
    }

    #[test]
    fn rejects_negative_amounts() {
        let e = Env::default();
        let result = record_operation(
            &e,
            entry(
                AccountingCategory::Vault,
                AccountingOperation::VaultDeposit,
                None,
                -1,
                0,
                0,
            ),
        );
        assert_eq!(result, Err(AccountingError::NegativeAmount));
    }
}
