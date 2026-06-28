use soroban_sdk::{contracttype, vec, Address, Env, Vec};

use crate::errors::AssetRegistryError;
use crate::types::AssetInfo;

// Instance storage holds small, always-needed state (admin, flags).
const INSTANCE_TTL_THRESHOLD: u32 = 518_400;
const INSTANCE_TTL_EXTEND_TO: u32 = 518_400;

// Persistent storage holds the asset list and per-asset metadata.
const PERSISTENT_TTL_THRESHOLD: u32 = 518_400;
const PERSISTENT_TTL_EXTEND_TO: u32 = 518_400;

/// Storage keys used by the asset registry.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    // --- instance storage ---
    Initialized,
    Admin,
    PendingAdmin,
    IsPaused,
    // --- persistent storage ---
    /// Ordered list of all registered asset addresses.
    AssetList,
    /// Metadata record for a specific asset address.
    AssetInfo(Address),
}

// ---------------------------------------------------------------------------
// Internal TTL helpers
// ---------------------------------------------------------------------------

fn extend_instance(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
}

fn extend_asset_info(e: &Env, asset: &Address) {
    e.storage()
        .persistent()
        .extend_ttl(&DataKey::AssetInfo(asset.clone()), PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

fn extend_asset_list(e: &Env) {
    e.storage()
        .persistent()
        .extend_ttl(&DataKey::AssetList, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

// ---------------------------------------------------------------------------
// Lifecycle guards
// ---------------------------------------------------------------------------

pub fn is_initialized(e: &Env) -> bool {
    e.storage().instance().has(&DataKey::Initialized)
}

pub fn require_initialized(e: &Env) -> Result<(), AssetRegistryError> {
    if !is_initialized(e) {
        return Err(AssetRegistryError::NotInitialized);
    }
    extend_instance(e);
    Ok(())
}

pub fn require_not_paused(e: &Env) -> Result<(), AssetRegistryError> {
    if get_is_paused(e) {
        return Err(AssetRegistryError::ContractPaused);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Initializer
// ---------------------------------------------------------------------------

pub fn initialize(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::Initialized, &true);
    e.storage().instance().set(&DataKey::Admin, admin);
    e.storage().instance().set(&DataKey::IsPaused, &false);
    // Initialise an empty persistent asset list.
    let empty: Vec<Address> = vec![e];
    e.storage().persistent().set(&DataKey::AssetList, &empty);
    extend_instance(e);
    extend_asset_list(e);
}

// ---------------------------------------------------------------------------
// Admin
// ---------------------------------------------------------------------------

pub fn get_admin(e: &Env) -> Result<Address, AssetRegistryError> {
    e.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(AssetRegistryError::NotInitialized)
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

// ---------------------------------------------------------------------------
// Asset list
// ---------------------------------------------------------------------------

pub fn get_asset_list(e: &Env) -> Vec<Address> {
    e.storage()
        .persistent()
        .get(&DataKey::AssetList)
        .unwrap_or_else(|| vec![e])
}

fn set_asset_list(e: &Env, list: &Vec<Address>) {
    e.storage().persistent().set(&DataKey::AssetList, list);
    extend_asset_list(e);
}

pub fn add_to_list(e: &Env, asset: &Address) {
    let mut list = get_asset_list(e);
    list.push_back(asset.clone());
    set_asset_list(e, &list);
}

pub fn remove_from_list(e: &Env, asset: &Address) {
    let list = get_asset_list(e);
    let mut new_list: Vec<Address> = vec![e];
    for a in list.iter() {
        if &a != asset {
            new_list.push_back(a);
        }
    }
    set_asset_list(e, &new_list);
}

// ---------------------------------------------------------------------------
// Per-asset metadata
// ---------------------------------------------------------------------------

pub fn has_asset(e: &Env, asset: &Address) -> bool {
    e.storage()
        .persistent()
        .has(&DataKey::AssetInfo(asset.clone()))
}

pub fn get_asset_info(e: &Env, asset: &Address) -> Result<AssetInfo, AssetRegistryError> {
    let info = e
        .storage()
        .persistent()
        .get(&DataKey::AssetInfo(asset.clone()))
        .ok_or(AssetRegistryError::AssetNotFound)?;
    extend_asset_info(e, asset);
    Ok(info)
}

pub fn set_asset_info(e: &Env, asset: &Address, info: &AssetInfo) {
    e.storage()
        .persistent()
        .set(&DataKey::AssetInfo(asset.clone()), info);
    extend_asset_info(e, asset);
}

pub fn remove_asset_info(e: &Env, asset: &Address) {
    e.storage()
        .persistent()
        .remove(&DataKey::AssetInfo(asset.clone()));
}
