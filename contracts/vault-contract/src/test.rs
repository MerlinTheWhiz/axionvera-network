#![cfg(test)]

//! Integration tests for the AxionVera Vault contract.

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    token, Address, Env,
};

type VaultClient<'a> = VaultContractClient<'a>;

/// Verifies that the contract can only be initialized once.
#[test]
fn test_initialization_is_one_time() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VaultContract {});
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let deposit_token = Address::generate(&e);
    let reward_token = Address::generate(&e);
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
    let deposit_token = Address::generate(&e);
    let reward_token = Address::generate(&e);
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
    let deposit_token = Address::generate(&e);
    let reward_token = Address::generate(&e);
    let vesting_period = 86400u64; // 1 day in seconds

    client.initialize(&admin, &deposit_token, &reward_token, &vesting_period);

    let user = Address::generate(&e);

    // Set up mock token clients
    let _deposit_token_client = token::Client::new(&e, &deposit_token);
    let _reward_token_client = token::Client::new(&e, &reward_token);

    // Mock token balances
    e.as_contract(&deposit_token, || {
        e.storage()
            .instance()
            .set(&token::DataKey::Admin, &admin);
        e.storage()
            .instance()
            .set(&token::DataKey::Balance(user.clone()), &1000i128);
        e.storage()
            .instance()
            .set(&token::DataKey::Balance(contract_id.clone()), &0i128);
    });
    e.as_contract(&reward_token, || {
        e.storage().instance().set(&token::DataKey::Admin, &admin);
        e.storage().instance().set(&token::DataKey::Balance(admin.clone()), &200000i128);
        e.storage().instance().set(&token::DataKey::Balance(contract_id.clone()), &0i128);
    });

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

#[cfg(test)]
mod lock_tests {
    use super::*;

    fn setup_test(e: &Env) -> (VaultClient, Address, Address, token::Client) {
        e.mock_all_auths();

        let contract_id = e.register_contract(None, VaultContract {});
        let client = VaultContractClient::new(e, &contract_id);

        let admin = Address::generate(e);
        let deposit_token_id = e.register_stellar_asset_contract(Address::generate(e));
        let reward_token_id = e.register_stellar_asset_contract(Address::generate(e));

        let deposit_token = token::Client::new(e, &deposit_token_id);

        client.initialize(&admin, &deposit_token_id, &reward_token_id, &0);

        (client, admin, deposit_token_id, deposit_token)
    }

    #[test]
    fn test_lock_and_early_withdraw_fails() {
        let e = Env::default();
        let (client, _admin, _deposit_token_id, deposit_token) = setup_test(&e);
        let user = Address::generate(&e);

        deposit_token.mint(&user, &1000);
        client.deposit(&user, &1000);

        assert_eq!(client.liquid_balance(&user), 1000);
        assert_eq!(client.locked_balance(&user), 0);
        assert_eq!(client.balance(&user), 1000);

        // Lock 600 tokens for 1 day
        let lock_duration = 86400;
        client.lock(&user, &600, &lock_duration);

        assert_eq!(client.liquid_balance(&user), 400);
        assert_eq!(client.locked_balance(&user), 600);
        assert_eq!(client.balance(&user), 1000);

        // Attempt to withdraw more than liquid balance fails
        let res = client.try_withdraw(&user, &500);
        assert_eq!(res, Err(Ok(VaultError::InsufficientBalance)));

        // Withdraw liquid balance successfully
        client.withdraw(&user, &400);
        assert_eq!(client.liquid_balance(&user), 0);
        assert_eq!(client.locked_balance(&user), 600);
        assert_eq!(client.balance(&user), 600);
    }

    #[test]
    fn test_unlock_after_expiry() {
        let e = Env::default();
        let (client, _admin, _deposit_token_id, deposit_token) = setup_test(&e);
        let user = Address::generate(&e);

        deposit_token.mint(&user, &1000);
        client.deposit(&user, &1000);

        let lock_duration = 100;
        e.ledger().set(LedgerInfo {
            timestamp: 1000,
            ..e.ledger().get()
        });
        client.lock(&user, &600, &lock_duration);

        assert_eq!(client.liquid_balance(&user), 400);
        assert_eq!(client.locked_balance(&user), 600);

        // Advance time just before expiry
        e.ledger().set(LedgerInfo {
            timestamp: 1000 + lock_duration - 1,
            ..e.ledger().get()
        });

        // Manual unlock should do nothing
        let unlocked = client.unlock_expired(&user);
        assert_eq!(unlocked, 0);
        assert_eq!(client.liquid_balance(&user), 400);

        // Advance time past expiry
        e.ledger().set(LedgerInfo {
            timestamp: 1000 + lock_duration + 1,
            ..e.ledger().get()
        });

        // Manual unlock now works
        let unlocked = client.unlock_expired(&user);
        assert_eq!(unlocked, 600);

        assert_eq!(client.liquid_balance(&user), 1000);
        assert_eq!(client.locked_balance(&user), 0);
        assert_eq!(client.balance(&user), 1000);

        // Now withdrawal succeeds
        client.withdraw(&user, &1000);
        assert_eq!(client.balance(&user), 0);
    }

    #[test]
    fn test_withdraw_auto_unlocks_expired_locks() {
        let e = Env::default();
        let (client, _admin, _deposit_token_id, deposit_token) = setup_test(&e);
        let user = Address::generate(&e);

        deposit_token.mint(&user, &1000);
        client.deposit(&user, &1000);

        let lock_duration = 100;
        e.ledger().set(LedgerInfo { timestamp: 1000, ..e.ledger().get() });
        client.lock(&user, &600, &lock_duration); // 400 liquid, 600 locked

        // Advance time past expiry
        e.ledger().set(LedgerInfo { timestamp: 1000 + lock_duration + 1, ..e.ledger().get() });

        // Withdraw should now succeed because it auto-unlocks the 600 expired tokens,
        // making the liquid balance 1000.
        client.withdraw(&user, &1000);
        assert_eq!(client.balance(&user), 0);
        assert_eq!(client.liquid_balance(&user), 0);
    }
}
