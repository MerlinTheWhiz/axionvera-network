#![cfg(test)]

use super::*;
use crate::errors::SchedulerError;
use soroban_sdk::{testutils::Address as _, Address, Bytes, BytesN, Env, Symbol, Val, Vec};
use axionvera_interfaces::{ExecutionWindow, ScheduledTask, ScheduledTaskStatus};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn name(e: &Env, s: &[u8]) -> Bytes {
    Bytes::from_slice(e, s)
}

fn setup<'a>(e: &'a Env) -> (SchedulerContractClient<'a>, Address) {
    let id = e.register_contract(None, SchedulerContract {});
    let client = SchedulerContractClient::new(e, &id);
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
    assert_eq!(result, Err(Ok(SchedulerError::AlreadyInitialized)));
}

// ---------------------------------------------------------------------------
// Scheduling Tasks
// ---------------------------------------------------------------------------

#[test]
fn test_schedule_task_succeeds() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let task_id = BytesN::from_array(&e, &[1u8; 32]);
    let target = Address::generate(&e);
    let window = ExecutionWindow {
        start_time: 0,
        end_time: 1000,
        recurrence_interval: None,
        max_recurrences: None,
    };

    let task = ScheduledTask {
        id: task_id.clone(),
        name: name(&e, b"Test Task"),
        priority: 100,
        window: window.clone(),
        target_contract: target.clone(),
        target_function: Symbol::new(&e, "test"),
        args: Vec::new(&e),
        dependencies: Vec::new(&e),
        status: ScheduledTaskStatus::Pending,
        execution_count: 0,
        created_at: e.ledger().timestamp(),
        last_executed_at: None,
    };

    client.schedule_task(&task);
    let retrieved = client.get_task(&task_id);
    assert_eq!(retrieved.name, name(&e, b"Test Task"));
}

#[test]
fn test_schedule_task_rejects_invalid_name() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let task_id = BytesN::from_array(&e, &[1u8; 32]);
    let target = Address::generate(&e);
    let window = ExecutionWindow {
        start_time: 0,
        end_time: 1000,
        recurrence_interval: None,
        max_recurrences: None,
    };

    let task = ScheduledTask {
        id: task_id.clone(),
        name: name(&e, b""), // Empty name
        priority: 100,
        window: window.clone(),
        target_contract: target.clone(),
        target_function: Symbol::new(&e, "test"),
        args: Vec::new(&e),
        dependencies: Vec::new(&e),
        status: ScheduledTaskStatus::Pending,
        execution_count: 0,
        created_at: e.ledger().timestamp(),
        last_executed_at: None,
    };

    let result = client.try_schedule_task(&task);
    assert_eq!(result, Err(Ok(SchedulerError::InvalidTaskName)));
}

// ---------------------------------------------------------------------------
// Version
// ---------------------------------------------------------------------------

#[test]
fn test_version_is_one() {
    assert_eq!(SchedulerContract::version(), 1);
}
