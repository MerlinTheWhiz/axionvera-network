#![no_std]

pub mod errors;
mod events;
mod storage;
pub mod types;
#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, Vec};

use crate::errors::FeatureError;
use crate::types::{FeatureFlag, MAX_FEATURES, MAX_ROLLOUT_PCT};

#[contract]
pub struct FeaturesContract;

#[contractimpl]
impl FeaturesContract {
    pub fn version() -> u32 {
        1
    }

    // Lifecycle

    pub fn initialize(e: Env, admin: Address) -> Result<(), FeatureError> {
        if storage::is_initialized(&e) {
            return Err(FeatureError::AlreadyInitialized);
        }
        admin.require_auth();
        storage::initialize(&e, &admin);
        events::emit_initialized(&e, admin);
        Ok(())
    }

    // Read

    pub fn admin(e: Env) -> Result<Address, FeatureError> {
        storage::require_initialized(&e)?;
        storage::get_admin(&e)
    }

    pub fn is_enabled(e: Env, name: Symbol) -> Result<bool, FeatureError> {
        storage::require_initialized(&e)?;
        let feature = storage::get_feature(&e, &name).ok_or(FeatureError::FeatureNotFound)?;
        Ok(feature.enabled)
    }

    pub fn get_feature(e: Env, name: Symbol) -> Result<FeatureFlag, FeatureError> {
        storage::require_initialized(&e)?;
        storage::get_feature(&e, &name).ok_or(FeatureError::FeatureNotFound)
    }

    pub fn list_features(e: Env) -> Result<Vec<FeatureFlag>, FeatureError> {
        storage::require_initialized(&e)?;
        let names = storage::get_feature_list(&e);
        let mut features: Vec<FeatureFlag> = Vec::new(&e);
        for name in names.iter() {
            if let Some(f) = storage::get_feature(&e, &name) {
                features.push_back(f);
            }
        }
        Ok(features)
    }

    // Feature management (admin only, blocked when paused)

    pub fn register_feature(e: Env, name: Symbol) -> Result<(), FeatureError> {
        storage::require_initialized(&e)?;
        storage::require_not_paused(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();

        if storage::get_feature(&e, &name).is_some() {
            return Err(FeatureError::FeatureAlreadyRegistered);
        }

        let mut list = storage::get_feature_list(&e);
        if list.len() >= MAX_FEATURES as u32 {
            return Err(FeatureError::FeatureLimitReached);
        }

        let ts = e.ledger().timestamp();
        let feature = FeatureFlag {
            name: name.clone(),
            enabled: false,
            rollout_pct: 0,
            registered_at: ts,
            enabled_at: None,
        };

        list.push_back(name.clone());
        storage::set_feature_list(&e, &list);
        storage::set_feature(&e, &feature);
        events::emit_feature_registered(&e, name, admin);
        Ok(())
    }

    pub fn enable_feature(e: Env, name: Symbol) -> Result<(), FeatureError> {
        storage::require_initialized(&e)?;
        storage::require_not_paused(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();

        let mut feature = storage::get_feature(&e, &name).ok_or(FeatureError::FeatureNotFound)?;
        feature.enabled = true;
        feature.enabled_at = Some(e.ledger().timestamp());
        storage::set_feature(&e, &feature);
        events::emit_feature_enabled(&e, name, admin);
        Ok(())
    }

    pub fn disable_feature(e: Env, name: Symbol) -> Result<(), FeatureError> {
        storage::require_initialized(&e)?;
        storage::require_not_paused(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();

        let mut feature = storage::get_feature(&e, &name).ok_or(FeatureError::FeatureNotFound)?;
        feature.enabled = false;
        storage::set_feature(&e, &feature);
        events::emit_feature_disabled(&e, name, admin);
        Ok(())
    }

    pub fn set_rollout(e: Env, name: Symbol, pct: u32) -> Result<(), FeatureError> {
        storage::require_initialized(&e)?;
        storage::require_not_paused(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();

        if pct > MAX_ROLLOUT_PCT {
            return Err(FeatureError::InvalidRolloutPct);
        }

        let mut feature = storage::get_feature(&e, &name).ok_or(FeatureError::FeatureNotFound)?;
        let old = feature.rollout_pct;
        feature.rollout_pct = pct;
        storage::set_feature(&e, &feature);
        events::emit_feature_rollout_updated(&e, name, admin, old, pct);
        Ok(())
    }

    // Admin transfer (two-step)

    pub fn propose_new_admin(e: Env, new_admin: Address) -> Result<(), FeatureError> {
        storage::require_initialized(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();
        storage::set_pending_admin(&e, &new_admin);
        events::emit_admin_transfer_proposed(&e, admin, new_admin);
        Ok(())
    }

    pub fn accept_admin(e: Env, new_admin: Address) -> Result<(), FeatureError> {
        storage::require_initialized(&e)?;
        new_admin.require_auth();
        let previous_admin = storage::get_admin(&e)?;
        let pending = storage::get_pending_admin(&e).ok_or(FeatureError::NoPendingAdmin)?;
        if pending != new_admin {
            return Err(FeatureError::Unauthorized);
        }
        storage::set_admin(&e, &new_admin);
        storage::clear_pending_admin(&e);
        events::emit_admin_transfer_accepted(&e, previous_admin, new_admin);
        Ok(())
    }

    // Emergency controls

    pub fn pause(e: Env) -> Result<(), FeatureError> {
        storage::require_initialized(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();
        storage::set_paused(&e, true);
        events::emit_paused(&e, admin);
        Ok(())
    }

    pub fn unpause(e: Env) -> Result<(), FeatureError> {
        storage::require_initialized(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();
        storage::set_paused(&e, false);
        events::emit_unpaused(&e, admin);
        Ok(())
    }
}
