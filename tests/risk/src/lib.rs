#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, vec, contracttype};
use axionvera_vault_contract::{VaultContract, VaultContractClient};
use axionvera_risk::RiskParameters;

#[contracttype]
enum TokenDataKey {
    Balance(Address),
    Admin,
}

#[soroban_sdk::contract]
pub struct MockToken;

#[soroban_sdk::contractimpl]
impl MockToken {
    pub fn balance(e: Env, id: Address) -> i128 {
        e.storage().instance().get(&TokenDataKey::Balance(id)).unwrap_or(0)
    }

    pub fn transfer(e: Env, from: Address, to: Address, amount: i128) {
        let from_balance = Self::balance(e.clone(), from.clone());
        let to_balance = Self::balance(e.clone(), to.clone());
        e.storage().instance().set(&TokenDataKey::Balance(from), &(from_balance - amount));
        e.storage().instance().set(&TokenDataKey::Balance(to), &(to_balance + amount));
    }
}

#[test]
fn test_vault_enforces_risk_limits() {
    let e = Env::default();
    e.mock_all_auths();

    // Register Vault
    let vault_id = e.register_contract(None, VaultContract);
    let vault_client = VaultContractClient::new(&e, &vault_id);

    let admin = Address::generate(&e);
    let deposit_token = e.register_contract(None, MockToken);
    let reward_token = Address::generate(&e);

    vault_client.initialize(&admin, &deposit_token, &reward_token, &0u64, &0, &vec![&e]);

    let risk_params = RiskParameters {
        max_deposit_amount: 1000,
        min_deposit_amount: 10,
        max_withdrawal_amount: 500,
        global_cap: 5000,
    };

    vault_client.initialize_risk(&admin, &risk_params);

    let user = Address::generate(&e);

    // Mock token balance
    e.as_contract(&deposit_token, || {
        e.storage().instance().set(&TokenDataKey::Balance(user.clone()), &2000i128);
        e.storage().instance().set(&TokenDataKey::Balance(vault_id.clone()), &0i128);
    });

    // Test min deposit
    let result = vault_client.try_deposit(&user, &5i128);
    assert!(result.is_err());

    // Test valid deposit
    vault_client.deposit(&user, &100i128);
    assert_eq!(vault_client.balance(&user), 100);

    // Test max deposit
    let result = vault_client.try_deposit(&user, &2000i128);
    assert!(result.is_err());

    // Test max withdrawal
    let result = vault_client.try_withdraw(&user, &600i128);
    assert!(result.is_err());

    // Test global cap
    vault_client.deposit(&user, &950i128); // current total 1050

    // Set a lower global cap to trigger it easily
    let new_risk_params = RiskParameters {
        max_deposit_amount: 1000,
        min_deposit_amount: 10,
        max_withdrawal_amount: 500,
        global_cap: 1100,
    };
    vault_client.set_risk_params(&admin, &new_risk_params);

    let result = vault_client.try_deposit(&user, &100i128);
    assert!(result.is_err());
}

#[test]
fn test_vault_asset_enforces_risk_limits() {
    let e = Env::default();
    e.mock_all_auths();

    // Register Vault
    let vault_id = e.register_contract(None, VaultContract);
    let vault_client = VaultContractClient::new(&e, &vault_id);

    let admin = Address::generate(&e);
    let deposit_token = e.register_contract(None, MockToken);
    let reward_token = Address::generate(&e);
    let other_asset = e.register_contract(None, MockToken);

    vault_client.initialize(&admin, &deposit_token, &reward_token, &0u64, &0, &vec![&e]);
    vault_client.add_asset(&admin, &other_asset);

    let risk_params = RiskParameters {
        max_deposit_amount: 1000,
        min_deposit_amount: 10,
        max_withdrawal_amount: 500,
        global_cap: 5000,
    };

    vault_client.initialize_risk(&admin, &risk_params);

    let user = Address::generate(&e);

    // Mock token balance
    e.as_contract(&other_asset, || {
        e.storage().instance().set(&TokenDataKey::Balance(user.clone()), &2000i128);
        e.storage().instance().set(&TokenDataKey::Balance(vault_id.clone()), &0i128);
    });

    // Test min deposit for asset
    let result = vault_client.try_deposit_asset(&user, &other_asset, &5i128);
    assert!(result.is_err());

    // Test valid deposit for asset
    vault_client.deposit_asset(&user, &other_asset, &100i128);
    assert_eq!(vault_client.balance_of_asset(&user, &other_asset), 100);

    // Test max withdrawal for asset
    let result = vault_client.try_withdraw_asset(&user, &other_asset, &600i128);
    assert!(result.is_err());
}
