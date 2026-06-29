use soroban_sdk::{Address, Bytes, BytesN, Env, Symbol};
use axionvera_events::{
    SchedulerInitializedEvent, TaskScheduledEvent, TaskUpdatedEvent, TaskCanceledEvent,
    TaskExecutedEvent, TaskFailedEvent, SchedulerAdminTransferProposedEvent,
    SchedulerAdminTransferAcceptedEvent, SchedulerPausedEvent, SchedulerUnpausedEvent,
    PROTOCOL_SCHEDULER, ACT_SCHED_INIT, ACT_SCHED_TASK_SCHEDULED, ACT_SCHED_TASK_UPDATED,
    ACT_SCHED_TASK_CANCELED, ACT_SCHED_TASK_EXECUTED, ACT_SCHED_TASK_FAILED, ACT_SCHED_ADMIN_P,
    ACT_SCHED_ADMIN_A, ACT_SCHED_PAUSE, ACT_SCHED_UNPAUSE, EVENT_VERSION, ledger_timestamp,
};

pub fn emit_initialized(e: &Env, admin: Address) {
    e.events().publish(
        (PROTOCOL_SCHEDULER, ACT_SCHED_INIT),
        SchedulerInitializedEvent {
            event_version: EVENT_VERSION,
            admin,
            timestamp: ledger_timestamp(e),
        },
    );
}

pub fn emit_task_scheduled(e: &Env, task_id: BytesN<32>, task_name: Bytes, priority: u32, created_by: Address) {
    e.events().publish(
        (PROTOCOL_SCHEDULER, ACT_SCHED_TASK_SCHEDULED),
        TaskScheduledEvent {
            event_version: EVENT_VERSION,
            task_id,
            task_name,
            priority,
            created_by,
            timestamp: ledger_timestamp(e),
        },
    );
}

pub fn emit_task_updated(e: &Env, task_id: BytesN<32>, task_name: Bytes, updated_by: Address) {
    e.events().publish(
        (PROTOCOL_SCHEDULER, ACT_SCHED_TASK_UPDATED),
        TaskUpdatedEvent {
            event_version: EVENT_VERSION,
            task_id,
            task_name,
            updated_by,
            timestamp: ledger_timestamp(e),
        },
    );
}

pub fn emit_task_canceled(e: &Env, task_id: BytesN<32>, canceled_by: Address) {
    e.events().publish(
        (PROTOCOL_SCHEDULER, ACT_SCHED_TASK_CANCELED),
        TaskCanceledEvent {
            event_version: EVENT_VERSION,
            task_id,
            canceled_by,
            timestamp: ledger_timestamp(e),
        },
    );
}

pub fn emit_task_executed(e: &Env, task_id: BytesN<32>, task_name: Bytes, execution_count: u32) {
    e.events().publish(
        (PROTOCOL_SCHEDULER, ACT_SCHED_TASK_EXECUTED),
        TaskExecutedEvent {
            event_version: EVENT_VERSION,
            task_id,
            task_name,
            execution_count,
            timestamp: ledger_timestamp(e),
        },
    );
}

pub fn emit_task_failed(e: &Env, task_id: BytesN<32>, task_name: Bytes) {
    e.events().publish(
        (PROTOCOL_SCHEDULER, ACT_SCHED_TASK_FAILED),
        TaskFailedEvent {
            event_version: EVENT_VERSION,
            task_id,
            task_name,
            timestamp: ledger_timestamp(e),
        },
    );
}

pub fn emit_admin_transfer_proposed(e: &Env, current_admin: Address, pending_admin: Address) {
    e.events().publish(
        (PROTOCOL_SCHEDULER, ACT_SCHED_ADMIN_P),
        SchedulerAdminTransferProposedEvent {
            event_version: EVENT_VERSION,
            current_admin,
            pending_admin,
            timestamp: ledger_timestamp(e),
        },
    );
}

pub fn emit_admin_transfer_accepted(e: &Env, previous_admin: Address, new_admin: Address) {
    e.events().publish(
        (PROTOCOL_SCHEDULER, ACT_SCHED_ADMIN_A),
        SchedulerAdminTransferAcceptedEvent {
            event_version: EVENT_VERSION,
            previous_admin,
            new_admin,
            timestamp: ledger_timestamp(e),
        },
    );
}

pub fn emit_paused(e: &Env, admin: Address) {
    e.events().publish(
        (PROTOCOL_SCHEDULER, ACT_SCHED_PAUSE),
        SchedulerPausedEvent {
            event_version: EVENT_VERSION,
            admin,
            timestamp: ledger_timestamp(e),
        },
    );
}

pub fn emit_unpaused(e: &Env, admin: Address) {
    e.events().publish(
        (PROTOCOL_SCHEDULER, ACT_SCHED_UNPAUSE),
        SchedulerUnpausedEvent {
            event_version: EVENT_VERSION,
            admin,
            timestamp: ledger_timestamp(e),
        },
    );
}
