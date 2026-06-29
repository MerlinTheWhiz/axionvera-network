#![cfg(test)]

use super::*;
use crate::errors::ConfigError;
use crate::types::ProtocolConfig;
use soroban_sdk::{testutils::Address as _, Address, Env};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn default_config() -> ProtocolConfig {
    ProtocolConfig {
        penalty_rate_bps: 500,
        vesting_period: 86_400,
        target_deposits: 1_000_000,
        min_reward_distribution: 100_000,
        max_unlock_limit: 50,
        withdraw_unlock_limit: 5,
        max_assets: 10,
    }
}

fn setup<'a>(e: &'a Env) -> (ConfigContractClient<'a>, Address) {
    let id = e.register_contract(None, ConfigContract {});
    let client = ConfigContractClient::new(e, &id);
    let admin = Address::generate(e);
    (client, admin)
}

// ---------------------------------------------------------------------------
// Initialization
// ---------------------------------------------------------------------------

#[test]
fn test_initialize_succeeds() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin, &default_config());
    let stored = client.get_config();
    assert_eq!(stored, default_config());
}

#[test]
fn test_initialize_is_one_time() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin, &default_config());
    let result = client.try_initialize(&admin, &default_config());
    assert_eq!(result, Err(Ok(ConfigError::AlreadyInitialized)));
}

#[test]
fn test_initialize_requires_admin_auth() {
    let e = Env::default();
    let (client, admin) = setup(&e);
    let result = client.try_initialize(&admin, &default_config());
    assert!(result.is_err());
}

#[test]
fn test_initialize_rejects_invalid_penalty_rate() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    let mut config = default_config();
    config.penalty_rate_bps = 10_001;
    let result = client.try_initialize(&admin, &config);
    assert_eq!(result, Err(Ok(ConfigError::InvalidPenaltyRate)));
}

#[test]
fn test_initialize_rejects_invalid_vesting_period() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    let mut config = default_config();
    config.vesting_period = 31_536_001;
    let result = client.try_initialize(&admin, &config);
    assert_eq!(result, Err(Ok(ConfigError::InvalidVestingPeriod)));
}

#[test]
fn test_initialize_rejects_zero_target_deposits() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    let mut config = default_config();
    config.target_deposits = 0;
    let result = client.try_initialize(&admin, &config);
    assert_eq!(result, Err(Ok(ConfigError::InvalidTargetDeposits)));
}

#[test]
fn test_initialize_rejects_zero_min_reward_distribution() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    let mut config = default_config();
    config.min_reward_distribution = 0;
    let result = client.try_initialize(&admin, &config);
    assert_eq!(result, Err(Ok(ConfigError::InvalidMinRewardDistribution)));
}

#[test]
fn test_initialize_rejects_zero_max_unlock_limit() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    let mut config = default_config();
    config.max_unlock_limit = 0;
    let result = client.try_initialize(&admin, &config);
    assert_eq!(result, Err(Ok(ConfigError::InvalidMaxUnlockLimit)));
}

#[test]
fn test_initialize_rejects_max_unlock_limit_over_ceiling() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    let mut config = default_config();
    config.max_unlock_limit = 101;
    let result = client.try_initialize(&admin, &config);
    assert_eq!(result, Err(Ok(ConfigError::InvalidMaxUnlockLimit)));
}

#[test]
fn test_initialize_rejects_zero_withdraw_unlock_limit() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    let mut config = default_config();
    config.withdraw_unlock_limit = 0;
    let result = client.try_initialize(&admin, &config);
    assert_eq!(result, Err(Ok(ConfigError::InvalidWithdrawUnlockLimit)));
}

#[test]
fn test_initialize_rejects_withdraw_unlock_limit_over_ceiling() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    let mut config = default_config();
    config.withdraw_unlock_limit = 51;
    let result = client.try_initialize(&admin, &config);
    assert_eq!(result, Err(Ok(ConfigError::InvalidWithdrawUnlockLimit)));
}

#[test]
fn test_initialize_rejects_zero_max_assets() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    let mut config = default_config();
    config.max_assets = 0;
    let result = client.try_initialize(&admin, &config);
    assert_eq!(result, Err(Ok(ConfigError::InvalidMaxAssets)));
}

#[test]
fn test_initialize_rejects_max_assets_over_ceiling() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    let mut config = default_config();
    config.max_assets = 51;
    let result = client.try_initialize(&admin, &config);
    assert_eq!(result, Err(Ok(ConfigError::InvalidMaxAssets)));
}

// ---------------------------------------------------------------------------
// set_penalty_rate
// ---------------------------------------------------------------------------

#[test]
fn test_set_penalty_rate_success() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin, &default_config());
    client.set_penalty_rate(&200);
    assert_eq!(client.get_config().penalty_rate_bps, 200);
}

#[test]
fn test_set_penalty_rate_boundary_max() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin, &default_config());
    client.set_penalty_rate(&10_000);
    assert_eq!(client.get_config().penalty_rate_bps, 10_000);
}

#[test]
fn test_set_penalty_rate_boundary_zero() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin, &default_config());
    client.set_penalty_rate(&0);
    assert_eq!(client.get_config().penalty_rate_bps, 0);
}

#[test]
fn test_set_penalty_rate_exceeds_max() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin, &default_config());
    let result = client.try_set_penalty_rate(&10_001);
    assert_eq!(result, Err(Ok(ConfigError::InvalidPenaltyRate)));
}

#[test]
fn test_set_penalty_rate_requires_auth() {
    // Without mock_all_auths, any require_auth() call causes a host error.
    // This verifies the auth gate is actually present on the setter.
    let e = Env::default();
    let id = e.register_contract(None, ConfigContract {});
    let client = ConfigContractClient::new(&e, &id);
    let result = client.try_set_penalty_rate(&100);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// set_vesting_period
// ---------------------------------------------------------------------------

#[test]
fn test_set_vesting_period_success() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin, &default_config());
    client.set_vesting_period(&172_800);
    assert_eq!(client.get_config().vesting_period, 172_800);
}

#[test]
fn test_set_vesting_period_zero_allowed() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin, &default_config());
    client.set_vesting_period(&0);
    assert_eq!(client.get_config().vesting_period, 0);
}

#[test]
fn test_set_vesting_period_exceeds_max() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin, &default_config());
    let result = client.try_set_vesting_period(&31_536_001);
    assert_eq!(result, Err(Ok(ConfigError::InvalidVestingPeriod)));
}

// ---------------------------------------------------------------------------
// set_target_deposits
// ---------------------------------------------------------------------------

#[test]
fn test_set_target_deposits_success() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin, &default_config());
    client.set_target_deposits(&5_000_000);
    assert_eq!(client.get_config().target_deposits, 5_000_000);
}

#[test]
fn test_set_target_deposits_rejects_zero() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin, &default_config());
    let result = client.try_set_target_deposits(&0);
    assert_eq!(result, Err(Ok(ConfigError::InvalidTargetDeposits)));
}

#[test]
fn test_set_target_deposits_rejects_negative() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin, &default_config());
    let result = client.try_set_target_deposits(&-1);
    assert_eq!(result, Err(Ok(ConfigError::InvalidTargetDeposits)));
}

// ---------------------------------------------------------------------------
// set_min_reward_distribution
// ---------------------------------------------------------------------------

#[test]
fn test_set_min_reward_distribution_success() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin, &default_config());
    client.set_min_reward_distribution(&500_000);
    assert_eq!(client.get_config().min_reward_distribution, 500_000);
}

#[test]
fn test_set_min_reward_distribution_rejects_zero() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin, &default_config());
    let result = client.try_set_min_reward_distribution(&0);
    assert_eq!(result, Err(Ok(ConfigError::InvalidMinRewardDistribution)));
}

// ---------------------------------------------------------------------------
// set_max_unlock_limit
// ---------------------------------------------------------------------------

#[test]
fn test_set_max_unlock_limit_success() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin, &default_config());
    client.set_max_unlock_limit(&75);
    assert_eq!(client.get_config().max_unlock_limit, 75);
}

#[test]
fn test_set_max_unlock_limit_at_bounds() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin, &default_config());
    client.set_max_unlock_limit(&1);
    assert_eq!(client.get_config().max_unlock_limit, 1);
    client.set_max_unlock_limit(&100);
    assert_eq!(client.get_config().max_unlock_limit, 100);
}

#[test]
fn test_set_max_unlock_limit_rejects_zero() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin, &default_config());
    let result = client.try_set_max_unlock_limit(&0);
    assert_eq!(result, Err(Ok(ConfigError::InvalidMaxUnlockLimit)));
}

#[test]
fn test_set_max_unlock_limit_rejects_over_ceiling() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin, &default_config());
    let result = client.try_set_max_unlock_limit(&101);
    assert_eq!(result, Err(Ok(ConfigError::InvalidMaxUnlockLimit)));
}

// ---------------------------------------------------------------------------
// set_withdraw_unlock_limit
// ---------------------------------------------------------------------------

#[test]
fn test_set_withdraw_unlock_limit_success() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin, &default_config());
    client.set_withdraw_unlock_limit(&10);
    assert_eq!(client.get_config().withdraw_unlock_limit, 10);
}

#[test]
fn test_set_withdraw_unlock_limit_rejects_zero() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin, &default_config());
    let result = client.try_set_withdraw_unlock_limit(&0);
    assert_eq!(result, Err(Ok(ConfigError::InvalidWithdrawUnlockLimit)));
}

#[test]
fn test_set_withdraw_unlock_limit_rejects_over_ceiling() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin, &default_config());
    let result = client.try_set_withdraw_unlock_limit(&51);
    assert_eq!(result, Err(Ok(ConfigError::InvalidWithdrawUnlockLimit)));
}

// ---------------------------------------------------------------------------
// set_max_assets
// ---------------------------------------------------------------------------

#[test]
fn test_set_max_assets_success() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin, &default_config());
    client.set_max_assets(&25);
    assert_eq!(client.get_config().max_assets, 25);
}

#[test]
fn test_set_max_assets_rejects_zero() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin, &default_config());
    let result = client.try_set_max_assets(&0);
    assert_eq!(result, Err(Ok(ConfigError::InvalidMaxAssets)));
}

#[test]
fn test_set_max_assets_rejects_over_ceiling() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin, &default_config());
    let result = client.try_set_max_assets(&51);
    assert_eq!(result, Err(Ok(ConfigError::InvalidMaxAssets)));
}

// ---------------------------------------------------------------------------
// Admin transfer
// ---------------------------------------------------------------------------

#[test]
fn test_propose_and_accept_admin_transfer() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin, &default_config());

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
    client.initialize(&admin, &default_config());

    let new_admin = Address::generate(&e);
    let result = client.try_accept_admin(&new_admin);
    assert_eq!(result, Err(Ok(ConfigError::NoPendingAdmin)));
}

#[test]
fn test_accept_admin_wrong_address_fails() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin, &default_config());

    let new_admin = Address::generate(&e);
    let wrong = Address::generate(&e);
    client.propose_new_admin(&new_admin);
    let result = client.try_accept_admin(&wrong);
    assert_eq!(result, Err(Ok(ConfigError::Unauthorized)));
}

// ---------------------------------------------------------------------------
// Pause / unpause
// ---------------------------------------------------------------------------

#[test]
fn test_pause_blocks_writes() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin, &default_config());
    client.pause_contract();
    assert!(client.is_paused());

    let result = client.try_set_penalty_rate(&100);
    assert_eq!(result, Err(Ok(ConfigError::ContractPaused)));
}

#[test]
fn test_unpause_restores_writes() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin, &default_config());
    client.pause_contract();
    client.unpause_contract();
    assert!(!client.is_paused());
    client.set_penalty_rate(&100);
    assert_eq!(client.get_config().penalty_rate_bps, 100);
}

#[test]
fn test_pause_does_not_block_reads() {
    let e = Env::default();
    e.mock_all_auths();
    let (client, admin) = setup(&e);
    client.initialize(&admin, &default_config());
    client.pause_contract();
    let config = client.get_config();
    assert_eq!(config, default_config());
}

// ---------------------------------------------------------------------------
// Getters before initialization
// ---------------------------------------------------------------------------

#[test]
fn test_get_config_before_init_fails() {
    let e = Env::default();
    let (client, _) = setup(&e);
    let result = client.try_get_config();
    assert_eq!(result, Err(Ok(ConfigError::NotInitialized)));
}

#[test]
fn test_admin_before_init_fails() {
    let e = Env::default();
    let (client, _) = setup(&e);
    let result = client.try_admin();
    assert_eq!(result, Err(Ok(ConfigError::NotInitialized)));
}

// ---------------------------------------------------------------------------
// version
// ---------------------------------------------------------------------------

#[test]
fn test_version_is_one() {
    assert_eq!(ConfigContract::version(), 1);
}
