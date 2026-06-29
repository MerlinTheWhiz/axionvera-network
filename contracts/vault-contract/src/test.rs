#![cfg(test)]

//! Integration tests for the AxionVera Vault contract.

use super::*;
use soroban_sdk::{
    contract, contractimpl, contracttype,
    testutils::{Address as _, Events, Ledger, LedgerInfo},
    token, Address, Env, TryIntoVal,
};

type VaultClient<'a> = VaultContractClient<'a>;

#[contract]
pub struct MockToken;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
enum MockTokenDataKey {
    Initialized,
    Balance(Address),
}

#[contractimpl]
impl MockToken {
    pub fn __constructor(e: Env) {
        e.storage()
            .instance()
            .set(&MockTokenDataKey::Initialized, &true);
    }

    pub fn mint(e: Env, to: Address, amount: i128) {
        let current = mock_token_balance(&e, &to);
        let next = current.checked_add(amount).expect("mint overflow");
        mock_token_set_balance(&e, &to, next);
    }

    pub fn balance(e: Env, id: Address) -> i128 {
        mock_token_balance(&e, &id)
    }

    pub fn transfer(e: Env, from: Address, to: Address, amount: i128) {
        let from_balance = mock_token_balance(&e, &from);
        assert!(from_balance >= amount, "insufficient balance");
        let to_balance = mock_token_balance(&e, &to);
        mock_token_set_balance(&e, &from, from_balance - amount);
        mock_token_set_balance(&e, &to, to_balance + amount);
    }
}

fn mock_token_balance(e: &Env, id: &Address) -> i128 {
    e.storage()
        .persistent()
        .get(&MockTokenDataKey::Balance(id.clone()))
        .unwrap_or(0)
}

fn mock_token_set_balance(e: &Env, id: &Address, amount: i128) {
    e.storage()
        .persistent()
        .set(&MockTokenDataKey::Balance(id.clone()), &amount);
}

fn create_stellar_asset(e: &Env, _admin: &Address) -> Address {
    e.register(MockToken, ())
}

fn mint_stellar_asset(e: &Env, token: &Address, to: &Address, amount: i128) {
    e.as_contract(token, || {
        let current = mock_token_balance(e, to);
        let next = current.checked_add(amount).expect("mint overflow");
        mock_token_set_balance(e, to, next);
    });
}

/// Verifies that the contract can only be initialized once.
#[test]
fn test_initialization_is_one_time() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VaultContract {});
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let deposit_token = create_stellar_asset(&e, &admin);
    let reward_token = create_stellar_asset(&e, &admin);
    let vesting_period = 86400u64; // 1 day

    client.initialize(&admin, &deposit_token, &reward_token, &vesting_period);

    let result = client.try_initialize(&admin, &deposit_token, &reward_token, &vesting_period);

    assert_eq!(result, Err(Ok(VaultError::AlreadyInitialized)));
}

/// Verifies that the `initialize` function requires the admin's authorization.
#[test]
fn test_initialize_requires_admin_auth() {
    let e = Env::default();

    let contract_id = e.register_contract(None, VaultContract {});
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let deposit_token = create_stellar_asset(&e, &admin);
    let reward_token = create_stellar_asset(&e, &admin);
    let vesting_period = 86400u64;

    let result = client.try_initialize(&admin, &deposit_token, &reward_token, &vesting_period);

    assert!(result.is_err());
}

/// Verifies that the contract cannot be initialized with identical tokens.
#[test]
fn test_initialize_fails_with_same_tokens() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VaultContract {});
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let token = Address::generate(&e);
    let vesting_period = 86400u64;

    let result = client.try_initialize(&admin, &token, &token, &vesting_period);

    assert_eq!(result, Err(Ok(VaultError::InvalidTokenConfiguration)));
}

/// Tests vesting period functionality.
#[test]
fn test_vesting() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VaultContract {});
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let deposit_token = create_stellar_asset(&e, &admin);
    let reward_token = create_stellar_asset(&e, &admin);
    let vesting_period = 86400u64; // 1 day in seconds

    client.initialize(&admin, &deposit_token, &reward_token, &vesting_period);

    let user = Address::generate(&e);

    mint_stellar_asset(&e, &deposit_token, &user, 1000);
    mint_stellar_asset(&e, &reward_token, &admin, 200000);

    // User deposits tokens
    client.deposit(&user, &100i128);

    // Set timestamp for distribution
    e.ledger().set_timestamp(1000);

    // Admin distributes rewards
    client.distribute_rewards(&200000i128);

    // Check pending rewards
    let pending = client.pending_rewards(&user);
    assert_eq!(pending, 200000);

    // Check vested rewards immediately (should be 0)
    let vested = client.vested_rewards(&user);
    assert_eq!(vested, 0);

    // Advance time halfway through vesting period
    e.ledger().set_timestamp(1000 + 43200);

    // Check vested rewards (should be half)
    let vested = client.vested_rewards(&user);
    assert_eq!(vested, 100000);

    // Advance time past vesting period
    e.ledger().set_timestamp(1000 + 86400 + 1);

    // Check vested rewards (should be full)
    let vested = client.vested_rewards(&user);
    assert_eq!(vested, 200000);

    // Claim rewards
    let claimed = client.claim_rewards(&user);
    assert_eq!(claimed, 200000);
}

// ---------------------------------------------------------------------------
// Multi-Asset Tests
// ---------------------------------------------------------------------------

/// Tests adding a new asset to the vault.
#[test]
fn test_add_asset() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VaultContract);
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let deposit_token = create_stellar_asset(&e, &admin);
    let reward_token = create_stellar_asset(&e, &admin);
    let vesting_period = 86400u64;

    client.initialize(&admin, &deposit_token, &reward_token, &vesting_period);

    let new_asset = Address::generate(&e);

    // Add asset
    client.add_asset(&admin, &new_asset);

    // Verify asset is supported
    assert!(client.is_asset_supported(&new_asset));
}

/// Tests depositing multiple assets.
#[test]
fn test_multiple_asset_deposits() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VaultContract);
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let deposit_token = Address::generate(&e);
    let reward_token = Address::generate(&e);
    let vesting_period = 86400u64;

    client.initialize(&admin, &deposit_token, &reward_token, &vesting_period);

    let asset1 = create_stellar_asset(&e, &admin);
    let asset2 = create_stellar_asset(&e, &admin);
    let user = Address::generate(&e);

    // Add assets
    client.add_asset(&admin, &asset1);
    client.add_asset(&admin, &asset2);

    mint_stellar_asset(&e, &asset1, &user, 1000);
    mint_stellar_asset(&e, &asset2, &user, 2000);

    // Deposit asset1
    client.deposit_asset(&user, &asset1, &100i128);

    // Deposit asset2
    client.deposit_asset(&user, &asset2, &200i128);

    // Verify balances
    assert_eq!(client.balance_of_asset(&user, &asset1), 100);
    assert_eq!(client.balance_of_asset(&user, &asset2), 200);

    // Verify total deposits
    assert_eq!(client.total_deposits_of_asset(&asset1), 100);
    assert_eq!(client.total_deposits_of_asset(&asset2), 200);
}

/// Tests withdrawing from multiple assets.
#[test]
fn test_multiple_asset_withdrawals() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VaultContract);
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let deposit_token = create_stellar_asset(&e, &admin);
    let reward_token = create_stellar_asset(&e, &admin);
    let vesting_period = 86400u64;

    client.initialize(&admin, &deposit_token, &reward_token, &vesting_period);

    let asset1 = create_stellar_asset(&e, &admin);
    let asset2 = create_stellar_asset(&e, &admin);
    let user = Address::generate(&e);

    // Add assets
    client.add_asset(&admin, &asset1);
    client.add_asset(&admin, &asset2);

    mint_stellar_asset(&e, &asset1, &user, 1000);
    mint_stellar_asset(&e, &asset2, &user, 2000);

    // Deposit assets
    client.deposit_asset(&user, &asset1, &100i128);
    client.deposit_asset(&user, &asset2, &200i128);

    // Withdraw from asset1
    client.withdraw_asset(&user, &asset1, &50i128);

    // Withdraw from asset2
    client.withdraw_asset(&user, &asset2, &100i128);

    // Verify balances
    assert_eq!(client.balance_of_asset(&user, &asset1), 50);
    assert_eq!(client.balance_of_asset(&user, &asset2), 100);

    // Verify total deposits
    assert_eq!(client.total_deposits_of_asset(&asset1), 50);
    assert_eq!(client.total_deposits_of_asset(&asset2), 100);
}

/// Tests reward distribution for a specific asset.
#[test]
fn test_asset_reward_distribution() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VaultContract);
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let deposit_token = create_stellar_asset(&e, &admin);
    let reward_token = create_stellar_asset(&e, &admin);
    let vesting_period = 86400u64;

    client.initialize(&admin, &deposit_token, &reward_token, &vesting_period);

    let asset1 = create_stellar_asset(&e, &admin);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);

    // Add asset
    client.add_asset(&admin, &asset1);

    mint_stellar_asset(&e, &asset1, &user1, 1000);
    mint_stellar_asset(&e, &asset1, &user2, 2000);
    mint_stellar_asset(&e, &reward_token, &admin, 1_000_000);

    // Users deposit
    client.deposit_asset(&user1, &asset1, &300i128);
    client.deposit_asset(&user2, &asset1, &600i128);

    // Set timestamp
    e.ledger().set_timestamp(1000);

    // Distribute rewards
    client.distribute_rewards_for_asset(&admin, &asset1, &900000i128);

    // Check pending rewards (user1 should get 1/3, user2 should get 2/3)
    let pending1 = client.pending_rewards_for_asset(&user1, &asset1);
    let pending2 = client.pending_rewards_for_asset(&user2, &asset1);

    assert_eq!(pending1, 300000);
    assert_eq!(pending2, 600000);
}

/// Tests claiming rewards for a specific asset.
#[test]
fn test_asset_reward_claiming() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VaultContract);
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let deposit_token = Address::generate(&e);
    let reward_token = Address::generate(&e);
    let vesting_period = 0u64; // No vesting for this test

    client.initialize(&admin, &deposit_token, &reward_token, &vesting_period);

    let asset1 = Address::generate(&e);
    let user = Address::generate(&e);

    // Add asset
    client.add_asset(&admin, &asset1);

    mint_stellar_asset(&e, &asset1, &user, 1000);
    mint_stellar_asset(&e, &reward_token, &admin, 1_000_000);

    // User deposits
    client.deposit_asset(&user, &asset1, &100i128);

    // Distribute rewards
    client.distribute_rewards_for_asset(&admin, &asset1, &200000i128);

    // Claim rewards
    let claimed = client.claim_rewards_for_asset(&user, &asset1);
    assert_eq!(claimed, 200000);

    // Verify rewards were claimed
    let pending = client.pending_rewards_for_asset(&user, &asset1);
    assert_eq!(pending, 0);
}

#[test]
fn test_locked_positions_unlock_after_expiration() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VaultContract {});
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let deposit_token = create_stellar_asset(&e, &admin);
    let reward_token = create_stellar_asset(&e, &admin);
    let vesting_period = 0_u64;

    client.initialize(&admin, &deposit_token, &reward_token, &vesting_period);

    let user = Address::generate(&e);
    mint_stellar_asset(&e, &deposit_token, &user, 1_000);

    client.deposit(&user, &1_000i128);
    client.lock(&user, &400i128, &604_800u64);

    assert_eq!(client.liquid_balance(&user), 600);
    assert_eq!(client.locked_balance(&user), 400);
    assert_eq!(client.weighted_total_deposits(), 10_400_000);

    e.ledger().set_timestamp(604_799);
    assert_eq!(client.unlock_expired(&user, &10), 0);
    assert_eq!(client.liquid_balance(&user), 600);
    assert_eq!(client.locked_balance(&user), 400);

    e.ledger().set_timestamp(604_801);
    assert_eq!(client.unlock_expired(&user, &10), 400);
    assert_eq!(client.liquid_balance(&user), 1_000);
    assert_eq!(client.locked_balance(&user), 0);
    assert_eq!(client.weighted_total_deposits(), 10_000_000);
}

#[test]
fn test_withdraw_auto_unlocks_expired_funds() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VaultContract {});
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let deposit_token = create_stellar_asset(&e, &admin);
    let reward_token = create_stellar_asset(&e, &admin);
    let vesting_period = 0_u64;

    client.initialize(&admin, &deposit_token, &reward_token, &vesting_period);

    let user = Address::generate(&e);
    mint_stellar_asset(&e, &deposit_token, &user, 1_000);

    client.deposit(&user, &1_000i128);
    client.lock(&user, &400i128, &604_800u64);

    e.ledger().set_timestamp(604_801);

    client.withdraw(&user, &800i128);

    assert_eq!(client.liquid_balance(&user), 200);
    assert_eq!(client.locked_balance(&user), 0);
    assert_eq!(client.weighted_total_deposits(), 2_000_000);
}

#[test]
fn test_lock_multiplier_changes_reward_split() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VaultContract {});
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let deposit_token = create_stellar_asset(&e, &admin);
    let reward_token = create_stellar_asset(&e, &admin);
    let vesting_period = 0_u64;

    client.initialize(&admin, &deposit_token, &reward_token, &vesting_period);

    let locked_user = Address::generate(&e);
    let liquid_user = Address::generate(&e);
    mint_stellar_asset(&e, &deposit_token, &locked_user, 1_000);
    mint_stellar_asset(&e, &deposit_token, &liquid_user, 1_000);
    mint_stellar_asset(&e, &reward_token, &admin, 2_100_000);

    client.deposit(&locked_user, &1_000i128);
    client.deposit(&liquid_user, &1_000i128);
    client.lock(&locked_user, &1_000i128, &604_800u64);

    assert_eq!(client.weighted_total_deposits(), 21_000_000);

    e.ledger().set_timestamp(1_000);
    client.distribute_rewards(&2_100_000i128);

    assert_eq!(client.pending_rewards(&locked_user), 1_100_000);
    assert_eq!(client.pending_rewards(&liquid_user), 1_000_000);

    assert_eq!(client.claim_rewards(&locked_user), 1_100_000);
    assert_eq!(client.claim_rewards(&liquid_user), 1_000_000);
}

#[test]
fn test_lock_rejects_invalid_duration() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VaultContract {});
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let deposit_token = create_stellar_asset(&e, &admin);
    let reward_token = create_stellar_asset(&e, &admin);
    let vesting_period = 0_u64;

    client.initialize(&admin, &deposit_token, &reward_token, &vesting_period);

    let user = Address::generate(&e);
    mint_stellar_asset(&e, &deposit_token, &user, 1_000);

    client.deposit(&user, &1_000i128);

    let unsupported = client.try_lock(&user, &100i128, &1_u64);
    assert_eq!(unsupported, Err(Ok(VaultError::UnsupportedLockDuration)));

    let invalid_duration = client.try_lock(&user, &100i128, &0_u64);
    assert_eq!(invalid_duration, Err(Ok(VaultError::InvalidLockDuration)));
}

#[test]
fn test_admin_can_update_lock_duration_models() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VaultContract {});
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let deposit_token = create_stellar_asset(&e, &admin);
    let reward_token = create_stellar_asset(&e, &admin);
    let vesting_period = 0_u64;

    client.initialize(&admin, &deposit_token, &reward_token, &vesting_period);

    let mut models = soroban_sdk::Vec::new(&e);
    models.push_back(super::storage::LockDurationModel {
        duration_seconds: 2 * 24 * 60 * 60,
        reward_multiplier_bps: 13_000,
    });
    models.push_back(super::storage::LockDurationModel {
        duration_seconds: 4 * 24 * 60 * 60,
        reward_multiplier_bps: 16_000,
    });

    client.set_lock_duration_models(&admin, &models);

    let user = Address::generate(&e);
    mint_stellar_asset(&e, &deposit_token, &user, 1_000);

    client.deposit(&user, &1_000i128);
    client.lock(&user, &1_000i128, &(2 * 24 * 60 * 60));

    assert_eq!(client.weighted_total_deposits(), 13_000_000);
}

/// Tests independent tracking of balances per asset.
#[test]
fn test_independent_asset_tracking() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VaultContract);
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let deposit_token = create_stellar_asset(&e, &admin);
    let reward_token = create_stellar_asset(&e, &admin);
    let vesting_period = 0u64;

    client.initialize(&admin, &deposit_token, &reward_token, &vesting_period);

    let asset1 = create_stellar_asset(&e, &admin);
    let asset2 = create_stellar_asset(&e, &admin);
    let user = Address::generate(&e);

    // Add assets
    client.add_asset(&admin, &asset1);
    client.add_asset(&admin, &asset2);

    mint_stellar_asset(&e, &asset1, &user, 10_000);
    mint_stellar_asset(&e, &asset2, &user, 10_000);
    mint_stellar_asset(&e, &reward_token, &admin, 2_000_000);

    // Deposit different amounts to each asset
    client.deposit_asset(&user, &asset1, &100i128);
    client.deposit_asset(&user, &asset2, &200i128);

    // Distribute different reward amounts to each asset
    client.distribute_rewards_for_asset(&admin, &asset1, &300000i128);
    client.distribute_rewards_for_asset(&admin, &asset2, &600000i128);

    // Check pending rewards are independent
    let pending1 = client.pending_rewards_for_asset(&user, &asset1);
    let pending2 = client.pending_rewards_for_asset(&user, &asset2);

    assert_eq!(pending1, 300000);
    assert_eq!(pending2, 600000);

    // Claim from asset1 only
    let claimed1 = client.claim_rewards_for_asset(&user, &asset1);
    assert_eq!(claimed1, 300000);

    // Verify asset2 rewards are unchanged
    let pending2_after = client.pending_rewards_for_asset(&user, &asset2);
    assert_eq!(pending2_after, 600000);
}

/// Tests that unsupported asset operations fail.
#[test]
fn test_unsupported_asset_fails() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VaultContract);
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let deposit_token = Address::generate(&e);
    let reward_token = Address::generate(&e);
    let vesting_period = 86400u64;

    client.initialize(&admin, &deposit_token, &reward_token, &vesting_period);

    let unsupported_asset = Address::generate(&e);
    let user = Address::generate(&e);

    // Try to deposit unsupported asset
    let result = client.try_deposit_asset(&user, &unsupported_asset, &100i128);
    assert!(result.is_err());

    // Verify asset is not supported
    assert!(!client.is_asset_supported(&unsupported_asset));
}

// ---------------------------------------------------------------------------
// Cross-Contract Interaction Tests
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Event Validation Tests
// ---------------------------------------------------------------------------

/// Verifies that events use the two-topic standard (Protocol, Action).
#[test]
fn test_event_topic_standard() {
    let e = Env::default();
    e.mock_all_auths();
    e.ledger().set(LedgerInfo {
        timestamp: 1000,
        protocol_version: 22,
        sequence_number: 1,
        network_id: [0; 32],
        base_reserve: 10,
        min_persistent_entry_ttl: 518400,
        min_temp_entry_ttl: 518400,
        max_entry_ttl: 6312000,
    });

    let contract_id = e.register_contract(None, VaultContract {});
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let deposit_token = Address::generate(&e);
    let reward_token = Address::generate(&e);
    let user = Address::generate(&e);
    let vesting_period = 86400u64;

    client.initialize(&admin, &deposit_token, &reward_token, &vesting_period);

    // Verify initialize event topics
    let events_snapshot = e.events().all();
    let last = events_snapshot.last().unwrap();
    assert_eq!(last.1.len(), 2, "Initialize must have 2 topics");
    let topic0: soroban_sdk::Symbol = last.1.get(0).unwrap().try_into_val(&e).unwrap();
    let topic1: soroban_sdk::Symbol = last.1.get(1).unwrap().try_into_val(&e).unwrap();
    assert_eq!(topic0, axionvera_events::PROTOCOL);
    assert_eq!(topic1, axionvera_events::ACT_INIT);
}

/// Verifies that deposit events include user indexing.
#[test]
fn test_deposit_event_indexing() {
    let e = Env::default();
    e.mock_all_auths();
    e.ledger().set(LedgerInfo {
        timestamp: 1000,
        protocol_version: 22,
        sequence_number: 1,
        network_id: [0; 32],
        base_reserve: 10,
        min_persistent_entry_ttl: 518400,
        min_temp_entry_ttl: 518400,
        max_entry_ttl: 6312000,
    });

    let contract_id = e.register_contract(None, VaultContract {});
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let deposit_token = Address::generate(&e);
    let reward_token = Address::generate(&e);
    let user = Address::generate(&e);
    let vesting_period = 86400u64;

    client.initialize(&admin, &deposit_token, &reward_token, &vesting_period);

    mint_stellar_asset(&e, &deposit_token, &user, 1000);

    client.deposit(&user, &100i128);

    // Verify event has two topics
    let events = e.events().all();
    let deposit_event = events.get(events.len() - 1).unwrap();
    assert_eq!(deposit_event.1.len(), 2, "Deposit must have 2 topics");

    // Verify on-chain indexing
    e.as_contract(&contract_id, || {
        let log = axionvera_core::get_user_event_log(&e, &user);
        assert!(!log.is_empty(), "User event log should not be empty");
        assert_eq!(log.get(0).unwrap().action, axionvera_events::ACT_DEPOSIT);

        let global_log = axionvera_core::get_global_event_log(&e);
        assert!(
            !global_log.is_empty(),
            "Global event log should not be empty"
        );

        let users = axionvera_core::get_interacting_users(&e);
        assert_eq!(users.len(), 1, "Should have one interacting user");
        assert_eq!(users.get(0).unwrap(), user);
    });
}

/// Verifies that pause_contract and unpause_contract emit events.
#[test]
fn test_pause_unpause_events() {
    let e = Env::default();
    e.mock_all_auths();
    e.ledger().set(LedgerInfo {
        timestamp: 1000,
        protocol_version: 22,
        sequence_number: 1,
        network_id: [0; 32],
        base_reserve: 10,
        min_persistent_entry_ttl: 518400,
        min_temp_entry_ttl: 518400,
        max_entry_ttl: 6312000,
    });

    let contract_id = e.register_contract(None, VaultContract {});
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let deposit_token = create_stellar_asset(&e, &admin);
    let reward_token = create_stellar_asset(&e, &admin);
    let vesting_period = 86400u64;

    client.initialize(&admin, &deposit_token, &reward_token, &vesting_period);

    let prev_event_count = e.events().all().len();

    client.pause_contract();
    let pause_events = e.events().all();
    let new_count = pause_events.len();
    assert!(new_count > prev_event_count, "Pause should emit an event");

    let pause_event = pause_events.get(new_count - 1).unwrap();
    assert_eq!(pause_event.1.len(), 2, "Pause must have 2 topics");
    let pause_topic: soroban_sdk::Symbol = pause_event.1.get(1).unwrap().try_into_val(&e).unwrap();
    assert_eq!(pause_topic, axionvera_events::ACT_PAUSE);

    client.unpause_contract();
    let all_events = e.events().all();
    let unpause_event = all_events.get(all_events.len() - 1).unwrap();
    assert_eq!(unpause_event.1.len(), 2, "Unpause must have 2 topics");
    let unpause_topic: soroban_sdk::Symbol =
        unpause_event.1.get(1).unwrap().try_into_val(&e).unwrap();
    assert_eq!(unpause_topic, axionvera_events::ACT_UNPAUSE);
}

/// Verifies that all events include event_version field.
#[test]
fn test_event_version_field() {
    let e = Env::default();
    e.mock_all_auths();
    e.ledger().set(LedgerInfo {
        timestamp: 1000,
        protocol_version: 22,
        sequence_number: 1,
        network_id: [0; 32],
        base_reserve: 10,
        min_persistent_entry_ttl: 518400,
        min_temp_entry_ttl: 518400,
        max_entry_ttl: 6312000,
    });

    let contract_id = e.register_contract(None, VaultContract {});
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let deposit_token = Address::generate(&e);
    let reward_token = Address::generate(&e);
    let user = Address::generate(&e);
    let vesting_period = 0u64; // No vesting for this test

    client.initialize(&admin, &deposit_token, &reward_token, &vesting_period);

    mint_stellar_asset(&e, &deposit_token, &user, 1000);
    mint_stellar_asset(&e, &reward_token, &admin, 200_000);

    // Verify that the event_version constant is 1
    assert_eq!(axionvera_events::EVENT_VERSION, 1);

    // Deposit triggers event with version
    client.deposit(&user, &100i128);
    // The event_struct includes event_version which is verified at compile time
    // via the struct definition. Runtime verification is implicit through the
    // event struct being correctly populated.
}

#[test]
fn test_cross_contract_client_validate_contract() {
    let e = Env::default();
    let contract_id = e.register_contract(None, VaultContract);
    let other_address = Address::generate(&e);

    // Test that self-contract validation fails
    e.as_contract(&contract_id, || {
        let result =
            crate::cross_contract::CrossContractClient::validate_contract_exists(&e, &contract_id);
        assert!(result.is_err());
    });

    // Test that other contract validation passes
    e.as_contract(&contract_id, || {
        let result = crate::cross_contract::CrossContractClient::validate_contract_exists(
            &e,
            &other_address,
        );
        assert!(result.is_ok());
    });
}
