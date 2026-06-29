use soroban_sdk::{contracttype, Address, BytesN, Env, Vec};
use axionvera_interfaces::{ReplayEvent, ReplayReport};
use crate::errors::ReplayError;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    NextEventId,
    Event(u64),
    EventList,
    Report(BytesN<32>),
    ReportList,
    LastProcessedEventId,
}

pub(super) fn is_initialized(e: &Env) -> bool {
    e.storage().instance().has(&DataKey::Admin)
}

pub(super) fn require_initialized(e: &Env) -> Result<(), ReplayError> {
    if !is_initialized(e) {
        return Err(ReplayError::NotInitialized);
    }
    Ok(())
}

pub(super) fn initialize(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::Admin, admin);
    e.storage().instance().set(&DataKey::NextEventId, &1u64);
    e.storage().instance().set(&DataKey::LastProcessedEventId, &0u64);
    e.storage().instance().set(&DataKey::EventList, &Vec::<u64>::new(e));
    e.storage().instance().set(&DataKey::ReportList, &Vec::<BytesN<32>>::new(e));
}

pub(super) fn get_admin(e: &Env) -> Result<Address, ReplayError> {
    e.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(ReplayError::NotInitialized)
}

pub(super) fn get_next_event_id(e: &Env) -> u64 {
    e.storage()
        .instance()
        .get(&DataKey::NextEventId)
        .unwrap_or(1)
}

pub(super) fn set_next_event_id(e: &Env, id: u64) {
    e.storage().instance().set(&DataKey::NextEventId, &id);
}

pub(super) fn get_last_processed_event_id(e: &Env) -> u64 {
    e.storage()
        .instance()
        .get(&DataKey::LastProcessedEventId)
        .unwrap_or(0)
}

pub(super) fn set_last_processed_event_id(e: &Env, id: u64) {
    e.storage().instance().set(&DataKey::LastProcessedEventId, &id);
}

pub(super) fn get_event(e: &Env, event_id: u64) -> Result<ReplayEvent, ReplayError> {
    e.storage()
        .persistent()
        .get(&DataKey::Event(event_id))
        .ok_or(ReplayError::InvalidEvent)
}

pub(super) fn set_event(e: &Env, event: &ReplayEvent) {
    e.storage().persistent().set(&DataKey::Event(event.id), event);
    
    let mut list = get_event_list(e);
    if !list.iter().any(|id| id == event.id) {
        list.push_back(event.id);
        set_event_list(e, &list);
    }
}

pub(super) fn get_event_list(e: &Env) -> Vec<u64> {
    e.storage()
        .instance()
        .get(&DataKey::EventList)
        .unwrap_or_else(|| Vec::new(e))
}

pub(super) fn set_event_list(e: &Env, list: &Vec<u64>) {
    e.storage().instance().set(&DataKey::EventList, list);
}

pub(super) fn get_report(e: &Env, run_id: &BytesN<32>) -> Result<ReplayReport, ReplayError> {
    e.storage()
        .persistent()
        .get(&DataKey::Report(run_id.clone()))
        .ok_or(ReplayError::ReplayFailed)
}

pub(super) fn set_report(e: &Env, report: &ReplayReport) {
    e.storage().persistent().set(&DataKey::Report(report.run_id.clone()), report);
    
    let mut list = get_report_list(e);
    if !list.iter().any(|id| id == report.run_id) {
        list.push_back(report.run_id.clone());
        set_report_list(e, &list);
    }
}

pub(super) fn get_report_list(e: &Env) -> Vec<BytesN<32>> {
    e.storage()
        .instance()
        .get(&DataKey::ReportList)
        .unwrap_or_else(|| Vec::new(e))
}

pub(super) fn set_report_list(e: &Env, list: &Vec<BytesN<32>>) {
    e.storage().instance().set(&DataKey::ReportList, list);
}
