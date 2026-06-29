#![cfg(test)]

use super::*;
use soroban_sdk::{
    contract, contractimpl,
    testutils::{Address as _, Events, Ledger},
    Address, BytesN, Env, Vec,
};

#[contract]
struct ContextHost;

#[contractimpl]
impl ContextHost {
    pub fn create(e: Env, caller: Address) -> ExecutionContext {
        create_context(&e, caller)
    }

    pub fn push(e: Env, parent: ExecutionContext, caller: Address) -> ExecutionContext {
        push_context(&e, &parent, caller).unwrap()
    }
}

#[test]
fn creates_top_level_context() {
    let e = Env::default();
    e.ledger().set_timestamp(5000);
    let caller = Address::generate(&e);

    let ctx = create_context(&e, caller.clone());

    assert_eq!(ctx.original_caller, caller);
    assert_eq!(ctx.current_caller, caller);
    assert_eq!(ctx.protocol_version, 1);
    assert_eq!(ctx.depth, 0);
    assert_eq!(ctx.timestamp, 5000);
    assert!(ctx.operation_id.is_none());
    assert!(ctx.plan_id.is_none());
}

#[test]
fn push_increments_depth() {
    let e = Env::default();
    e.ledger().set_timestamp(1000);
    let alice = Address::generate(&e);
    let bob = Address::generate(&e);

    let ctx = create_context(&e, alice.clone());
    let nested = push_context(&e, &ctx, bob.clone()).unwrap();

    assert_eq!(nested.depth, 1);
    assert_eq!(nested.original_caller, alice);
    assert_eq!(nested.current_caller, bob);
}

#[test]
fn push_preserves_plan_metadata() {
    let e = Env::default();
    e.ledger().set_timestamp(2000);
    let caller = Address::generate(&e);
    let plan_id = BytesN::from_array(&e, &[42; 32]);

    let ctx = create_context(&e, caller.clone());
    let ctx = ctx.with_operation(7, plan_id.clone());

    let nested = push_context(&e, &ctx, Address::generate(&e)).unwrap();
    assert_eq!(nested.operation_id, Some(7));
    assert_eq!(nested.plan_id, Some(plan_id));
}

#[test]
fn validates_consistent_context() {
    let e = Env::default();
    e.ledger().set_timestamp(1000);
    let caller = Address::generate(&e);

    let ctx = create_context(&e, caller);
    assert_eq!(ctx.validate(&e), Ok(()));
}

#[test]
fn rejects_future_timestamp() {
    let e = Env::default();
    let caller = Address::generate(&e);
    e.ledger().set_timestamp(500);

    let ctx = ExecutionContext::new(&e, caller);
    e.ledger().set_timestamp(100);
    assert_eq!(ctx.validate(&e), Err(ContextError::TimestampInconsistency));
}

#[test]
fn original_caller_immutable_across_nesting() {
    let e = Env::default();
    e.ledger().set_timestamp(100);
    let alice = Address::generate(&e);
    let bob = Address::generate(&e);
    let charlie = Address::generate(&e);

    let ctx = create_context(&e, alice.clone());
    let ctx = push_context(&e, &ctx, bob).unwrap();
    let ctx = push_context(&e, &ctx, charlie).unwrap();

    assert_eq!(ctx.original_caller, alice);
    assert_eq!(ctx.depth, 2);
}

#[test]
fn push_beyond_max_depth_returns_error() {
    let e = Env::default();
    e.ledger().set_timestamp(100);
    let caller = Address::generate(&e);

    let mut ctx = create_context(&e, caller.clone());
    for _ in 0..MAX_DEPTH {
        ctx = push_context(&e, &ctx, Address::generate(&e)).unwrap();
    }

    assert_eq!(
        push_context(&e, &ctx, Address::generate(&e)),
        Err(ContextError::MaxDepthExceeded)
    );
}

#[test]
fn events_emitted_through_contract() {
    let e = Env::default();
    e.ledger().set_timestamp(500);
    e.mock_all_auths();

    let host_id = e.register(ContextHost, ());
    let host = ContextHostClient::new(&e, &host_id);

    let caller = Address::generate(&e);
    let _ = host.create(&caller);

    let all: Vec<_> = e.events().all();
    assert_eq!(all.len(), 1);
}

#[test]
fn with_operation_sets_metadata() {
    let e = Env::default();
    e.ledger().set_timestamp(100);
    let caller = Address::generate(&e);
    let plan_id = BytesN::from_array(&e, &[1; 32]);

    let ctx = create_context(&e, caller);
    let ctx = ctx.with_operation(3, plan_id.clone());

    assert_eq!(ctx.operation_id, Some(3));
    assert_eq!(ctx.plan_id, Some(plan_id));
}
