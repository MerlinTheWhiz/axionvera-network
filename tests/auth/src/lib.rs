#![cfg(test)]

use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env, Symbol};
use axionvera_auth::delegation::DelegationManager;
use axionvera_interfaces::DelegationError;

#[test]
fn test_successful_delegation() {
    let e = Env::default();
    e.mock_all_auths();

    let delegator = Address::generate(&e);
    let delegatee = Address::generate(&e);
    let operation = Symbol::new(&e, "transfer");
    let expiration = 1000;

    e.ledger().set_timestamp(500);

    let result = DelegationManager::delegate(&e, delegator.clone(), delegatee.clone(), operation.clone(), expiration);
    assert!(result.is_ok());

    assert!(DelegationManager::is_authorized(&e, delegator.clone(), delegatee.clone(), operation.clone()));
}

#[test]
fn test_rejection_unauthorized() {
    let e = Env::default();
    let delegator = Address::generate(&e);
    let delegatee = Address::generate(&e);
    let operation = Symbol::new(&e, "transfer");

    assert!(!DelegationManager::is_authorized(&e, delegator, delegatee, operation));
}

#[test]
fn test_revocation() {
    let e = Env::default();
    e.mock_all_auths();

    let delegator = Address::generate(&e);
    let delegatee = Address::generate(&e);
    let operation = Symbol::new(&e, "transfer");
    let expiration = 1000;

    e.ledger().set_timestamp(500);

    DelegationManager::delegate(&e, delegator.clone(), delegatee.clone(), operation.clone(), expiration).unwrap();
    assert!(DelegationManager::is_authorized(&e, delegator.clone(), delegatee.clone(), operation.clone()));

    let result = DelegationManager::revoke_delegation(&e, delegator.clone(), delegatee.clone(), operation.clone());
    assert!(result.is_ok());

    assert!(!DelegationManager::is_authorized(&e, delegator, delegatee, operation));
}

#[test]
fn test_expired_delegation() {
    let e = Env::default();
    e.mock_all_auths();

    let delegator = Address::generate(&e);
    let delegatee = Address::generate(&e);
    let operation = Symbol::new(&e, "transfer");
    let expiration = 1000;

    e.ledger().set_timestamp(500);
    DelegationManager::delegate(&e, delegator.clone(), delegatee.clone(), operation.clone(), expiration).unwrap();

    e.ledger().set_timestamp(1001);
    assert!(!DelegationManager::is_authorized(&e, delegator, delegatee, operation));
}

#[test]
fn test_invalid_expiration() {
    let e = Env::default();
    e.mock_all_auths();

    let delegator = Address::generate(&e);
    let delegatee = Address::generate(&e);
    let operation = Symbol::new(&e, "transfer");

    e.ledger().set_timestamp(1000);
    let result = DelegationManager::delegate(&e, delegator, delegatee, operation, 500);

    assert_eq!(result, Err(DelegationError::InvalidExpiration));
}
