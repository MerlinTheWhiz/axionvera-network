use soroban_sdk::{Address, Bytes, BytesN, Env, Symbol};
use axionvera_events::{
    PolicyInitializedEvent, PolicyAddedEvent, PolicyUpdatedEvent, PolicyDeletedEvent,
    PolicyEvaluatedEvent, PolicyAdminTransferProposedEvent, PolicyAdminTransferAcceptedEvent,
    PolicyPausedEvent, PolicyUnpausedEvent, PROTOCOL_POLICY, ACT_POL_INIT, ACT_POL_ADD,
    ACT_POL_UPD, ACT_POL_DEL, ACT_POL_EVAL, ACT_POL_ADM_P, ACT_POL_ADM_A, ACT_POL_PAUSE,
    ACT_POL_UNPAU, EVENT_VERSION,
};

pub(super) fn emit_initialized(e: &Env, admin: Address) {
    e.events().publish(
        (PROTOCOL_POLICY, ACT_POL_INIT),
        PolicyInitializedEvent {
            event_version: EVENT_VERSION,
            admin,
            timestamp: axionvera_events::ledger_timestamp(e),
        },
    );
}

pub(super) fn emit_policy_added(e: &Env, policy_id: BytesN<32>, policy_name: Bytes, added_by: Address) {
    e.events().publish(
        (PROTOCOL_POLICY, ACT_POL_ADD),
        PolicyAddedEvent {
            event_version: EVENT_VERSION,
            policy_id,
            policy_name,
            added_by,
            timestamp: axionvera_events::ledger_timestamp(e),
        },
    );
}

pub(super) fn emit_policy_updated(e: &Env, policy_id: BytesN<32>, policy_name: Bytes, updated_by: Address) {
    e.events().publish(
        (PROTOCOL_POLICY, ACT_POL_UPD),
        PolicyUpdatedEvent {
            event_version: EVENT_VERSION,
            policy_id,
            policy_name,
            updated_by,
            timestamp: axionvera_events::ledger_timestamp(e),
        },
    );
}

pub(super) fn emit_policy_deleted(e: &Env, policy_id: BytesN<32>, deleted_by: Address) {
    e.events().publish(
        (PROTOCOL_POLICY, ACT_POL_DEL),
        PolicyDeletedEvent {
            event_version: EVENT_VERSION,
            policy_id,
            deleted_by,
            timestamp: axionvera_events::ledger_timestamp(e),
        },
    );
}

pub(super) fn emit_policy_evaluated(
    e: &Env,
    request_caller: Address,
    target_contract: Address,
    target_function: Symbol,
    passed: bool,
    failed_policy_id: Option<BytesN<32>>,
) {
    e.events().publish(
        (PROTOCOL_POLICY, ACT_POL_EVAL),
        PolicyEvaluatedEvent {
            event_version: EVENT_VERSION,
            request_caller,
            target_contract,
            target_function,
            passed,
            failed_policy_id,
            timestamp: axionvera_events::ledger_timestamp(e),
        },
    );
}

pub(super) fn emit_admin_transfer_proposed(e: &Env, current_admin: Address, pending_admin: Address) {
    e.events().publish(
        (PROTOCOL_POLICY, ACT_POL_ADM_P),
        PolicyAdminTransferProposedEvent {
            event_version: EVENT_VERSION,
            current_admin,
            pending_admin,
            timestamp: axionvera_events::ledger_timestamp(e),
        },
    );
}

pub(super) fn emit_admin_transfer_accepted(e: &Env, previous_admin: Address, new_admin: Address) {
    e.events().publish(
        (PROTOCOL_POLICY, ACT_POL_ADM_A),
        PolicyAdminTransferAcceptedEvent {
            event_version: EVENT_VERSION,
            previous_admin,
            new_admin,
            timestamp: axionvera_events::ledger_timestamp(e),
        },
    );
}

pub(super) fn emit_paused(e: &Env, admin: Address) {
    e.events().publish(
        (PROTOCOL_POLICY, ACT_POL_PAUSE),
        PolicyPausedEvent {
            event_version: EVENT_VERSION,
            admin,
            timestamp: axionvera_events::ledger_timestamp(e),
        },
    );
}

pub(super) fn emit_unpaused(e: &Env, admin: Address) {
    e.events().publish(
        (PROTOCOL_POLICY, ACT_POL_UNPAU),
        PolicyUnpausedEvent {
            event_version: EVENT_VERSION,
            admin,
            timestamp: axionvera_events::ledger_timestamp(e),
        },
    );
}
