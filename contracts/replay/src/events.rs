use soroban_sdk::{Address, BytesN, Env, Symbol};
use axionvera_events::{ledger_timestamp, EVENT_VERSION};

// Protocol and action symbols for replay events
pub const PROTOCOL_REPLAY: Symbol = Symbol::new_from_array(b"AxReplay\0\0\0\0\0\0");
pub const ACT_REPLAY_INIT: Symbol = Symbol::new_from_array(b"rep_init\0\0\0\0");
pub const ACT_REPLAY_START: Symbol = Symbol::new_from_array(b"rep_start\0\0");
pub const ACT_REPLAY_COMPLETE: Symbol = Symbol::new_from_array(b"rep_complete\0");

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayInitializedEvent {
    pub event_version: u32,
    pub admin: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayStartedEvent {
    pub event_version: u32,
    pub run_id: BytesN<32>,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayCompletedEvent {
    pub event_version: u32,
    pub run_id: BytesN<32>,
    pub success: bool,
    pub total_events: u64,
    pub successful_events: u64,
    pub timestamp: u64,
}

pub(super) fn emit_initialized(e: &Env, admin: Address) {
    e.events().publish(
        (PROTOCOL_REPLAY, ACT_REPLAY_INIT),
        ReplayInitializedEvent {
            event_version: EVENT_VERSION,
            admin,
            timestamp: ledger_timestamp(e),
        },
    );
}

pub(super) fn emit_replay_started(e: &Env, run_id: BytesN<32>) {
    e.events().publish(
        (PROTOCOL_REPLAY, ACT_REPLAY_START),
        ReplayStartedEvent {
            event_version: EVENT_VERSION,
            run_id,
            timestamp: ledger_timestamp(e),
        },
    );
}

pub(super) fn emit_replay_completed(
    e: &Env,
    run_id: BytesN<32>,
    success: bool,
    total_events: u64,
    successful_events: u64,
) {
    e.events().publish(
        (PROTOCOL_REPLAY, ACT_REPLAY_COMPLETE),
        ReplayCompletedEvent {
            event_version: EVENT_VERSION,
            run_id,
            success,
            total_events,
            successful_events,
            timestamp: ledger_timestamp(e),
        },
    );
}
