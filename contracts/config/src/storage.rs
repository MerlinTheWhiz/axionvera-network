use soroban_sdk::{contracttype, Address, Env};

use crate::errors::ConfigError;
use crate::types::ProtocolConfig;

// Instance storage is extended on every read so the config stays live as long
// as it is actively used by downstream contracts.
const INSTANCE_TTL_THRESHOLD: u32 = 518_400;
const INSTANCE_TTL_EXTEND_TO: u32 = 518_400;

/// Keys for all data stored in instance storage.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Initialized,
    Admin,
    PendingAdmin,
    Config,
    IsPaused,
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn extend_instance_ttl(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
}

// ---------------------------------------------------------------------------
// Lifecycle guards
// ---------------------------------------------------------------------------

pub fn is_initialized(e: &Env) -> bool {
    e.storage().instance().has(&DataKey::Initialized)
}

pub fn require_initialized(e: &Env) -> Result<(), ConfigError> {
    if !is_initialized(e) {
        return Err(ConfigError::NotInitialized);
    }
    extend_instance_ttl(e);
    Ok(())
}

pub fn require_not_paused(e: &Env) -> Result<(), ConfigError> {
    if get_is_paused(e) {
        return Err(ConfigError::ContractPaused);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Initializer
// ---------------------------------------------------------------------------

pub fn initialize(e: &Env, admin: &Address, config: &ProtocolConfig) {
    e.storage().instance().set(&DataKey::Initialized, &true);
    e.storage().instance().set(&DataKey::Admin, admin);
    e.storage().instance().set(&DataKey::Config, config);
    e.storage().instance().set(&DataKey::IsPaused, &false);
    extend_instance_ttl(e);
}

// ---------------------------------------------------------------------------
// Admin
// ---------------------------------------------------------------------------

pub fn get_admin(e: &Env) -> Result<Address, ConfigError> {
    e.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(ConfigError::NotInitialized)
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

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

pub fn get_config(e: &Env) -> Result<ProtocolConfig, ConfigError> {
    e.storage()
        .instance()
        .get(&DataKey::Config)
        .ok_or(ConfigError::NotInitialized)
}

pub fn set_config(e: &Env, config: &ProtocolConfig) {
    e.storage().instance().set(&DataKey::Config, config);
}

// ---------------------------------------------------------------------------
// Pause flag
// ---------------------------------------------------------------------------

pub fn get_is_paused(e: &Env) -> bool {
    e.storage()
        .instance()
        .get(&DataKey::IsPaused)
        .unwrap_or(false)
}

pub fn set_paused(e: &Env, paused: bool) {
    e.storage().instance().set(&DataKey::IsPaused, &paused);
}
