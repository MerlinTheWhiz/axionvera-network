use soroban_sdk::{contracttype, Address, BytesN, Env, Vec};
use axionvera_interfaces::Policy;
use crate::errors::PolicyError;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    IsPaused,
    Policy(BytesN<32>),
    PolicyList,
}

pub(super) fn is_initialized(e: &Env) -> bool {
    e.storage().instance().has(&DataKey::Admin)
}

pub(super) fn require_initialized(e: &Env) -> Result<(), PolicyError> {
    if !is_initialized(e) {
        Err(PolicyError::NotInitialized)
    } else {
        Ok(())
    }
}

pub(super) fn require_not_paused(e: &Env) -> Result<(), PolicyError> {
    if get_is_paused(e) {
        Err(PolicyError::Paused)
    } else {
        Ok(())
    }
}

pub(super) fn initialize(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::Admin, admin);
    e.storage().instance().set(&DataKey::IsPaused, &false);
    e.storage().instance().set(&DataKey::PolicyList, &Vec::<BytesN<32>>::new(e));
}

pub(super) fn get_admin(e: &Env) -> Result<Address, PolicyError> {
    e.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(PolicyError::NotInitialized)
}

pub(super) fn set_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::Admin, admin);
}

pub(super) fn get_pending_admin(e: &Env) -> Option<Address> {
    e.storage().instance().get(&DataKey::PendingAdmin)
}

pub(super) fn set_pending_admin(e: &Env, pending_admin: &Address) {
    e.storage().instance().set(&DataKey::PendingAdmin, pending_admin);
}

pub(super) fn clear_pending_admin(e: &Env) {
    e.storage().instance().remove(&DataKey::PendingAdmin);
}

pub(super) fn get_is_paused(e: &Env) -> bool {
    e.storage()
        .instance()
        .get(&DataKey::IsPaused)
        .unwrap_or(false)
}

pub(super) fn set_paused(e: &Env, is_paused: &bool) {
    e.storage().instance().set(&DataKey::IsPaused, is_paused);
}

pub(super) fn has_policy(e: &Env, policy_id: &BytesN<32>) -> bool {
    e.storage().persistent().has(&DataKey::Policy(policy_id.clone()))
}

pub(super) fn get_policy(e: &Env, policy_id: &BytesN<32>) -> Result<Policy, PolicyError> {
    e.storage()
        .persistent()
        .get(&DataKey::Policy(policy_id.clone()))
        .ok_or(PolicyError::PolicyNotFound)
}

pub(super) fn set_policy(e: &Env, policy: &Policy) {
    e.storage().persistent().set(&DataKey::Policy(policy.id.clone()), policy);
    
    let mut policy_list = get_policy_list(e);
    if !policy_list.iter().any(|id| id == policy.id) {
        policy_list.push_back(policy.id.clone());
        set_policy_list(e, &policy_list);
    }
}

pub(super) fn delete_policy(e: &Env, policy_id: &BytesN<32>) {
    e.storage().persistent().remove(&DataKey::Policy(policy_id.clone()));
    
    let mut policy_list = get_policy_list(e);
    let mut new_list = Vec::new(e);
    for id in policy_list.iter() {
        if id != *policy_id {
            new_list.push_back(id);
        }
    }
    set_policy_list(e, &new_list);
}

pub(super) fn get_policy_list(e: &Env) -> Vec<BytesN<32>> {
    e.storage()
        .instance()
        .get(&DataKey::PolicyList)
        .unwrap_or_else(|| Vec::new(e))
}

pub(super) fn set_policy_list(e: &Env, list: &Vec<BytesN<32>>) {
    e.storage().instance().set(&DataKey::PolicyList, list);
}
