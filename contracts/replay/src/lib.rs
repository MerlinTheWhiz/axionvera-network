#![no_std]

pub mod errors;
mod events;
mod storage;
#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env, Symbol, Val, Vec};
use axionvera_interfaces::{
    EventReplayEngine, ReplayError, ReplayEvent, ReplayEventStatus, ReplayReport,
};
use axionvera_events::{
    PROTOCOL, PROTOCOL_CONFIG, PROTOCOL_ASSETS, PROTOCOL_POLICY, PROTOCOL_REPLAY,
    ACT_INIT, ACT_CFG_INIT, ACT_ASSET_REG, ACT_POL_INIT,
};
use crate::errors::ReplayError as Error;
use crate::storage;

#[contract]
pub struct ReplayContract;

#[contractimpl]
impl ReplayContract {
    /// Returns the contract version.
    pub fn version() -> u32 {
        1
    }
}

#[contractimpl]
impl EventReplayEngine for ReplayContract {
    fn initialize(e: Env, admin: Address) -> Result<(), Error> {
        if storage::is_initialized(&e) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        storage::initialize(&e, &admin);
        events::emit_initialized(&e, admin);
        Ok(())
    }

    fn add_event(
        e: Env,
        protocol: Symbol,
        action: Symbol,
        timestamp: u64,
        payload: Val,
    ) -> Result<u64, Error> {
        storage::require_initialized(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();

        let event_id = storage::get_next_event_id(&e);
        let event = ReplayEvent {
            id: event_id,
            protocol,
            action,
            timestamp,
            payload,
            status: ReplayEventStatus::Pending,
            error_message: Bytes::new(&e),
        };

        storage::set_event(&e, &event);
        storage::set_next_event_id(&e, event_id + 1);

        Ok(event_id)
    }

    fn start_replay(e: Env) -> Result<ReplayReport, Error> {
        storage::require_initialized(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();

        let run_id = BytesN::from_array(&e, &e.ledger().timestamp().to_be_bytes());
        events::emit_replay_started(&e, run_id.clone());

        let started_at = e.ledger().timestamp();
        let last_processed = storage::get_last_processed_event_id(&e);
        let event_list = storage::get_event_list(&e);

        let mut total_events = 0u64;
        let mut successful_events = 0u64;
        let mut failed_events = 0u64;
        let mut skipped_events = 0u64;

        // Process events in order
        for event_id in event_list.iter() {
            if event_id <= last_processed {
                skipped_events += 1;
                continue;
            }

            let mut event = storage::get_event(&e, event_id)?;
            total_events += 1;

            // Basic event type checking (placeholder for real state reconstruction)
            match (event.protocol, event.action) {
                (PROTOCOL, ACT_INIT) |
                (PROTOCOL_CONFIG, ACT_CFG_INIT) |
                (PROTOCOL_ASSETS, ACT_ASSET_REG) |
                (PROTOCOL_POLICY, ACT_POL_INIT) |
                (PROTOCOL_REPLAY, ACT_REPLAY_INIT) => {
                    // Mark these events as successfully processed
                    event.status = ReplayEventStatus::Success;
                    successful_events += 1;
                },
                _ => {
                    // Mark all other events as skipped for now
                    event.status = ReplayEventStatus::Skipped;
                    skipped_events += 1;
                }
            }

            storage::set_event(&e, &event);
            storage::set_last_processed_event_id(&e, event_id);
        }

        let ended_at = e.ledger().timestamp();
        let success = failed_events == 0;

        let report = ReplayReport {
            run_id: run_id.clone(),
            total_events,
            successful_events,
            failed_events,
            skipped_events,
            started_at,
            ended_at,
            success,
        };

        storage::set_report(&e, &report);
        events::emit_replay_completed(
            &e,
            run_id,
            success,
            total_events,
            successful_events,
        );

        Ok(report)
    }

    fn get_event(e: Env, event_id: u64) -> Result<ReplayEvent, Error> {
        storage::require_initialized(&e)?;
        storage::get_event(&e, event_id)
    }

    fn list_events(e: Env) -> Result<Vec<ReplayEvent>, Error> {
        storage::require_initialized(&e)?;
        let event_ids = storage::get_event_list(&e);
        let mut events = Vec::new(&e);
        for id in event_ids.iter() {
            if let Ok(event) = storage::get_event(&e, id) {
                events.push_back(event);
            }
        }
        Ok(events)
    }

    fn get_report(e: Env, run_id: BytesN<32>) -> Result<ReplayReport, Error> {
        storage::require_initialized(&e)?;
        storage::get_report(&e, &run_id)
    }

    fn admin(e: Env) -> Result<Address, Error> {
        storage::require_initialized(&e)?;
        storage::get_admin(&e)
    }
}
