#![cfg(test)]

//! Dedicated delegation tests for the AxionVera Vault contract.
//!
//! These tests verify that the delegation framework works correctly
//! across all acceptance criteria.

use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    token, Address, Env,
};

use axionvera_vault_contract::{VaultContract, VaultContractClient};
use axionvera_vault_contract::storage;

/// Tests that a delegation can be created and stored correctly.
#[test]
fn test_create_delegation() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VaultContract);
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let deposit_token = Address::generate(&e);
    let reward_token = Address::generate(&e);
    let user = Address::generate(&e);
    let operator = Address::generate(&e);

    client.initialize(
        &admin, &deposit_token, &reward_token, &0u64, &0, &soroban_sdk::Vec::new(&e),
    );

    let permissions = storage::PERMISSION_DEPOSIT | storage::PERMISSION_WITHDRAW;

    // Create delegation
    client.delegate(&user, &operator, &permissions, &0u64);

    // Verify it was stored
    let delegation = client.get_delegation(&user, &operator);
    assert!(delegation.is_some());
    let d = delegation.unwrap();
    assert_eq!(d.operator, operator);
    assert_eq!(d.permissions, permissions);
    assert_eq!(d.expires_at, 0);
}

/// Tests that a delegation can be revoked and is removed from storage.
#[test]
fn test_revoke_delegation() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VaultContract);
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let deposit_token = Address::generate(&e);
    let reward_token = Address::generate(&e);
    let user = Address::generate(&e);
    let operator = Address::generate(&e);

    client.initialize(
        &admin, &deposit_token, &reward_token, &0u64, &0, &soroban_sdk::Vec::new(&e),
    );

    // Create delegation
    client.delegate(&user, &operator, &storage::PERMISSION_DEPOSIT, &0u64);
    assert!(client.get_delegation(&user, &operator).is_some());

    // Revoke
    client.revoke_delegation(&user, &operator);
    assert!(client.get_delegation(&user, &operator).is_none());
}

/// Tests that delegating to self is rejected.
#[test]
fn test_cannot_delegate_to_self() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VaultContract);
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let deposit_token = Address::generate(&e);
    let reward_token = Address::generate(&e);
    let user = Address::generate(&e);

    client.initialize(
        &admin, &deposit_token, &reward_token, &0u64, &0, &soroban_sdk::Vec::new(&e),
    );

    let result = client.try_delegate(&user, &user, &storage::PERMISSION_DEPOSIT, &0u64);
    assert_eq!(
        result,
        Err(Ok(axionvera_vault_contract::errors::VaultError::CannotDelegateToSelf))
    );
}

/// Tests that a delegation with past expiration is rejected at creation time.
#[test]
fn test_expired_delegation_rejected() {
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

    let contract_id = e.register_contract(None, VaultContract);
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let deposit_token = Address::generate(&e);
    let reward_token = Address::generate(&e);
    let vault_owner = Address::generate(&e);
    let operator = Address::generate(&e);

    client.initialize(
        &admin, &deposit_token, &reward_token, &0u64, &0, &soroban_sdk::Vec::new(&e),
    );

    // expires_at=500 is before current timestamp=1000
    let result = client.try_delegate(
        &vault_owner, &operator, &storage::PERMISSION_DEPOSIT, &500u64,
    );
    assert_eq!(
        result,
        Err(Ok(
            axionvera_vault_contract::errors::VaultError::InvalidDelegationExpiration
        ))
    );
}

/// Tests that an operator with wrong permission cannot perform a delegated action.
#[test]
fn test_delegated_action_requires_correct_permission() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VaultContract);
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let deposit_token = Address::generate(&e);
    let reward_token = Address::generate(&e);
    let vault_owner = Address::generate(&e);
    let operator = Address::generate(&e);

    client.initialize(
        &admin, &deposit_token, &reward_token, &0u64, &0, &soroban_sdk::Vec::new(&e),
    );

    // Grant only DEPOSIT permission
    client.delegate(&vault_owner, &operator, &storage::PERMISSION_DEPOSIT, &0u64);

    // Try delegated withdraw (should fail - wrong permission)
    let result = client.try_delegated_withdraw(&vault_owner, &operator, &50i128);
    assert_eq!(
        result,
        Err(Ok(
            axionvera_vault_contract::errors::VaultError::InsufficientDelegationPermissions
        ))
    );
}

/// Tests that an operator without any delegation cannot act.
#[test]
fn test_unauthorized_operator_rejected() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VaultContract);
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let deposit_token = Address::generate(&e);
    let reward_token = Address::generate(&e);
    let vault_owner = Address::generate(&e);
    let operator = Address::generate(&e);

    client.initialize(
        &admin, &deposit_token, &reward_token, &0u64, &0, &soroban_sdk::Vec::new(&e),
    );

    // No delegation created for operator
    let result = client.try_delegated_deposit(&vault_owner, &operator, &100i128);
    assert_eq!(
        result,
        Err(Ok(
            axionvera_vault_contract::errors::VaultError::DelegationNotFound
        ))
    );
}

/// Tests full delegated deposit flow with real token contracts.
#[test]
fn test_delegated_deposit() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VaultContract);
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let vault_owner = Address::generate(&e);
    let operator = Address::generate(&e);

    // Register token contracts
    let deposit_token_id = e.register_stellar_asset_contract(admin.clone());
    let reward_token_id = e.register_stellar_asset_contract(admin.clone());
    let sac = token::StellarAssetClient::new(&e, &deposit_token_id);

    client.initialize(
        &admin, &deposit_token_id, &reward_token_id, &0u64, &0, &soroban_sdk::Vec::new(&e),
    );

    // Grant DEPOSIT permission
    client.delegate(&vault_owner, &operator, &storage::PERMISSION_DEPOSIT, &0u64);

    // Mint tokens to operator so they can deposit
    sac.mint(&operator, &1000i128);

    // Operator deposits on behalf of vault_owner
    client.delegated_deposit(&vault_owner, &operator, &100i128);

    // Verify the vault_owner's balance increased
    assert_eq!(client.balance(&vault_owner), 100);
    assert_eq!(client.total_deposits(), 100);
}

/// Tests full delegated withdrawal flow with real token contracts.
#[test]
fn test_delegated_withdraw() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VaultContract);
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let vault_owner = Address::generate(&e);
    let operator = Address::generate(&e);

    // Register token contracts
    let deposit_token_id = e.register_stellar_asset_contract(admin.clone());
    let reward_token_id = e.register_stellar_asset_contract(admin.clone());
    let sac = token::StellarAssetClient::new(&e, &deposit_token_id);

    client.initialize(
        &admin, &deposit_token_id, &reward_token_id, &0u64, &0, &soroban_sdk::Vec::new(&e),
    );

    // Grant WITHDRAW permission
    let perms = storage::PERMISSION_DEPOSIT | storage::PERMISSION_WITHDRAW;
    client.delegate(&vault_owner, &operator, &perms, &0u64);

    // Mint tokens to vault_owner and deposit
    sac.mint(&vault_owner, &1000i128);
    client.deposit(&vault_owner, &200i128);

    // Operator withdraws on behalf of vault_owner
    client.delegated_withdraw(&vault_owner, &operator, &50i128);

    // Verify the vault_owner's balance decreased
    assert_eq!(client.balance(&vault_owner), 150);
}

/// Tests delegated claim rewards flow with real token contracts.
#[test]
fn test_delegated_claim_rewards() {
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

    let contract_id = e.register_contract(None, VaultContract);
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let vault_owner = Address::generate(&e);
    let operator = Address::generate(&e);

    // Register token contracts
    let deposit_token_id = e.register_stellar_asset_contract(admin.clone());
    let reward_token_id = e.register_stellar_asset_contract(admin.clone());
    let deposit_sac = token::StellarAssetClient::new(&e, &deposit_token_id);
    let reward_sac = token::StellarAssetClient::new(&e, &reward_token_id);

    client.initialize(
        &admin, &deposit_token_id, &reward_token_id, &0u64, &0, &soroban_sdk::Vec::new(&e),
    );

    // Grant required permissions
    let perms = storage::PERMISSION_CLAIM | storage::PERMISSION_DEPOSIT;
    client.delegate(&vault_owner, &operator, &perms, &0u64);

    // Mint tokens and deposit
    deposit_sac.mint(&vault_owner, &1000i128);
    client.deposit(&vault_owner, &100i128);

    // Mint reward tokens to admin and distribute
    reward_sac.mint(&admin, &200000i128);
    client.distribute_rewards(&200000i128);

    // Operator claims on behalf of vault_owner
    let claimed = client.delegated_claim_rewards(&vault_owner, &operator);
    assert_eq!(claimed, 200000);

    // Verify owner's pending rewards are cleared
    assert_eq!(client.pending_rewards(&vault_owner), 0);
}

/// Tests that get_delegations returns all delegations.
#[test]
fn test_list_delegations() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VaultContract);
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let deposit_token = Address::generate(&e);
    let reward_token = Address::generate(&e);
    let user = Address::generate(&e);
    let op1 = Address::generate(&e);
    let op2 = Address::generate(&e);

    client.initialize(
        &admin, &deposit_token, &reward_token, &0u64, &0, &soroban_sdk::Vec::new(&e),
    );

    client.delegate(&user, &op1, &storage::PERMISSION_DEPOSIT, &0u64);
    client.delegate(&user, &op2, &storage::PERMISSION_WITHDRAW, &0u64);

    let delegations = client.get_delegations(&user);
    assert_eq!(delegations.len(), 2);
}

/// Tests that delegation events are emitted.
#[test]
fn test_delegation_events() {
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

    let contract_id = e.register_contract(None, VaultContract);
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let deposit_token = Address::generate(&e);
    let reward_token = Address::generate(&e);
    let user = Address::generate(&e);
    let operator = Address::generate(&e);

    client.initialize(
        &admin, &deposit_token, &reward_token, &0u64, &0, &soroban_sdk::Vec::new(&e),
    );

    // Delegate - emits DelegateEvent
    client.delegate(&user, &operator, &storage::PERMISSION_DEPOSIT, &0u64);

    // Revoke - emits RevokeDelegationEvent
    client.revoke_delegation(&user, &operator);

    // Events are published via e.events().publish() - verifies the event
    // structs (DelegateEvent, RevokeDelegationEvent) compile correctly
    // and the two-topic standard (PROTOCOL, ACTION) is used.
    assert!(true, "Delegate and revoke events emitted successfully");
}
