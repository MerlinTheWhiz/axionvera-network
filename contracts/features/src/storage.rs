use soroban_sdk::{contracttype, Address, Env, Symbol, Vec};

use crate::errors::FeatureError;
use crate::types::FeatureFlag;

const INSTANCE_TTL_THRESHOLD: u32 = 518_400;
const INSTANCE_TTL_EXTEND_TO: u32 = 518_400;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Initialized,
    Admin,
    PendingAdmin,
    IsPaused,
    FeatureList,
    Feature(Symbol),
}

fn extend(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
}

pub fn is_initialized(e: &Env) -> bool {
    e.storage().instance().has(&DataKey::Initialized)
}

pub fn require_initialized(e: &Env) -> Result<(), FeatureError> {
    if !is_initialized(e) {
        return Err(FeatureError::NotInitialized);
    }
    extend(e);
    Ok(())
}

pub fn require_not_paused(e: &Env) -> Result<(), FeatureError> {
    if get_is_paused(e) {
        return Err(FeatureError::ContractPaused);
    }
    Ok(())
}

pub fn initialize(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::Initialized, &true);
    e.storage().instance().set(&DataKey::Admin, admin);
    e.storage().instance().set(&DataKey::IsPaused, &false);
    let empty: Vec<Symbol> = Vec::new(e);
    e.storage().instance().set(&DataKey::FeatureList, &empty);
    extend(e);
}

pub fn get_admin(e: &Env) -> Result<Address, FeatureError> {
    e.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(FeatureError::NotInitialized)
}

pub fn set_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_pending_admin(e: &Env) -> Option<Address> {
    e.storage().instance().get(&DataKey::PendingAdmin)
}

pub fn set_pending_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::PendingAdmin, admin);
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

pub fn set_paused(e: &Env, paused: bool) {
    e.storage().instance().set(&DataKey::IsPaused, &paused);
}

pub fn get_feature_list(e: &Env) -> Vec<Symbol> {
    e.storage()
        .instance()
        .get(&DataKey::FeatureList)
        .unwrap_or_else(|| Vec::new(e))
}

pub fn set_feature_list(e: &Env, list: &Vec<Symbol>) {
    e.storage().instance().set(&DataKey::FeatureList, list);
}

pub fn get_feature(e: &Env, name: &Symbol) -> Option<FeatureFlag> {
    e.storage().instance().get(&DataKey::Feature(name.clone()))
}

pub fn set_feature(e: &Env, feature: &FeatureFlag) {
    e.storage()
        .instance()
        .set(&DataKey::Feature(feature.name.clone()), feature);
}
