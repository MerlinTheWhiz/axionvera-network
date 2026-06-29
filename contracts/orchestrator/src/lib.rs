#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, InvokeError, Vec};

use axionvera_events::{
    self, OrchestrationExecutedEvent, OrchestrationFailedEvent, OrchestrationRollbackEvent,
    OrchestrationValidatedEvent, ACT_ORCH_EXECUTED, ACT_ORCH_FAILED, ACT_ORCH_ROLLBACK,
    ACT_ORCH_VALIDATED, EVENT_VERSION, PROTOCOL,
};
use axionvera_interfaces::{
    ExecutionPlan, ExecutionReceipt, ExecutionStatus, OperationReceipt, OperationStatus,
    OrchestrationError, OrchestrationOperation, RollbackOperation, TransactionOrchestrator,
};

const MAX_OPERATIONS: u32 = 16;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Receipt(BytesN<32>),
}

#[contract]
pub struct OrchestratorContract;

#[contractimpl]
impl OrchestratorContract {
    pub fn version() -> u32 {
        1
    }
}

#[contractimpl]
impl TransactionOrchestrator for OrchestratorContract {
    fn validate_plan(e: Env, plan: ExecutionPlan) -> Result<(), OrchestrationError> {
        validate_plan_inner(&e, &plan)?;
        emit_validated(&e, &plan);
        Ok(())
    }

    fn execute_plan(e: Env, plan: ExecutionPlan) -> Result<ExecutionReceipt, OrchestrationError> {
        validate_plan_inner(&e, &plan)?;
        plan.caller.require_auth();
        emit_validated(&e, &plan);

        let mut executed = Vec::new(&e);
        let mut rollback_receipts = Vec::new(&e);

        for operation in plan.operations.iter() {
            let operation_failed = match e.try_invoke_contract::<(), InvokeError>(
                &operation.target,
                &operation.function,
                operation.args.clone(),
            ) {
                Ok(Ok(())) => false,
                Ok(Err(_)) | Err(_) => true,
            };

            if operation_failed {
                rollback_receipts = rollback_executed(&e, &plan.id, &executed, &plan.operations)?;
                let receipt = build_receipt(
                    &e,
                    &plan,
                    ExecutionStatus::RolledBack,
                    executed,
                    rollback_receipts,
                    Some(operation.id),
                );
                store_receipt(&e, &receipt);
                emit_failed(&e, &receipt, operation.id);
                return Ok(receipt);
            }

            executed.push_back(OperationReceipt {
                operation_id: operation.id,
                status: OperationStatus::Executed,
            });
        }

        let receipt = build_receipt(
            &e,
            &plan,
            ExecutionStatus::Succeeded,
            executed,
            rollback_receipts,
            None,
        );
        store_receipt(&e, &receipt);
        emit_executed(&e, &receipt);
        Ok(receipt)
    }

    fn execution_receipt(e: Env, plan_id: BytesN<32>) -> Option<ExecutionReceipt> {
        e.storage()
            .persistent()
            .get(&DataKey::Receipt(plan_id))
    }
}

fn validate_plan_inner(e: &Env, plan: &ExecutionPlan) -> Result<(), OrchestrationError> {
    let operation_count = plan.operations.len();
    if operation_count == 0 {
        return Err(OrchestrationError::EmptyPlan);
    }
    if operation_count > MAX_OPERATIONS {
        return Err(OrchestrationError::TooManyOperations);
    }

    let mut seen = Vec::new(e);
    for operation in plan.operations.iter() {
        validate_operation(e, &operation)?;

        if contains_u32(&seen, operation.id) {
            return Err(OrchestrationError::DuplicateOperationId);
        }

        for dependency in operation.depends_on.iter() {
            if dependency == operation.id || !contains_u32(&seen, dependency) {
                return Err(OrchestrationError::DependencyNotOrdered);
            }
        }

        seen.push_back(operation.id);
    }

    Ok(())
}

fn validate_operation(
    e: &Env,
    operation: &OrchestrationOperation,
) -> Result<(), OrchestrationError> {
    validate_target(e, &operation.target)?;
    if operation.rollback.len() > 1 {
        return Err(OrchestrationError::InvalidDependency);
    }
    for rollback in operation.rollback.iter() {
        validate_rollback(e, &rollback)?;
    }
    Ok(())
}

fn validate_rollback(e: &Env, rollback: &RollbackOperation) -> Result<(), OrchestrationError> {
    validate_target(e, &rollback.target)
}

fn validate_target(e: &Env, target: &Address) -> Result<(), OrchestrationError> {
    if target == &e.current_contract_address() {
        return Err(OrchestrationError::InvalidTarget);
    }
    Ok(())
}

fn rollback_executed(
    e: &Env,
    plan_id: &BytesN<32>,
    executed: &Vec<OperationReceipt>,
    operations: &Vec<OrchestrationOperation>,
) -> Result<Vec<OperationReceipt>, OrchestrationError> {
    let mut rollback_receipts = Vec::new(e);
    let mut index = executed.len();

    while index > 0 {
        index -= 1;
        let receipt = executed.get(index).ok_or(OrchestrationError::RollbackFailed)?;
        let operation = find_operation(operations, receipt.operation_id)
            .ok_or(OrchestrationError::RollbackFailed)?;

        if let Some(rollback) = operation.rollback.get(0) {
            e.try_invoke_contract::<(), InvokeError>(
                &rollback.target,
                &rollback.function,
                rollback.args.clone(),
            )
            .map_err(|_| OrchestrationError::RollbackFailed)?
            .map_err(|_| OrchestrationError::RollbackFailed)?;

            rollback_receipts.push_back(OperationReceipt {
                operation_id: receipt.operation_id,
                status: OperationStatus::RolledBack,
            });
            emit_rollback(e, plan_id, receipt.operation_id);
        }
    }

    Ok(rollback_receipts)
}

fn find_operation(
    operations: &Vec<OrchestrationOperation>,
    operation_id: u32,
) -> Option<OrchestrationOperation> {
    for operation in operations.iter() {
        if operation.id == operation_id {
            return Some(operation);
        }
    }
    None
}

fn contains_u32(values: &Vec<u32>, needle: u32) -> bool {
    for value in values.iter() {
        if value == needle {
            return true;
        }
    }
    false
}

fn build_receipt(
    e: &Env,
    plan: &ExecutionPlan,
    status: ExecutionStatus,
    executed: Vec<OperationReceipt>,
    rollback: Vec<OperationReceipt>,
    failed_operation: Option<u32>,
) -> ExecutionReceipt {
    ExecutionReceipt {
        plan_id: plan.id.clone(),
        caller: plan.caller.clone(),
        status,
        executed,
        rollback,
        failed_operation,
        timestamp: axionvera_events::ledger_timestamp(e),
    }
}

fn store_receipt(e: &Env, receipt: &ExecutionReceipt) {
    e.storage()
        .persistent()
        .set(&DataKey::Receipt(receipt.plan_id.clone()), receipt);
}

fn emit_validated(e: &Env, plan: &ExecutionPlan) {
    e.events().publish(
        (PROTOCOL, ACT_ORCH_VALIDATED),
        OrchestrationValidatedEvent {
            event_version: EVENT_VERSION,
            plan_id: plan.id.clone(),
            caller: plan.caller.clone(),
            operation_count: plan.operations.len(),
            timestamp: axionvera_events::ledger_timestamp(e),
        },
    );
}

fn emit_executed(e: &Env, receipt: &ExecutionReceipt) {
    e.events().publish(
        (PROTOCOL, ACT_ORCH_EXECUTED),
        OrchestrationExecutedEvent {
            event_version: EVENT_VERSION,
            plan_id: receipt.plan_id.clone(),
            caller: receipt.caller.clone(),
            executed_count: receipt.executed.len(),
            timestamp: receipt.timestamp,
        },
    );
}

fn emit_rollback(e: &Env, plan_id: &BytesN<32>, operation_id: u32) {
    e.events().publish(
        (PROTOCOL, ACT_ORCH_ROLLBACK),
        OrchestrationRollbackEvent {
            event_version: EVENT_VERSION,
            plan_id: plan_id.clone(),
            operation_id,
            timestamp: axionvera_events::ledger_timestamp(e),
        },
    );
}

fn emit_failed(e: &Env, receipt: &ExecutionReceipt, failed_operation: u32) {
    e.events().publish(
        (PROTOCOL, ACT_ORCH_FAILED),
        OrchestrationFailedEvent {
            event_version: EVENT_VERSION,
            plan_id: receipt.plan_id.clone(),
            caller: receipt.caller.clone(),
            failed_operation,
            rollback_count: receipt.rollback.len(),
            timestamp: receipt.timestamp,
        },
    );
}

#[cfg(test)]
mod test;
