use super::*;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype,
    testutils::{Address as _, Ledger},
    vec, Address, BytesN, Env, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum TokenError {
    InsufficientBalance = 1,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TokenDataKey {
    Balance(Address),
}

#[contract]
pub struct MockToken;

#[contractimpl]
impl MockToken {
    pub fn mint(e: Env, to: Address, amount: i128) {
        let key = token_balance_key(&e, &to);
        let balance = e.storage().persistent().get(&key).unwrap_or(0_i128);
        e.storage().persistent().set(&key, &(balance + amount));
    }

    pub fn balance(e: Env, id: Address) -> i128 {
        e.storage()
            .persistent()
            .get(&token_balance_key(&e, &id))
            .unwrap_or(0)
    }

    pub fn transfer(e: Env, from: Address, to: Address, amount: i128) -> Result<(), TokenError> {
        let from_key = token_balance_key(&e, &from);
        let to_key = token_balance_key(&e, &to);
        let from_balance = e.storage().persistent().get(&from_key).unwrap_or(0_i128);
        if from_balance < amount {
            return Err(TokenError::InsufficientBalance);
        }
        let to_balance = e.storage().persistent().get(&to_key).unwrap_or(0_i128);
        e.storage()
            .persistent()
            .set(&from_key, &(from_balance - amount));
        e.storage()
            .persistent()
            .set(&to_key, &(to_balance + amount));
        Ok(())
    }
}

fn token_balance_key(_e: &Env, address: &Address) -> TokenDataKey {
    TokenDataKey::Balance(address.clone())
}

fn id(e: &Env, seed: u8) -> BytesN<32> {
    BytesN::from_array(e, &[seed; 32])
}

fn rule(recipient: &Address, share_bps: u32) -> AllocationRule {
    AllocationRule {
        recipient: recipient.clone(),
        share_bps,
    }
}

fn strategy(e: &Env, seed: u8, rules: Vec<AllocationRule>) -> AllocationStrategy {
    AllocationStrategy {
        id: id(e, seed),
        rules,
    }
}

fn setup(
    e: &Env,
) -> (
    TreasuryContractClient<'_>,
    MockTokenClient<'_>,
    Address,
    Address,
    Address,
    Address,
) {
    e.mock_all_auths();
    let treasury_id = e.register(TreasuryContract, ());
    let token_id = e.register(MockToken, ());
    let treasury = TreasuryContractClient::new(e, &treasury_id);
    let token = MockTokenClient::new(e, &token_id);
    let admin = Address::generate(e);
    let recipient_a = Address::generate(e);
    let recipient_b = Address::generate(e);

    treasury.initialize(&admin, &token_id);
    token.mint(&treasury_id, &10_000_i128);

    (
        treasury,
        token,
        admin,
        treasury_id,
        recipient_a,
        recipient_b,
    )
}

#[test]
fn configure_strategy_rejects_invalid_share_total() {
    let e = Env::default();
    let (treasury, _token, admin, _treasury_id, recipient_a, recipient_b) = setup(&e);
    let strategy = strategy(
        &e,
        1,
        vec![&e, rule(&recipient_a, 6_000), rule(&recipient_b, 3_000)],
    );

    let result = treasury.try_configure_strategy(&admin, &strategy);

    assert_eq!(result, Err(Ok(TreasuryError::InvalidShareTotal)));
}

#[test]
fn configure_strategy_rejects_duplicate_recipient() {
    let e = Env::default();
    let (treasury, _token, admin, _treasury_id, recipient_a, _recipient_b) = setup(&e);
    let strategy = strategy(
        &e,
        2,
        vec![&e, rule(&recipient_a, 5_000), rule(&recipient_a, 5_000)],
    );

    let result = treasury.try_configure_strategy(&admin, &strategy);

    assert_eq!(result, Err(Ok(TreasuryError::DuplicateRecipient)));
}

#[test]
fn distributes_by_configured_allocation_rules() {
    let e = Env::default();
    e.ledger().set_timestamp(2_000);
    let (treasury, token, admin, treasury_id, recipient_a, recipient_b) = setup(&e);
    let strategy = strategy(
        &e,
        3,
        vec![&e, rule(&recipient_a, 7_000), rule(&recipient_b, 3_000)],
    );
    treasury.configure_strategy(&admin, &strategy);

    let receipt = treasury.distribute(&admin, &id(&e, 44), &strategy.id, &1_000_i128);

    assert_eq!(token.balance(&treasury_id), 9_000);
    assert_eq!(token.balance(&recipient_a), 700);
    assert_eq!(token.balance(&recipient_b), 300);
    assert_eq!(receipt.total_amount, 1_000);
    assert_eq!(receipt.transfers.len(), 2);
    assert_eq!(receipt.transfers.get(0).unwrap().amount, 700);
    assert_eq!(receipt.transfers.get(1).unwrap().amount, 300);
    assert_eq!(receipt.timestamp, 2_000);
    assert_eq!(
        treasury.distribution_receipt(&receipt.distribution_id),
        Some(receipt)
    );
    assert_eq!(treasury.total_distributed(), 1_000);
    assert_eq!(treasury.recipient_distributed(&recipient_a), 700);
    assert_eq!(treasury.recipient_distributed(&recipient_b), 300);
}

#[test]
fn assigns_rounding_remainder_to_last_rule() {
    let e = Env::default();
    let (treasury, token, admin, _treasury_id, recipient_a, recipient_b) = setup(&e);
    let strategy = strategy(
        &e,
        4,
        vec![&e, rule(&recipient_a, 3_333), rule(&recipient_b, 6_667)],
    );
    treasury.configure_strategy(&admin, &strategy);

    treasury.distribute(&admin, &id(&e, 45), &strategy.id, &101_i128);

    assert_eq!(token.balance(&recipient_a), 33);
    assert_eq!(token.balance(&recipient_b), 68);
}

#[test]
fn rejects_unknown_strategy_duplicate_distribution_and_low_balance() {
    let e = Env::default();
    let (treasury, _token, admin, _treasury_id, recipient_a, recipient_b) = setup(&e);
    let missing = id(&e, 99);

    let missing_result = treasury.try_distribute(&admin, &id(&e, 50), &missing, &100_i128);
    assert_eq!(missing_result, Err(Ok(TreasuryError::StrategyNotFound)));

    let strategy = strategy(
        &e,
        5,
        vec![&e, rule(&recipient_a, 5_000), rule(&recipient_b, 5_000)],
    );
    treasury.configure_strategy(&admin, &strategy);

    treasury.distribute(&admin, &id(&e, 51), &strategy.id, &100_i128);
    let duplicate_result = treasury.try_distribute(&admin, &id(&e, 51), &strategy.id, &100_i128);
    assert_eq!(
        duplicate_result,
        Err(Ok(TreasuryError::DuplicateDistribution))
    );

    let low_balance_result =
        treasury.try_distribute(&admin, &id(&e, 52), &strategy.id, &100_000_i128);
    assert_eq!(
        low_balance_result,
        Err(Ok(TreasuryError::InsufficientBalance))
    );
}

#[test]
fn records_protocol_fee_and_reports_asset_balance() {
    let e = Env::default();
    e.ledger().set_timestamp(3_000);
    let (treasury, token, admin, treasury_id, _recipient_a, _recipient_b) = setup(&e);
    let payer = Address::generate(&e);
    token.mint(&payer, &750_i128);

    let record = treasury.record_fee(&admin, &id(&e, 60), &payer, &250_i128);

    assert_eq!(token.balance(&payer), 500);
    assert_eq!(token.balance(&treasury_id), 10_250);
    assert_eq!(record.amount, 250);
    assert_eq!(record.payer, payer);
    assert_eq!(record.treasury_balance, 10_250);
    assert_eq!(record.timestamp, 3_000);
    assert_eq!(treasury.fee_record(&record.fee_id), Some(record));
    assert_eq!(treasury.total_fees_recorded(), 250);
    assert_eq!(treasury.asset_balance(), 10_250);
}

#[test]
fn rejects_duplicate_fee_and_invalid_fee_amount() {
    let e = Env::default();
    let (treasury, token, admin, _treasury_id, _recipient_a, _recipient_b) = setup(&e);
    let payer = Address::generate(&e);
    token.mint(&payer, &100_i128);

    let invalid = treasury.try_record_fee(&admin, &id(&e, 61), &payer, &0_i128);
    assert_eq!(invalid, Err(Ok(TreasuryError::InvalidAmount)));

    treasury.record_fee(&admin, &id(&e, 62), &payer, &50_i128);
    let duplicate = treasury.try_record_fee(&admin, &id(&e, 62), &payer, &50_i128);
    assert_eq!(duplicate, Err(Ok(TreasuryError::DuplicateFee)));
}
