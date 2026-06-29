#![cfg(test)]

use super::*;
use crate::errors::ReplayError;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Symbol, Val, Vec};
use axionvera_interfaces::{ReplayEvent, ReplayEventStatus, ReplayReport};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn setup<'a>(e: &'a Env) -> (ReplayContractClient<'a>, Address) {
    let id = e.register_contract(None, ReplayContract {});
    let client = ReplayContractClient::new(e, &id);
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
}

#[test]
fn test_initialize_is_one_time() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);
    let result = client.try_initialize(&admin);
    assert_eq!(result, Err(Ok(ReplayError::AlreadyInitialized)));
}

// ---------------------------------------------------------------------------
// Add Event
// ---------------------------------------------------------------------------

#[test]
fn test_add_event_succeeds() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let protocol = Symbol::new(&e, "TestProtocol");
    let action = Symbol::new(&e, "TestAction");
    let timestamp = 1234567890;
    let payload: Val = ().into_val(&e);

    let event_id = client.add_event(&protocol, &action, &timestamp, &payload);
    assert_eq!(event_id, 1);

    let events = client.list_events();
    assert_eq!(events.len(), 1);
    assert_eq!(events.get(0).unwrap().id, 1);
}

// ---------------------------------------------------------------------------
// Replay
// ---------------------------------------------------------------------------

#[test]
fn test_start_replay_succeeds() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    // Add a few test events
    let protocol = Symbol::new(&e, "TestProtocol");
    let action = Symbol::new(&e, "TestAction");
    let payload: Val = ().into_val(&e);
    client.add_event(&protocol, &action, &1, &payload);
    client.add_event(&protocol, &action, &2, &payload);

    // Start replay
    let report = client.start_replay();
    assert_eq!(report.total_events, 2);
    assert_eq!(report.successful_events, 2);
    assert_eq!(report.failed_events, 0);
    assert!(report.success);
}

// ---------------------------------------------------------------------------
// Version
// ---------------------------------------------------------------------------

#[test]
fn test_version_is_one() {
    assert_eq!(ReplayContract::version(), 1);
}
