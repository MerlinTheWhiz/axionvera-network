#![cfg(test)]

use super::*;
use crate::errors::PolicyError;
use soroban_sdk::{testutils::Address as _, Address, Bytes, BytesN, Env, Symbol, Vec};
use axionvera_interfaces::{Policy, PolicyEvaluationRequest, PolicyStatus, PolicyType};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn name(e: &Env, s: &[u8]) -> Bytes {
    Bytes::from_slice(e, s)
}

fn setup<'a>(e: &'a Env) -> (PolicyContractClient<'a>, Address) {
    let id = e.register_contract(None, PolicyContract {});
    let client = PolicyContractClient::new(e, &id);
    let admin = Address::generate(e);
    (client, admin)
}

// ---------------------------------------------------------------------------
// Initialization
// ---------------------------------------------------------------------------

#[test]
fn test_initialize_succeeds() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);
    assert_eq!(client.admin(), admin);
    assert!(!client.is_paused());
}

#[test]
fn test_initialize_is_one_time() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);
    let result = client.try_initialize(&admin);
    assert_eq!(result, Err(Ok(PolicyError::AlreadyInitialized)));
}

#[test]
fn test_initialize_requires_admin_auth() {
    let e = Env::default();
    let id = e.register_contract(None, PolicyContract {});
    let client = PolicyContractClient::new(&e, &id);
    let admin = Address::generate(&e);
    let result = client.try_initialize(&admin);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// add_policy
// ---------------------------------------------------------------------------

#[test]
fn test_add_policy_succeeds() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let policy_id = BytesN::from_array(&e, &[0; 32]);
    let policy = Policy {
        id: policy_id.clone(),
        name: name(&e, b"Test Policy"),
        policy_type: PolicyType::AllowDeny,
        status: PolicyStatus::Active,
        target_contract: None,
        target_function: None,
        priority: 100,
        created_at: e.ledger().timestamp(),
    };

    client.add_policy(&policy);
    assert_eq!(client.list_policies().len(), 1);
}

#[test]
fn test_add_policy_rejects_empty_name() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let policy_id = BytesN::from_array(&e, &[1; 32]);
    let policy = Policy {
        id: policy_id.clone(),
        name: name(&e, b""),
        policy_type: PolicyType::AllowDeny,
        status: PolicyStatus::Active,
        target_contract: None,
        target_function: None,
        priority: 100,
        created_at: e.ledger().timestamp(),
    };

    let result = client.try_add_policy(&policy);
    assert_eq!(result, Err(Ok(PolicyError::InvalidPolicyName)));
}

#[test]
fn test_add_policy_rejects_duplicate() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let policy_id = BytesN::from_array(&e, &[2; 32]);
    let policy = Policy {
        id: policy_id.clone(),
        name: name(&e, b"Test Policy"),
        policy_type: PolicyType::AllowDeny,
        status: PolicyStatus::Active,
        target_contract: None,
        target_function: None,
        priority: 100,
        created_at: e.ledger().timestamp(),
    };

    client.add_policy(&policy);
    let result = client.try_add_policy(&policy);
    assert_eq!(result, Err(Ok(PolicyError::PolicyAlreadyExists)));
}

// ---------------------------------------------------------------------------
// update_policy
// ---------------------------------------------------------------------------

#[test]
fn test_update_policy_succeeds() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let policy_id = BytesN::from_array(&e, &[3; 32]);
    let mut policy = Policy {
        id: policy_id.clone(),
        name: name(&e, b"Old Name"),
        policy_type: PolicyType::AllowDeny,
        status: PolicyStatus::Active,
        target_contract: None,
        target_function: None,
        priority: 100,
        created_at: e.ledger().timestamp(),
    };

    client.add_policy(&policy);
    policy.name = name(&e, b"New Name");
    client.update_policy(&policy);

    assert_eq!(client.get_policy(&policy_id).name, name(&e, b"New Name"));
}

// ---------------------------------------------------------------------------
// delete_policy
// ---------------------------------------------------------------------------

#[test]
fn test_delete_policy_succeeds() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let policy_id = BytesN::from_array(&e, &[4; 32]);
    let policy = Policy {
        id: policy_id.clone(),
        name: name(&e, b"To Delete"),
        policy_type: PolicyType::AllowDeny,
        status: PolicyStatus::Active,
        target_contract: None,
        target_function: None,
        priority: 100,
        created_at: e.ledger().timestamp(),
    };

    client.add_policy(&policy);
    client.delete_policy(&policy_id);
    assert_eq!(client.list_policies().len(), 0);
}

// ---------------------------------------------------------------------------
// evaluate
// ---------------------------------------------------------------------------

#[test]
fn test_evaluate_passes_with_no_policies() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let caller = Address::generate(&e);
    let target_contract = Address::generate(&e);
    let request = PolicyEvaluationRequest {
        target_contract: target_contract.clone(),
        target_function: Symbol::new(&e, "test_func"),
        args: Vec::new(&e),
        caller: caller.clone(),
    };

    let result = client.evaluate(&request);
    assert!(result.passed);
}

// ---------------------------------------------------------------------------
// Admin transfer
// ---------------------------------------------------------------------------

#[test]
fn test_propose_and_accept_admin_transfer() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let new_admin = Address::generate(&e);
    client.propose_new_admin(&new_admin);
    client.accept_admin(&new_admin);

    assert_eq!(client.admin(), new_admin);
}

// ---------------------------------------------------------------------------
// Pause / unpause
// ---------------------------------------------------------------------------

#[test]
fn test_pause_blocks_add_policy() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);
    client.pause_contract();

    let policy_id = BytesN::from_array(&e, &[5; 32]);
    let policy = Policy {
        id: policy_id.clone(),
        name: name(&e, b"Test"),
        policy_type: PolicyType::AllowDeny,
        status: PolicyStatus::Active,
        target_contract: None,
        target_function: None,
        priority: 100,
        created_at: e.ledger().timestamp(),
    };

    let result = client.try_add_policy(&policy);
    assert_eq!(result, Err(Ok(PolicyError::Paused)));
}

#[test]
fn test_unpause_works() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);
    client.pause_contract();
    client.unpause_contract();
    assert!(!client.is_paused());
}

// ---------------------------------------------------------------------------
// version
// ---------------------------------------------------------------------------

#[test]
fn test_version_is_one() {
    assert_eq!(PolicyContract::version(), 1);
}
