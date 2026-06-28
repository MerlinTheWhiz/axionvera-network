#![cfg(test)]

use super::*;
use crate::errors::AssetRegistryError;
use crate::types::AssetInfo;
use soroban_sdk::{testutils::Address as _, Address, Bytes, Env};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn name(e: &Env, s: &[u8]) -> Bytes {
    Bytes::from_slice(e, s)
}

fn setup<'a>(e: &'a Env) -> (AssetRegistryContractClient<'a>, Address) {
    let id = e.register_contract(None, AssetRegistryContract {});
    let client = AssetRegistryContractClient::new(e, &id);
    let admin = Address::generate(e);
    (client, admin)
}

fn register_one<'a>(
    e: &'a Env,
    client: &AssetRegistryContractClient<'a>,
) -> Address {
    let asset = Address::generate(e);
    client.register_asset(
        &asset,
        &name(e, b"USD Coin"),
        &name(e, b"USDC"),
        &7,
    );
    asset
}

// ---------------------------------------------------------------------------
// Initialization
// ---------------------------------------------------------------------------

#[test]
fn test_initialize_succeeds() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);
    assert_eq!(client.admin(), admin);
}

#[test]
fn test_initialize_is_one_time() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);
    let result = client.try_initialize(&admin);
    assert_eq!(result, Err(Ok(AssetRegistryError::AlreadyInitialized)));
}

#[test]
fn test_initialize_requires_admin_auth() {
    let e = Env::default();
    // No mock_all_auths — require_auth() will reject the call
    let id = e.register_contract(None, AssetRegistryContract {});
    let client = AssetRegistryContractClient::new(&e, &id);
    let admin = Address::generate(&e);
    let result = client.try_initialize(&admin);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// register_asset
// ---------------------------------------------------------------------------

#[test]
fn test_register_asset_succeeds() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let asset = Address::generate(&e);
    client.register_asset(&asset, &name(&e, b"USD Coin"), &name(&e, b"USDC"), &7);

    assert!(client.is_registered(&asset));
    assert!(client.is_whitelisted(&asset));
    assert_eq!(client.asset_count(), 1);
}

#[test]
fn test_register_asset_stores_correct_metadata() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let asset = Address::generate(&e);
    client.register_asset(&asset, &name(&e, b"Stellar Lumens"), &name(&e, b"XLM"), &7);

    let info = client.get_asset_info(&asset);
    assert_eq!(info.name, name(&e, b"Stellar Lumens"));
    assert_eq!(info.symbol, name(&e, b"XLM"));
    assert_eq!(info.decimals, 7);
    assert!(info.is_active);
}

#[test]
fn test_register_multiple_assets() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let a1 = Address::generate(&e);
    let a2 = Address::generate(&e);
    let a3 = Address::generate(&e);

    client.register_asset(&a1, &name(&e, b"Token A"), &name(&e, b"TKA"), &6);
    client.register_asset(&a2, &name(&e, b"Token B"), &name(&e, b"TKB"), &8);
    client.register_asset(&a3, &name(&e, b"Token C"), &name(&e, b"TKC"), &18);

    assert_eq!(client.asset_count(), 3);
    let all = client.get_all_assets();
    assert_eq!(all.len(), 3);
}

#[test]
fn test_register_duplicate_asset_fails() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let asset = Address::generate(&e);
    client.register_asset(&asset, &name(&e, b"Token A"), &name(&e, b"TKA"), &6);
    let result = client.try_register_asset(&asset, &name(&e, b"Token A2"), &name(&e, b"TKA"), &6);
    assert_eq!(result, Err(Ok(AssetRegistryError::AssetAlreadyRegistered)));
}

#[test]
fn test_register_asset_rejects_empty_name() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let asset = Address::generate(&e);
    let result = client.try_register_asset(&asset, &name(&e, b""), &name(&e, b"TKA"), &6);
    assert_eq!(result, Err(Ok(AssetRegistryError::InvalidAssetName)));
}

#[test]
fn test_register_asset_rejects_name_too_long() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let asset = Address::generate(&e);
    let long_name = name(&e, b"This name is way too long for an asset whitelisting framework");
    let result = client.try_register_asset(&asset, &long_name, &name(&e, b"TKA"), &6);
    assert_eq!(result, Err(Ok(AssetRegistryError::InvalidAssetName)));
}

#[test]
fn test_register_asset_rejects_empty_symbol() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let asset = Address::generate(&e);
    let result = client.try_register_asset(&asset, &name(&e, b"Token"), &name(&e, b""), &6);
    assert_eq!(result, Err(Ok(AssetRegistryError::InvalidAssetSymbol)));
}

#[test]
fn test_register_asset_rejects_symbol_too_long() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let asset = Address::generate(&e);
    let long_sym = name(&e, b"TOOLONGSYMBOL");
    let result = client.try_register_asset(&asset, &name(&e, b"Token"), &long_sym, &6);
    assert_eq!(result, Err(Ok(AssetRegistryError::InvalidAssetSymbol)));
}

#[test]
fn test_register_asset_rejects_invalid_decimals() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let asset = Address::generate(&e);
    let result = client.try_register_asset(&asset, &name(&e, b"Token"), &name(&e, b"TKN"), &19);
    assert_eq!(result, Err(Ok(AssetRegistryError::InvalidDecimals)));
}

#[test]
fn test_register_asset_accepts_zero_decimals() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let asset = Address::generate(&e);
    client.register_asset(&asset, &name(&e, b"NFT Token"), &name(&e, b"NFT"), &0);
    assert_eq!(client.get_asset_info(&asset).decimals, 0);
}

#[test]
fn test_register_asset_accepts_max_decimals() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let asset = Address::generate(&e);
    client.register_asset(&asset, &name(&e, b"ETH Token"), &name(&e, b"WETH"), &18);
    assert_eq!(client.get_asset_info(&asset).decimals, 18);
}

// ---------------------------------------------------------------------------
// deregister_asset
// ---------------------------------------------------------------------------

#[test]
fn test_deregister_asset_removes_it() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let asset = register_one(&e, &client);
    assert_eq!(client.asset_count(), 1);

    client.deregister_asset(&asset);
    assert!(!client.is_registered(&asset));
    assert!(!client.is_whitelisted(&asset));
    assert_eq!(client.asset_count(), 0);
}

#[test]
fn test_deregister_nonexistent_asset_fails() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let unknown = Address::generate(&e);
    let result = client.try_deregister_asset(&unknown);
    assert_eq!(result, Err(Ok(AssetRegistryError::AssetNotFound)));
}

#[test]
fn test_deregister_removes_metadata() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let asset = register_one(&e, &client);
    client.deregister_asset(&asset);

    let result = client.try_get_asset_info(&asset);
    assert_eq!(result, Err(Ok(AssetRegistryError::AssetNotFound)));
}

// ---------------------------------------------------------------------------
// set_asset_status
// ---------------------------------------------------------------------------

#[test]
fn test_deactivate_asset_fails_whitelist_check() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let asset = register_one(&e, &client);
    assert!(client.is_whitelisted(&asset));

    client.set_asset_status(&asset, &false);
    assert!(!client.is_whitelisted(&asset));
    // Still registered, just inactive
    assert!(client.is_registered(&asset));
}

#[test]
fn test_reactivate_asset_passes_whitelist_check() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let asset = register_one(&e, &client);
    client.set_asset_status(&asset, &false);
    client.set_asset_status(&asset, &true);
    assert!(client.is_whitelisted(&asset));
}

#[test]
fn test_set_status_on_nonexistent_asset_fails() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let unknown = Address::generate(&e);
    let result = client.try_set_asset_status(&unknown, &false);
    assert_eq!(result, Err(Ok(AssetRegistryError::AssetNotFound)));
}

// ---------------------------------------------------------------------------
// get_active_assets / get_all_assets
// ---------------------------------------------------------------------------

#[test]
fn test_get_active_assets_excludes_inactive() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let a1 = Address::generate(&e);
    let a2 = Address::generate(&e);
    client.register_asset(&a1, &name(&e, b"Token A"), &name(&e, b"TKA"), &6);
    client.register_asset(&a2, &name(&e, b"Token B"), &name(&e, b"TKB"), &6);

    client.set_asset_status(&a2, &false);

    let active = client.get_active_assets();
    assert_eq!(active.len(), 1);

    let all = client.get_all_assets();
    assert_eq!(all.len(), 2);
}

#[test]
fn test_get_all_assets_includes_inactive() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let a1 = register_one(&e, &client);
    client.set_asset_status(&a1, &false);

    let all = client.get_all_assets();
    assert_eq!(all.len(), 1);
}

// ---------------------------------------------------------------------------
// Whitelist queries on unregistered assets
// ---------------------------------------------------------------------------

#[test]
fn test_is_whitelisted_returns_false_for_unknown_asset() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let unknown = Address::generate(&e);
    assert!(!client.is_whitelisted(&unknown));
}

#[test]
fn test_is_registered_returns_false_for_unknown_asset() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let unknown = Address::generate(&e);
    assert!(!client.is_registered(&unknown));
}

// ---------------------------------------------------------------------------
// Admin transfer
// ---------------------------------------------------------------------------

#[test]
fn test_propose_and_accept_admin_transfer() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let new_admin = Address::generate(&e);
    client.propose_new_admin(&new_admin);
    assert_eq!(client.pending_admin(), Some(new_admin.clone()));

    client.accept_admin(&new_admin);
    assert_eq!(client.admin(), new_admin);
    assert_eq!(client.pending_admin(), None);
}

#[test]
fn test_accept_admin_no_pending_fails() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let new_admin = Address::generate(&e);
    let result = client.try_accept_admin(&new_admin);
    assert_eq!(result, Err(Ok(AssetRegistryError::NoPendingAdmin)));
}

#[test]
fn test_accept_admin_wrong_address_fails() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let new_admin = Address::generate(&e);
    let wrong = Address::generate(&e);
    client.propose_new_admin(&new_admin);
    let result = client.try_accept_admin(&wrong);
    assert_eq!(result, Err(Ok(AssetRegistryError::Unauthorized)));
}

// ---------------------------------------------------------------------------
// Pause / unpause
// ---------------------------------------------------------------------------

#[test]
fn test_pause_blocks_register() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);
    client.pause_contract();
    assert!(client.is_paused());

    let asset = Address::generate(&e);
    let result = client.try_register_asset(&asset, &name(&e, b"Token"), &name(&e, b"TKN"), &6);
    assert_eq!(result, Err(Ok(AssetRegistryError::ContractPaused)));
}

#[test]
fn test_pause_blocks_deregister() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);
    let asset = register_one(&e, &client);
    client.pause_contract();

    let result = client.try_deregister_asset(&asset);
    assert_eq!(result, Err(Ok(AssetRegistryError::ContractPaused)));
}

#[test]
fn test_pause_does_not_block_reads() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);
    let asset = register_one(&e, &client);
    client.pause_contract();

    // These all read-only — must succeed while paused
    assert!(client.is_whitelisted(&asset));
    assert!(client.is_registered(&asset));
    assert_eq!(client.asset_count(), 1);
    client.get_asset_info(&asset);
    client.get_all_assets();
}

#[test]
fn test_unpause_restores_register() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);
    client.pause_contract();
    client.unpause_contract();
    assert!(!client.is_paused());

    let asset = Address::generate(&e);
    client.register_asset(&asset, &name(&e, b"Token"), &name(&e, b"TKN"), &6);
    assert!(client.is_whitelisted(&asset));
}

// ---------------------------------------------------------------------------
// version
// ---------------------------------------------------------------------------

#[test]
fn test_version_is_one() {
    assert_eq!(AssetRegistryContract::version(), 1);
}
