use super::*;

use axionvera_accounting as accounting;
use axionvera_accounting::{AccountingCategory, AccountingOperation, OperationResources};
use axionvera_storage as storage;
use soroban_sdk::{
    contract, contractimpl,
    testutils::{Address as _, Ledger},
    Address, Env,
};

#[contract]
pub struct FeeHarness;

#[contractimpl]
impl FeeHarness {
    pub fn noop() {}
}

#[test]
fn calculates_fee_amount_and_receipt() {
    let e = Env::default();
    e.ledger().set_timestamp(100);
    let actor = Address::generate(&e);
    let treasury = Address::generate(&e);
    let asset = Address::generate(&e);

    let receipt = build_fee_receipt(
        FeeType::Deposit,
        actor.clone(),
        treasury.clone(),
        Some(asset.clone()),
        10_000,
        250,
        e.ledger().timestamp(),
    )
    .unwrap();

    assert_eq!(receipt.fee_amount, 250);
    assert_eq!(receipt.net_amount, 9_750);
    assert_eq!(receipt.treasury_amount, 250);
    assert_eq!(receipt.actor, actor);
    assert_eq!(receipt.treasury, treasury);
    assert_eq!(receipt.asset, Some(asset));
}

#[test]
fn records_fee_collection_and_updates_accounting() {
    let e = Env::default();
    e.ledger().set_timestamp(250);
    let contract_id = e.register(FeeHarness, ());
    let actor = Address::generate(&e);
    let treasury = Address::generate(&e);
    let asset = Address::generate(&e);

    e.as_contract(&contract_id, || {
        let config = FeeConfig {
            treasury: treasury.clone(),
            deposit_fee_bps: 250,
            withdrawal_fee_bps: 125,
            reward_fee_bps: 75,
        };
        let receipt = collect_fee(
            &e,
            FeeType::Deposit,
            actor.clone(),
            Some(asset.clone()),
            20_000,
            &config,
            OperationResources::new(1, 1, 1, 1),
        )
        .unwrap();

        assert_eq!(receipt.fee_amount, 500);
        assert_eq!(receipt.net_amount, 19_500);

        let totals = storage::get_fee_totals(&e, FeeType::Deposit);
        assert_eq!(totals.operation_count, 1);
        assert_eq!(totals.collected_amount, 500);
        assert_eq!(totals.treasury_amount, 500);

        let accounting = accounting::get_category_usage(&e, AccountingCategory::Treasury);
        assert_eq!(accounting.operation_count, 1);
        assert_eq!(accounting.amount_in, 500);
        assert_eq!(
            accounting::get_operation_usage(&e, AccountingOperation::TreasuryDepositFee).operation_count,
            1
        );
    });
}

#[test]
fn zero_fee_does_not_mutate_totals() {
    let e = Env::default();
    let contract_id = e.register(FeeHarness, ());
    let actor = Address::generate(&e);
    let treasury = Address::generate(&e);

    e.as_contract(&contract_id, || {
        let config = FeeConfig {
            treasury,
            deposit_fee_bps: 0,
            withdrawal_fee_bps: 0,
            reward_fee_bps: 0,
        };

        let receipt = collect_fee(
            &e,
            FeeType::Reward,
            actor,
            None,
            1000,
            &config,
            OperationResources::new(1, 1, 1, 1),
        )
        .unwrap();

        assert_eq!(receipt.fee_amount, 0);
        let totals = storage::get_fee_totals(&e, FeeType::Reward);
        assert_eq!(totals.operation_count, 0);
        assert_eq!(totals.collected_amount, 0);
        assert_eq!(totals.treasury_amount, 0);
    });
}
