use soroban_sdk::{contracttype, Address, BytesN, Env, Vec};
use axionvera_interfaces::ScheduledTask;
use crate::errors::SchedulerError;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Initialized,
    Admin,
    PendingAdmin,
    IsPaused,
    Task(BytesN<32>),
    TaskList,
}

pub fn is_initialized(e: &Env) -> bool {
    e.storage().instance().has(&DataKey::Initialized)
}

pub fn require_initialized(e: &Env) -> Result<(), SchedulerError> {
    if !is_initialized(e) {
        return Err(SchedulerError::NotInitialized);
    }
    Ok(())
}

pub fn require_not_paused(e: &Env) -> Result<(), SchedulerError> {
    if get_is_paused(e) {
        return Err(SchedulerError::ContractPaused);
    }
    Ok(())
}

pub fn initialize(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::Initialized, &true);
    e.storage().instance().set(&DataKey::Admin, admin);
    e.storage().instance().set(&DataKey::IsPaused, &false);
    e.storage().instance().set(&DataKey::TaskList, &Vec::<BytesN<32>>::new(e));
}

pub fn get_admin(e: &Env) -> Result<Address, SchedulerError> {
    e.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(SchedulerError::NotInitialized)
}

pub fn set_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_pending_admin(e: &Env) -> Option<Address> {
    e.storage().instance().get(&DataKey::PendingAdmin)
}

pub fn set_pending_admin(e: &Env, pending_admin: &Address) {
    e.storage().instance().set(&DataKey::PendingAdmin, pending_admin);
}

pub fn clear_pending_admin(e: &Env) {
    e.storage().instance().remove(&DataKey::PendingAdmin);
}

pub fn get_is_paused(e: &Env) -> bool {
    e.storage()
        .instance()
        .get(&DataKey::IsPaused)
        .unwrap_or(false)
}

pub fn set_paused(e: &Env, is_paused: &bool) {
    e.storage().instance().set(&DataKey::IsPaused, is_paused);
}

pub fn has_task(e: &Env, task_id: &BytesN<32>) -> bool {
    e.storage().persistent().has(&DataKey::Task(task_id.clone()))
}

pub fn get_task(e: &Env, task_id: &BytesN<32>) -> Result<ScheduledTask, SchedulerError> {
    e.storage()
        .persistent()
        .get(&DataKey::Task(task_id.clone()))
        .ok_or(SchedulerError::TaskNotFound)
}

pub fn set_task(e: &Env, task: &ScheduledTask) {
    e.storage().persistent().set(&DataKey::Task(task.id.clone()), task);
    let mut task_list = get_task_list(e);
    if !task_list.iter().any(|id| id == task.id) {
        task_list.push_back(task.id.clone());
        set_task_list(e, &task_list);
    }
}

pub fn delete_task(e: &Env, task_id: &BytesN<32>) {
    e.storage().persistent().remove(&DataKey::Task(task_id.clone()));
    let mut task_list = get_task_list(e);
    let mut new_list = Vec::new(e);
    for id in task_list.iter() {
        if id != *task_id {
            new_list.push_back(id);
        }
    }
    set_task_list(e, &new_list);
}

pub fn get_task_list(e: &Env) -> Vec<BytesN<32>> {
    e.storage()
        .instance()
        .get(&DataKey::TaskList)
        .unwrap_or_else(|| Vec::new(e))
}

pub fn set_task_list(e: &Env, list: &Vec<BytesN<32>>) {
    e.storage().instance().set(&DataKey::TaskList, list);
}
