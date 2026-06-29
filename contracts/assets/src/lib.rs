#![no_std]

pub mod errors;
mod events;
mod storage;
pub mod types;
#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, Vec};

use crate::errors::AssetRegistryError;
use crate::types::{AssetInfo, MAX_DECIMALS, MAX_NAME_LEN, MAX_SYMBOL_LEN};

#[contract]
pub struct AssetRegistryContract;

#[contractimpl]
impl AssetRegistryContract {
    /// Returns the contract version.
    pub fn version() -> u32 {
        1
    }

    // -----------------------------------------------------------------------
    // Lifecycle
    // -----------------------------------------------------------------------

    /// One-time initializer. Sets the admin and prepares an empty registry.
    ///
    /// Requires `admin` authorization. Emits no event (use on-chain state as
    /// source of truth for the initial admin).
    pub fn initialize(e: Env, admin: Address) -> Result<(), AssetRegistryError> {
        if storage::is_initialized(&e) {
            return Err(AssetRegistryError::AlreadyInitialized);
        }
        admin.require_auth();
        storage::initialize(&e, &admin);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Asset management (admin only, writes blocked when paused)
    // -----------------------------------------------------------------------

    /// Registers a new asset in the whitelist.
    ///
    /// - `asset`: The Soroban token contract address.
    /// - `name`: Human-readable name (1–32 bytes).
    /// - `symbol`: Ticker symbol (1–12 bytes).
    /// - `decimals`: Decimal places (0–18).
    ///
    /// The asset is registered as **active** by default.
    /// Emits `AssetRegisteredEvent`.
    pub fn register_asset(
        e: Env,
        asset: Address,
        name: Bytes,
        symbol: Bytes,
        decimals: u32,
    ) -> Result<(), AssetRegistryError> {
        storage::require_initialized(&e)?;
        storage::require_not_paused(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();

        if storage::has_asset(&e, &asset) {
            return Err(AssetRegistryError::AssetAlreadyRegistered);
        }

        validate_name(&name)?;
        validate_symbol(&symbol)?;
        validate_decimals(decimals)?;

        let registered_at = e.ledger().timestamp();
        let info = AssetInfo {
            name: name.clone(),
            symbol: symbol.clone(),
            decimals,
            is_active: true,
            registered_at,
        };

        storage::set_asset_info(&e, &asset, &info);
        storage::add_to_list(&e, &asset);
        events::emit_asset_registered(&e, asset, name, symbol, decimals, admin);
        Ok(())
    }

    /// Permanently removes an asset from the whitelist.
    ///
    /// After deregistration the asset address is no longer in the registry and
    /// all metadata is deleted. Emits `AssetDeregisteredEvent`.
    pub fn deregister_asset(e: Env, asset: Address) -> Result<(), AssetRegistryError> {
        storage::require_initialized(&e)?;
        storage::require_not_paused(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();

        if !storage::has_asset(&e, &asset) {
            return Err(AssetRegistryError::AssetNotFound);
        }

        storage::remove_asset_info(&e, &asset);
        storage::remove_from_list(&e, &asset);
        events::emit_asset_deregistered(&e, asset, admin);
        Ok(())
    }

    /// Enables or disables a registered asset without removing it.
    ///
    /// Disabled assets fail `is_whitelisted` checks while remaining in the
    /// registry with their metadata intact. Emits `AssetStatusChangedEvent`.
    pub fn set_asset_status(
        e: Env,
        asset: Address,
        is_active: bool,
    ) -> Result<(), AssetRegistryError> {
        storage::require_initialized(&e)?;
        storage::require_not_paused(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();

        let mut info = storage::get_asset_info(&e, &asset)?;
        info.is_active = is_active;
        storage::set_asset_info(&e, &asset, &info);
        events::emit_asset_status_changed(&e, asset, is_active, admin);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Whitelist queries
    // -----------------------------------------------------------------------

    /// Returns `true` if the asset is registered **and** currently active.
    ///
    /// This is the primary validation function for downstream protocol contracts.
    pub fn is_whitelisted(e: Env, asset: Address) -> bool {
        if !storage::has_asset(&e, &asset) {
            return false;
        }
        storage::get_asset_info(&e, &asset)
            .map(|info| info.is_active)
            .unwrap_or(false)
    }

    /// Returns `true` if the asset is registered, regardless of active status.
    pub fn is_registered(e: Env, asset: Address) -> bool {
        storage::has_asset(&e, &asset)
    }

    /// Returns full metadata for a registered asset.
    pub fn get_asset_info(e: Env, asset: Address) -> Result<AssetInfo, AssetRegistryError> {
        storage::require_initialized(&e)?;
        storage::get_asset_info(&e, &asset)
    }

    /// Returns the list of all registered asset addresses (active and inactive).
    pub fn get_all_assets(e: Env) -> Result<Vec<Address>, AssetRegistryError> {
        storage::require_initialized(&e)?;
        Ok(storage::get_asset_list(&e))
    }

    /// Returns only the currently active (whitelisted) asset addresses.
    pub fn get_active_assets(e: Env) -> Result<Vec<Address>, AssetRegistryError> {
        storage::require_initialized(&e)?;
        let all = storage::get_asset_list(&e);
        let mut active: Vec<Address> = soroban_sdk::vec![&e];
        for asset in all.iter() {
            if let Ok(info) = storage::get_asset_info(&e, &asset) {
                if info.is_active {
                    active.push_back(asset);
                }
            }
        }
        Ok(active)
    }

    /// Returns the total number of registered assets (active and inactive).
    pub fn asset_count(e: Env) -> Result<u32, AssetRegistryError> {
        storage::require_initialized(&e)?;
        Ok(storage::get_asset_list(&e).len())
    }

    // -----------------------------------------------------------------------
    // Read — admin state
    // -----------------------------------------------------------------------

    /// Returns the current admin address.
    pub fn admin(e: Env) -> Result<Address, AssetRegistryError> {
        storage::require_initialized(&e)?;
        storage::get_admin(&e)
    }

    /// Returns the pending admin address, if a transfer is in progress.
    pub fn pending_admin(e: Env) -> Result<Option<Address>, AssetRegistryError> {
        storage::require_initialized(&e)?;
        Ok(storage::get_pending_admin(&e))
    }

    /// Returns whether the contract is currently paused.
    pub fn is_paused(e: Env) -> bool {
        storage::get_is_paused(&e)
    }

    // -----------------------------------------------------------------------
    // Admin transfer (two-step)
    // -----------------------------------------------------------------------

    /// Proposes a new admin. The transfer is not final until `accept_admin` is
    /// called by the proposed address. Emits `AssetRegistryAdminTransferProposedEvent`.
    pub fn propose_new_admin(e: Env, new_admin: Address) -> Result<(), AssetRegistryError> {
        storage::require_initialized(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();
        storage::set_pending_admin(&e, &new_admin);
        events::emit_admin_transfer_proposed(&e, admin, new_admin);
        Ok(())
    }

    /// Finalises an in-progress admin transfer. Must be called by the pending
    /// admin. Emits `AssetRegistryAdminTransferAcceptedEvent`.
    pub fn accept_admin(e: Env, new_admin: Address) -> Result<(), AssetRegistryError> {
        storage::require_initialized(&e)?;
        new_admin.require_auth();
        let previous_admin = storage::get_admin(&e)?;
        let pending = storage::get_pending_admin(&e).ok_or(AssetRegistryError::NoPendingAdmin)?;
        if pending != new_admin {
            return Err(AssetRegistryError::Unauthorized);
        }
        storage::set_admin(&e, &new_admin);
        storage::clear_pending_admin(&e);
        events::emit_admin_transfer_accepted(&e, previous_admin, new_admin);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Emergency controls
    // -----------------------------------------------------------------------

    /// Pauses all registry-write operations. Read queries remain available.
    /// Emits `AssetRegistryPausedEvent`.
    pub fn pause_contract(e: Env) -> Result<(), AssetRegistryError> {
        storage::require_initialized(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();
        storage::set_paused(&e, true);
        events::emit_paused(&e, admin);
        Ok(())
    }

    /// Resumes registry-write operations after a pause.
    /// Emits `AssetRegistryUnpausedEvent`.
    pub fn unpause_contract(e: Env) -> Result<(), AssetRegistryError> {
        storage::require_initialized(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();
        storage::set_paused(&e, false);
        events::emit_unpaused(&e, admin);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Validation helpers
// ---------------------------------------------------------------------------

fn validate_name(name: &Bytes) -> Result<(), AssetRegistryError> {
    let len = name.len();
    if len == 0 || len > MAX_NAME_LEN {
        return Err(AssetRegistryError::InvalidAssetName);
    }
    Ok(())
}

fn validate_symbol(symbol: &Bytes) -> Result<(), AssetRegistryError> {
    let len = symbol.len();
    if len == 0 || len > MAX_SYMBOL_LEN {
        return Err(AssetRegistryError::InvalidAssetSymbol);
    }
    Ok(())
}

fn validate_decimals(decimals: u32) -> Result<(), AssetRegistryError> {
    if decimals > MAX_DECIMALS {
        return Err(AssetRegistryError::InvalidDecimals);
    }
    Ok(())
}
