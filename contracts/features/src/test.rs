#![cfg(test)]

use super::*;
use crate::errors::FeatureError;
use soroban_sdk::{testutils::Address as _, Address, Env, Symbol};

fn setup(e: &Env) -> (FeaturesContractClient, Address) {
    let id = e.register_contract(None, FeaturesContract {});
    let client = FeaturesContractClient::new(e, &id);
    let admin = Address::generate(e);
    (client, admin)
}

fn feature_name(e: &Env) -> Symbol {
    Symbol::new(e, "multi_asset_v2")
}

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
    assert_eq!(result, Err(Ok(FeatureError::AlreadyInitialized)));
}

#[test]
fn test_register_feature() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);
    let name = feature_name(&e);
    client.register_feature(&name);
    let f = client.get_feature(&name);
    assert_eq!(f.name, name);
    assert!(!f.enabled);
    assert_eq!(f.rollout_pct, 0);
}

#[test]
fn test_double_register_fails() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);
    let name = feature_name(&e);
    client.register_feature(&name);
    let result = client.try_register_feature(&name);
    assert_eq!(result, Err(Ok(FeatureError::FeatureAlreadyRegistered)));
}

#[test]
fn test_enable_disable() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);
    let name = feature_name(&e);
    client.register_feature(&name);
    assert!(!client.is_enabled(&name));

    client.enable_feature(&name);
    assert!(client.is_enabled(&name));
    assert!(client.get_feature(&name).enabled_at.is_some());

    client.disable_feature(&name);
    assert!(!client.is_enabled(&name));
}

#[test]
fn test_set_rollout() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);
    let name = feature_name(&e);
    client.register_feature(&name);
    client.set_rollout(&name, &50);
    assert_eq!(client.get_feature(&name).rollout_pct, 50);
}

#[test]
fn test_set_rollout_exceeds_max() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);
    let name = feature_name(&e);
    client.register_feature(&name);
    let result = client.try_set_rollout(&name, &101);
    assert_eq!(result, Err(Ok(FeatureError::InvalidRolloutPct)));
}

#[test]
fn test_feature_not_found() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);
    let name = feature_name(&e);
    let result = client.try_get_feature(&name);
    assert_eq!(result, Err(Ok(FeatureError::FeatureNotFound)));
}

#[test]
fn test_list_features() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let a = Symbol::new(&e, "feat_a");
    let b = Symbol::new(&e, "feat_b");

    client.register_feature(&a);
    client.register_feature(&b);
    client.enable_feature(&b);

    let list = client.list_features();
    assert_eq!(list.len(), 2);
    assert!(!list.get(0).unwrap().enabled);
    assert!(list.get(1).unwrap().enabled);
}

#[test]
fn test_feature_limit() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    for i in 0..MAX_FEATURES {
        let name = Symbol::new(&e, core::str::from_utf8(&[b'a' + (i % 26) as u8, b'a' + (i / 26) as u8]).unwrap());
        client.register_feature(&name);
    }

    let overflow = Symbol::new(&e, "overflow");
    let result = client.try_register_feature(&overflow);
    assert_eq!(result, Err(Ok(FeatureError::FeatureLimitReached)));
}

#[test]
fn test_propose_and_accept_admin() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);

    let new_admin = Address::generate(&e);
    client.propose_new_admin(&new_admin);
    client.accept_admin(&new_admin);
    assert_eq!(client.admin(), new_admin);
}

#[test]
fn test_pause_blocks_writes() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);
    client.pause();

    let name = feature_name(&e);
    let result = client.try_register_feature(&name);
    assert_eq!(result, Err(Ok(FeatureError::ContractPaused)));
}

#[test]
fn test_unpause_restores_writes() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin);
    client.pause();
    client.unpause();

    let name = feature_name(&e);
    client.register_feature(&name);
    assert_eq!(client.get_feature(&name).name, name);
}

#[test]
fn test_requires_auth() {
    let e = Env::default();
    let (client, admin) = setup(&e);
    let result = client.try_initialize(&admin);
    assert!(result.is_err());
}

#[test]
fn test_getters_before_init_fail() {
    let e = Env::default();
    let (client, _) = setup(&e);
    let result = client.try_admin();
    assert_eq!(result, Err(Ok(FeatureError::NotInitialized)));
}

#[test]
fn test_version() {
    assert_eq!(FeaturesContract::version(), 1);
}
