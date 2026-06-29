use super::*;

use soroban_sdk::{
    contract, contracterror, contractimpl, symbol_short,
    testutils::{Address as _, Ledger},
    vec, Address, BytesN, Env, IntoVal, Symbol, Val, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum TargetError {
    Failed = 1,
}

#[contract]
pub struct TargetContract;

#[contractimpl]
impl TargetContract {
    pub fn mark(e: Env, key: Symbol, value: u32) {
        let mut entries: Vec<u32> = e
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(&e));
        entries.push_back(value);
        e.storage().persistent().set(&key, &entries);
    }

    pub fn fail(_e: Env) -> Result<(), TargetError> {
        Err(TargetError::Failed)
    }

    pub fn entries(e: Env, key: Symbol) -> Vec<u32> {
        e.storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(&e))
    }
}

fn plan_id(e: &Env, seed: u8) -> BytesN<32> {
    BytesN::from_array(e, &[seed; 32])
}

fn empty_vals(e: &Env) -> Vec<Val> {
    Vec::new(e)
}

fn rollback_vec(e: &Env, rollback: RollbackOperation) -> Vec<RollbackOperation> {
    let mut rollbacks = Vec::new(e);
    rollbacks.push_back(rollback);
    rollbacks
}

fn operation(
    _e: &Env,
    id: u32,
    target: &Address,
    function: Symbol,
    args: Vec<Val>,
    depends_on: Vec<u32>,
    rollback: Vec<RollbackOperation>,
) -> OrchestrationOperation {
    OrchestrationOperation {
        id,
        target: target.clone(),
        function,
        args,
        depends_on,
        rollback,
    }
}

fn rollback(e: &Env, target: &Address, key: &Symbol, value: u32) -> RollbackOperation {
    RollbackOperation {
        target: target.clone(),
        function: symbol_short!("mark"),
        args: (key.clone(), value).into_val(e),
    }
}

fn mark_args(e: &Env, key: &Symbol, value: u32) -> Vec<Val> {
    (key.clone(), value).into_val(e)
}

fn setup(
    e: &Env,
) -> (
    Address,
    OrchestratorContractClient<'_>,
    Address,
    TargetContractClient<'_>,
    Address,
) {
    e.mock_all_auths();
    let orchestrator_id = e.register(OrchestratorContract, ());
    let target_id = e.register(TargetContract, ());
    let orchestrator = OrchestratorContractClient::new(e, &orchestrator_id);
    let target = TargetContractClient::new(e, &target_id);
    let caller = Address::generate(e);
    (orchestrator_id, orchestrator, target_id, target, caller)
}

#[test]
fn validates_execution_plan_dependencies() {
    let e = Env::default();
    let (_orchestrator_id, orchestrator, target_id, _target, caller) = setup(&e);
    let key = symbol_short!("main");

    let plan = ExecutionPlan {
        id: plan_id(&e, 1),
        caller,
        operations: vec![
            &e,
            operation(
                &e,
                2,
                &target_id,
                symbol_short!("mark"),
                mark_args(&e, &key, 20),
                vec![&e, 1_u32],
                Vec::new(&e),
            ),
        ],
    };

    let result = orchestrator.try_validate_plan(&plan);
    assert_eq!(result, Err(Ok(OrchestrationError::DependencyNotOrdered)));
}

#[test]
fn rejects_duplicate_operation_ids() {
    let e = Env::default();
    let (_orchestrator_id, orchestrator, target_id, _target, caller) = setup(&e);
    let key = symbol_short!("main");

    let plan = ExecutionPlan {
        id: plan_id(&e, 2),
        caller,
        operations: vec![
            &e,
            operation(
                &e,
                1,
                &target_id,
                symbol_short!("mark"),
                mark_args(&e, &key, 10),
                Vec::new(&e),
                Vec::new(&e),
            ),
            operation(
                &e,
                1,
                &target_id,
                symbol_short!("mark"),
                mark_args(&e, &key, 20),
                Vec::new(&e),
                Vec::new(&e),
            ),
        ],
    };

    let result = orchestrator.try_validate_plan(&plan);
    assert_eq!(result, Err(Ok(OrchestrationError::DuplicateOperationId)));
}

#[test]
fn rejects_self_targeting_operations() {
    let e = Env::default();
    let (orchestrator_id, orchestrator, _target_id, _target, caller) = setup(&e);

    let plan = ExecutionPlan {
        id: plan_id(&e, 3),
        caller,
        operations: vec![
            &e,
            operation(
                &e,
                1,
                &orchestrator_id,
                symbol_short!("version"),
                empty_vals(&e),
                Vec::new(&e),
                Vec::new(&e),
            ),
        ],
    };

    let result = orchestrator.try_validate_plan(&plan);
    assert_eq!(result, Err(Ok(OrchestrationError::InvalidTarget)));
}

#[test]
fn executes_plan_and_persists_receipt() {
    let e = Env::default();
    e.ledger().set_timestamp(1234);
    let (_orchestrator_id, orchestrator, target_id, target, caller) = setup(&e);
    let key = symbol_short!("main");

    let plan = ExecutionPlan {
        id: plan_id(&e, 4),
        caller: caller.clone(),
        operations: vec![
            &e,
            operation(
                &e,
                1,
                &target_id,
                symbol_short!("mark"),
                mark_args(&e, &key, 10),
                Vec::new(&e),
                Vec::new(&e),
            ),
            operation(
                &e,
                2,
                &target_id,
                symbol_short!("mark"),
                mark_args(&e, &key, 20),
                vec![&e, 1_u32],
                Vec::new(&e),
            ),
        ],
    };

    let receipt = orchestrator.execute_plan(&plan);

    assert_eq!(target.entries(&key), vec![&e, 10_u32, 20_u32]);
    assert_eq!(receipt.status, ExecutionStatus::Succeeded);
    assert_eq!(receipt.executed.len(), 2);
    assert_eq!(receipt.rollback.len(), 0);
    assert_eq!(receipt.failed_operation, None);
    assert_eq!(receipt.timestamp, 1234);
    assert_eq!(orchestrator.execution_receipt(&plan.id), Some(receipt));
}

#[test]
fn rolls_back_executed_operations_in_reverse_order() {
    let e = Env::default();
    let (_orchestrator_id, orchestrator, target_id, target, caller) = setup(&e);
    let main_key = symbol_short!("main");
    let rollback_key = symbol_short!("rb");

    let plan = ExecutionPlan {
        id: plan_id(&e, 5),
        caller,
        operations: vec![
            &e,
            operation(
                &e,
                1,
                &target_id,
                symbol_short!("mark"),
                mark_args(&e, &main_key, 10),
                Vec::new(&e),
                rollback_vec(&e, rollback(&e, &target_id, &rollback_key, 100)),
            ),
            operation(
                &e,
                2,
                &target_id,
                symbol_short!("mark"),
                mark_args(&e, &main_key, 20),
                vec![&e, 1_u32],
                rollback_vec(&e, rollback(&e, &target_id, &rollback_key, 200)),
            ),
            operation(
                &e,
                3,
                &target_id,
                symbol_short!("fail"),
                empty_vals(&e),
                vec![&e, 2_u32],
                Vec::new(&e),
            ),
        ],
    };

    let receipt = orchestrator.execute_plan(&plan);

    assert_eq!(target.entries(&main_key), vec![&e, 10_u32, 20_u32]);
    assert_eq!(target.entries(&rollback_key), vec![&e, 200_u32, 100_u32]);

    assert_eq!(receipt.status, ExecutionStatus::RolledBack);
    assert_eq!(receipt.executed.len(), 2);
    assert_eq!(receipt.rollback.len(), 2);
    assert_eq!(receipt.rollback.get(0).unwrap().operation_id, 2);
    assert_eq!(receipt.rollback.get(1).unwrap().operation_id, 1);
    assert_eq!(receipt.failed_operation, Some(3));
    assert_eq!(orchestrator.execution_receipt(&plan.id), Some(receipt));
}
