# Cross-Contract Transaction Orchestrator

The orchestrator contract coordinates ordered Soroban contract calls from a
single `ExecutionPlan`. A plan contains a caller, a deterministic id, and a
bounded list of operations. Each operation targets a contract function, carries
serialized Soroban arguments, declares dependencies, and can include an optional
rollback call.

## Execution Lifecycle

1. The caller submits an `ExecutionPlan`.
2. The orchestrator validates the plan before any contract call is made.
3. The caller authorizes execution.
4. Operations are invoked in plan order.
5. On success, the orchestrator stores an `ExecutionReceipt` and emits an
   `orc_exec` event.
6. If an operation fails, rollback calls for already executed operations are
   invoked in reverse order. The receipt records the failed operation and all
   completed rollback steps.

Soroban transactions are atomic at the host layer. The orchestrator therefore
uses rollback calls as explicit compensating actions and records deterministic
rollback intent and outcome in the receipt. If a rollback call itself fails, the
execution returns `RollbackFailed` and the transaction aborts.

## Dependency Validation

Dependencies are declared by operation id. Validation enforces:

- plans are non-empty and contain at most 16 operations;
- operation ids are unique;
- an operation cannot target the orchestrator itself;
- rollback targets cannot be the orchestrator itself;
- every dependency must refer to an operation that appears earlier in the plan.

This keeps execution deterministic without a runtime topological sort.

## Events

Orchestration events follow the existing two-topic AxionVera event pattern:

- `(AxVault, orc_val)` when a plan validates;
- `(AxVault, orc_exec)` after all operations succeed;
- `(AxVault, orc_rb)` for each completed rollback operation;
- `(AxVault, orc_fail)` when execution fails after rollback handling.
