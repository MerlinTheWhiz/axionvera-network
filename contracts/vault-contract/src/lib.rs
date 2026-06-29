#![no_std]

pub mod cross_contract;
pub mod errors;
mod events;
mod storage;
#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};

use crate::cross_contract::CrossContractClient;
use crate::errors::{AuthorizationError, BalanceError, StateError, ValidationError, VaultError};
mod access;
pub mod cross_contract;
pub mod errors;
pub mod events;
pub mod storage;
#[cfg(test)]
mod test;



use soroban_sdk::symbol_short;
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};

use axionvera_accounting as accounting;
use axionvera_fees as fee_framework;
use axionvera_interfaces::{FeeConfig, FeeReceipt, FeeTotals, FeeType};
use axionvera_storage as protocol_storage;

use crate::cross_contract::CrossContractClient;
use axionvera_risk::{RiskManagement, RiskManagementClient, RiskParameters};
use crate::errors::{
     BalanceError, DelegationError, StateError, ValidationError, VaultError,
};

const DELEGATE_PERM_DEPOSIT: u32 = 1 << 0;
const DELEGATE_PERM_WITHDRAW: u32 = 1 << 1;
const DELEGATE_PERM_CLAIM: u32 = 1 << 2;

#[contract]
pub struct VaultContract;

#[contractimpl]
impl VaultContract {
    pub fn initialize_risk(e: Env, admin: Address, params: RiskParameters) {
        let admin_stored = storage::get_admin(&e).unwrap();
        admin.require_auth();
        if admin != admin_stored {
            panic!("Unauthorized");
        }
        e.storage().instance().set(&axionvera_risk::DataKey::RiskParams, &params);
        e.storage().instance().set(&axionvera_risk::DataKey::CurrentTotalDeposits, &0_i128);
    }

    pub fn set_risk_params(e: Env, admin: Address, params: RiskParameters) {
        admin.require_auth();
        let stored_admin: Address = storage::get_admin(&e).unwrap();
        if admin != stored_admin {
            panic!("Unauthorized");
        }
        e.storage().instance().set(&axionvera_risk::DataKey::RiskParams, &params);
    }

    pub fn get_risk_params(e: Env) -> RiskParameters {
        e.storage().instance().get(&axionvera_risk::DataKey::RiskParams).unwrap()
    }

    pub fn check_deposit(e: Env, amount: i128) -> Result<(), axionvera_risk::RiskError> {
        let params: RiskParameters = e.storage().instance().get(&axionvera_risk::DataKey::RiskParams).unwrap_or(RiskParameters {
            max_deposit_amount: 0,
            min_deposit_amount: 0,
            max_withdrawal_amount: 0,
            global_cap: 0,
        });
        let current_total: i128 = e.storage().instance().get(&axionvera_risk::DataKey::CurrentTotalDeposits).unwrap_or(0);

        if amount < params.min_deposit_amount {
            return Err(axionvera_risk::RiskError::TooSmall);
        }
        if amount > params.max_deposit_amount && params.max_deposit_amount > 0 {
            return Err(axionvera_risk::RiskError::TooLarge);
        }
        if current_total + amount > params.global_cap && params.global_cap > 0 {
            return Err(axionvera_risk::RiskError::CapReached);
        }
        Ok(())
    }

    pub fn check_withdrawal(e: Env, amount: i128) -> Result<(), axionvera_risk::RiskError> {
        let params: RiskParameters = e.storage().instance().get(&axionvera_risk::DataKey::RiskParams).unwrap_or(RiskParameters {
            max_deposit_amount: 0,
            min_deposit_amount: 0,
            max_withdrawal_amount: 0,
            global_cap: 0,
        });
        if amount > params.max_withdrawal_amount && params.max_withdrawal_amount > 0 {
            return Err(axionvera_risk::RiskError::TooLarge);
        }
        Ok(())
    }

    pub fn update_total_deposits(e: Env, delta: i128) {
        let current_total: i128 = e.storage().instance().get(&axionvera_risk::DataKey::CurrentTotalDeposits).unwrap_or(0);
        e.storage().instance().set(&axionvera_risk::DataKey::CurrentTotalDeposits, &(current_total + delta));
    }

    pub fn version() -> u32 {
        1
    }

    pub fn initialize(
        e: Env,
        admin: Address,
        deposit_token: Address,
        reward_token: Address,
        vesting_period: u64,
    ) -> Result<(), VaultError> {
        storage::require_not_paused(&e)?;
        if storage::is_initialized(&e) {
            return Err(StateError::AlreadyInitialized.into());
        }

        validate_distinct_token_addresses(&deposit_token, &reward_token)?;

        access::require_actor(&admin)?;

        let target_deposits = 0_i128;
        let utilization_multipliers = soroban_sdk::Vec::new(&e);
        storage::initialize_state(
            &e,
            &admin,
            &deposit_token,
            &reward_token,
            vesting_period,
            target_deposits,
            &utilization_multipliers,
        );
        account_operation(
            &e,
            accounting::AccountingCategory::Governance,
            accounting::AccountingOperation::Initialize,
            Some(admin.clone()),
            None,
            0,
            0,
            0,
            accounting::OperationResources::new(1, 10, 2, 0),
        )?;
        events::emit_initialize(&e, admin, deposit_token, reward_token);

        Ok(())
    }

    pub fn propose_new_admin(e: Env, new_admin: Address) -> Result<(), VaultError> {
        storage::require_initialized(&e)?;

        let admin = storage::get_admin(&e)?;
        access::require_stored_admin(&admin)?;

        storage::set_pending_admin(&e, &new_admin);
        account_operation(
            &e,
            accounting::AccountingCategory::Governance,
            accounting::AccountingOperation::GovernanceAdminPropose,
            Some(admin.clone()),
            None,
            0,
            0,
            0,
            accounting::OperationResources::new(2, 1, 2, 0),
        )?;
        events::emit_admin_transfer_proposed(&e, admin, new_admin);

        Ok(())
    }

    pub fn accept_admin(e: Env, new_admin: Address) -> Result<(), VaultError> {
        storage::require_initialized(&e)?;

        let previous_admin = storage::get_admin(&e)?;
        let pending_admin = storage::get_pending_admin(&e)?.ok_or(StateError::NoPendingAdmin)?;
        access::require_pending_admin(&new_admin, Some(pending_admin.clone()))?;

        storage::set_admin(&e, &new_admin);
        storage::clear_pending_admin(&e);
        account_operation(
            &e,
            accounting::AccountingCategory::Governance,
            accounting::AccountingOperation::GovernanceAdminAccept,
            Some(new_admin.clone()),
            None,
            0,
            0,
            0,
            accounting::OperationResources::new(3, 2, 2, 0),
        )?;
        events::emit_admin_transfer_accepted(&e, previous_admin, new_admin);

        Ok(())
    }

    pub fn deposit(e: Env, from: Address, amount: i128) -> Result<(), VaultError> {
        storage::require_not_paused(&e)?;
        storage::require_initialized(&e)?;
        validate_positive_amount(amount)?;
        access::require_actor(&from)?;

        Self::check_deposit(e.clone(), amount).map_err(|_| VaultError::OperationLimitExceeded)?;

        with_non_reentrant(&e, || {
            let deposit_token = storage::get_deposit_token(&e)?;
            CrossContractClient::token_transfer(
                &e,
                &deposit_token,
                &from,
                &e.current_contract_address(),
                amount,
            )?;

            let net_amount = collect_protocol_fee(
                &e,
                FeeType::Deposit,
                from.clone(),
                Some(deposit_token.clone()),
                amount,
                accounting::OperationResources::new(1, 1, 1, 1),
            )?
            .unwrap_or(amount);

            let (_state, _position) = storage::store_deposit(&e, &from, net_amount)?;
            let (state, _position) = storage::store_deposit(&e, &from, amount)?;
            account_operation(
                &e,
                accounting::AccountingCategory::Vault,
                accounting::AccountingOperation::VaultDeposit,
                Some(from.clone()),
                Some(deposit_token.clone()),
                net_amount,
                0,
                net_amount,
                accounting::OperationResources::new(5, 5, 2, 1),
            )?;
            events::emit_deposit(&e, from.clone(), net_amount);
            Ok(())
        })
    }

    pub fn authorize_delegate(
        e: Env,
        owner: Address,
        delegate: Address,
        permissions: u32,
    ) -> Result<(), VaultError> {
        storage::require_initialized(&e)?;
        owner.require_auth();
        if permissions == 0 {
            return Err(ValidationError::InvalidAddress.into());
        }

        storage::authorize_delegate(&e, &owner, &delegate, permissions)?;
        events::emit_delegate_authorized(&e, owner, delegate, permissions);
        Ok(())
    }

    pub fn revoke_delegate(e: Env, owner: Address, delegate: Address) -> Result<(), VaultError> {
        storage::require_initialized(&e)?;
        owner.require_auth();

        storage::revoke_delegate(&e, &owner, &delegate)?;
        events::emit_delegate_revoked(&e, owner, delegate);
        Ok(())
    }

    pub fn deposit_as_delegate(
        e: Env,
        owner: Address,
        delegate: Address,
        amount: i128,
    ) -> Result<(), VaultError> {
        storage::require_not_paused(&e)?;
        storage::require_initialized(&e)?;
        validate_positive_amount(amount)?;
        delegate.require_auth();

        storage::require_delegate_permission(&e, &owner, &delegate, DELEGATE_PERM_DEPOSIT)?;

        Self::check_deposit(e.clone(), amount).map_err(|_| VaultError::OperationLimitExceeded)?;

        with_non_reentrant(&e, || {
            let state = storage::get_state(&e)?;
            CrossContractClient::token_transfer(
                &e,
                &state.deposit_token,
                &delegate,
                &e.current_contract_address(),
                amount,
            )?;

            let net_amount = collect_protocol_fee(
                &e,
                FeeType::Deposit,
                owner.clone(),
                Some(state.deposit_token.clone()),
                amount,
                accounting::OperationResources::new(1, 1, 1, 1),
            )?
            .unwrap_or(amount);

            let (_state, _position) = storage::store_deposit(&e, &owner, net_amount)?;
            events::emit_deposit(&e, owner.clone(), net_amount);
            events::emit_delegate_action(&e, owner.clone(), delegate.clone(), symbol_short!("deposit"));
            let (_state, _position) = storage::store_deposit(&e, &owner, amount)?;
            Self::update_total_deposits(e.clone(), amount);
            events::emit_deposit(&e, owner.clone(), amount);
            events::emit_delegate_action(
                &e,
                owner.clone(),
                delegate.clone(),
                symbol_short!("deposit"),
            );
            Ok(())
        })
    }

    pub fn withdraw(e: Env, to: Address, amount: i128) -> Result<(), VaultError> {
        storage::require_not_paused(&e)?;
        storage::require_initialized(&e)?;
        validate_positive_amount(amount)?;
        access::require_actor(&to)?;

        Self::check_withdrawal(e.clone(), amount).map_err(|_| VaultError::OperationLimitExceeded)?;

        with_non_reentrant(&e, || {
            let (state, position) = storage::store_withdraw(&e, &to, amount)?;
            let deposit_token = state.deposit_token.clone();

            account_operation(
                &e,
                accounting::AccountingCategory::Vault,
                accounting::AccountingOperation::VaultWithdraw,
                Some(to.clone()),
                Some(state.deposit_token.clone()),
                0,
                net_amount,
                net_amount,
                accounting::OperationResources::new(6, 5, 2, 1),
            )?;
            Self::update_total_deposits(e.clone(), -amount);
            events::emit_withdraw(&e, to.clone(), amount, position.balance);

            let deposit_token = storage::get_deposit_token(&e)?;
            CrossContractClient::token_transfer(
                &e,
                &state.deposit_token,
                &e.current_contract_address(),
                &to,
                net_amount,
            )?;

            Ok(())
        })
    }

    pub fn withdraw_as_delegate(
        e: Env,
        owner: Address,
        delegate: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), VaultError> {
        storage::require_not_paused(&e)?;
        storage::require_initialized(&e)?;
        validate_positive_amount(amount)?;
        delegate.require_auth();

        storage::require_delegate_permission(&e, &owner, &delegate, DELEGATE_PERM_WITHDRAW)?;

        Self::check_withdrawal(e.clone(), amount).map_err(|_| VaultError::OperationLimitExceeded)?;

        with_non_reentrant(&e, || {
            let _state = storage::get_state(&e)?;
            let (state, position) = storage::store_withdraw(&e, &owner, amount)?;
            Self::update_total_deposits(e.clone(), -amount);

            events::emit_withdraw(&e, owner.clone(), net_amount, position.balance);
            events::emit_delegate_action(&e, owner.clone(), delegate.clone(), symbol_short!("withdraw"));
            events::emit_withdraw(&e, owner.clone(), amount, position.balance);
            events::emit_delegate_action(
                &e,
                owner.clone(),
                delegate.clone(),
                symbol_short!("withdraw"),
            );

            CrossContractClient::token_transfer(
                &e,
                &state.deposit_token,
                &e.current_contract_address(),
                &to,
                net_amount,
            )?;

            Ok(())
        })
    }

    pub fn distribute_rewards(e: Env, amount: i128) -> Result<i128, VaultError> {
        storage::require_initialized(&e)?;
        validate_positive_amount(amount)?;

        const MIN_REWARD_DISTRIBUTION: i128 = 100_000;
        if amount < MIN_REWARD_DISTRIBUTION {
            return Err(ValidationError::InsufficientRewardAmount.into());
        }

        let admin = storage::get_admin(&e)?;
        let reward_token_id = storage::get_reward_token(&e)?;

        access::require_stored_admin(&admin)?;

        with_non_reentrant(&e, || {
            CrossContractClient::token_transfer(
                &e,
                &reward_token_id,
                &admin,
                &e.current_contract_address(),
                amount,
            )?;

            let net_amount = collect_protocol_fee(
                &e,
                FeeType::Reward,
                admin.clone(),
                Some(reward_token_id.clone()),
                amount,
                accounting::OperationResources::new(1, 1, 1, 1),
            )?
            .unwrap_or(amount);

            let next_state = storage::store_reward_distribution(&e, net_amount)?;
            account_operation(
                &e,
                accounting::AccountingCategory::Rewards,
                accounting::AccountingOperation::RewardDistribute,
                Some(admin.clone()),
                Some(reward_token_id.clone()),
                net_amount,
                0,
                net_amount,
                accounting::OperationResources::new(4, 2, 2, 1),
            )?;
            events::emit_distribute(&e, admin.clone(), net_amount);
            Ok(next_state.reward_index)
        })
    }

    pub fn lock(
        e: Env,
        from: Address,
        amount: i128,
        duration_seconds: u64,
    ) -> Result<(), VaultError> {
        storage::require_not_paused(&e)?;
        storage::require_initialized(&e)?;
        validate_positive_amount(amount)?;
        if duration_seconds == 0 {
            return Err(ValidationError::InvalidLockDuration.into());
        }
        access::require_actor(&from)?;

        with_non_reentrant(&e, || {
            let unlock_timestamp = e
                .ledger()
                .timestamp()
                .checked_add(duration_seconds)
                .ok_or(VaultError::MathOverflow)?;
            storage::store_lock(&e, &from, amount, duration_seconds)?;
            let deposit_token = storage::get_deposit_token(&e)?;
            account_operation(
                &e,
                accounting::AccountingCategory::Vault,
                accounting::AccountingOperation::VaultLock,
                Some(from.clone()),
                Some(deposit_token),
                0,
                0,
                amount,
                accounting::OperationResources::new(4, 3, 2, 0),
            )?;
            events::emit_lock(&e, from, amount, unlock_timestamp);
            Ok(())
        })
    }

    pub fn unlock_expired(e: Env, user: Address, limit: u32) -> Result<i128, VaultError> {
        storage::require_not_paused(&e)?;
        storage::require_initialized(&e)?;
        access::require_actor(&user)?;

        // Enforce a maximum limit to prevent budget exhaustion in a single call.
        const MAX_UNLOCK_LIMIT: u32 = 50;
        if limit > MAX_UNLOCK_LIMIT {
            return Err(VaultError::OperationLimitExceeded);
        }

        with_non_reentrant(&e, || {
            let unlocked_amount = storage::unlock_expired_locks(&e, &user, limit)?;
            if unlocked_amount > 0 {
                let deposit_token = storage::get_deposit_token(&e)?;
                account_operation(
                    &e,
                    accounting::AccountingCategory::Vault,
                    accounting::AccountingOperation::VaultUnlock,
                    Some(user.clone()),
                    Some(deposit_token),
                    0,
                    0,
                    unlocked_amount,
                    accounting::OperationResources::new(3, 2, 2, 0),
                )?;
                events::emit_unlock(&e, user, unlocked_amount);
            }
            Ok(unlocked_amount)
        })
    }

    pub fn claim_rewards(e: Env, user: Address) -> Result<i128, VaultError> {
        storage::require_not_paused(&e)?;
        storage::require_initialized(&e)?;
        access::require_actor(&user)?;

        with_non_reentrant(&e, || {
            let amt = storage::store_claimable_rewards(&e, &user)?;
            if amt <= 0 {
                return Ok(0);
            }

            let total_deposits = storage::get_total_deposits(&e)?;
            update_protocol_metrics(&e, total_deposits, 0)?;

            let reward_token_id = storage::get_reward_token(&e)?;
            let contract_balance = CrossContractClient::token_balance(
                &e,
                &reward_token_id,
                &e.current_contract_address(),
            )?;
            ensure_contract_balance(contract_balance, amt)?;
            CrossContractClient::token_transfer(
                &e,
                &reward_token_id,
                &e.current_contract_address(),
                &user,
                amt,
            )?;

            account_operation(
                &e,
                accounting::AccountingCategory::Rewards,
                accounting::AccountingOperation::RewardClaim,
                Some(user.clone()),
                Some(reward_token_id),
                0,
                amt,
                amt,
                accounting::OperationResources::new(5, 3, 2, 1),
            )?;
            events::emit_claim_rewards(&e, user.clone(), amt);
            let remaining = storage::pending_user_rewards_view(&e, &user)?;
            events::emit_vesting_claimed(&e, user, None, amt, remaining);
            Ok(amt)
        })
    }

    pub fn claim_rewards_as_delegate(
        e: Env,
        owner: Address,
        delegate: Address,
    ) -> Result<i128, VaultError> {
        storage::require_not_paused(&e)?;
        storage::require_initialized(&e)?;
        delegate.require_auth();

        storage::require_delegate_permission(&e, &owner, &delegate, DELEGATE_PERM_CLAIM)?;

        with_non_reentrant(&e, || {
            let amt = storage::store_claimable_rewards(&e, &owner)?;
            if amt <= 0 {
                return Ok(0);
            }

            let total_deposits = storage::get_total_deposits(&e)?;
            update_protocol_metrics(&e, total_deposits, 0)?;

            let reward_token_id = storage::get_reward_token(&e)?;
            let contract_balance = CrossContractClient::token_balance(
                &e,
                &reward_token_id,
                &e.current_contract_address(),
            )?;
            ensure_contract_balance(contract_balance, amt)?;
            CrossContractClient::token_transfer(
                &e,
                &reward_token_id,
                &e.current_contract_address(),
                &owner,
                amt,
            )?;

            events::emit_claim_rewards(&e, owner.clone(), amt);
            events::emit_delegate_action(
                &e,
                owner.clone(),
                delegate.clone(),
                symbol_short!("claim"),
            );
            Ok(amt)
        })
    }

    pub fn balance(e: Env, user: Address) -> Result<i128, VaultError> {
        storage::get_user_balance(&e, &user)
    }

    pub fn liquid_balance(e: Env, user: Address) -> Result<i128, VaultError> {
        storage::get_liquid_balance(&e, &user)
    }

    pub fn locked_balance(e: Env, user: Address) -> Result<i128, VaultError> {
        storage::get_locked_balance(&e, &user)
    }

    pub fn total_deposits(e: Env) -> Result<i128, VaultError> {
        storage::get_total_deposits(&e)
    }

    pub fn reward_index(e: Env) -> Result<i128, VaultError> {
        storage::get_reward_index(&e)
    }

    pub fn delegate_permissions(
        e: Env,
        owner: Address,
        delegate: Address,
    ) -> Result<u32, VaultError> {
        storage::get_delegate_permissions(&e, &owner, &delegate)
    }

    pub fn pending_rewards(e: Env, user: Address) -> Result<i128, VaultError> {
        storage::pending_user_rewards_view(&e, &user)
    }

    pub fn vested_rewards(e: Env, user: Address) -> Result<i128, VaultError> {
        storage::vested_user_rewards_view(&e, &user)
    }

    pub fn vesting_period(e: Env) -> Result<u64, VaultError> {
        storage::get_vesting_period(&e)
    }

    pub fn admin(e: Env) -> Result<Address, VaultError> {
        storage::get_admin(&e)
    }

    pub fn pending_admin(e: Env) -> Result<Option<Address>, VaultError> {
        storage::get_pending_admin(&e)
    }

    pub fn deposit_token(e: Env) -> Result<Address, VaultError> {
        storage::get_deposit_token(&e)
    }

    pub fn reward_token(e: Env) -> Result<Address, VaultError> {
        storage::get_reward_token(&e)
    }

    pub fn weighted_total_deposits(e: Env) -> Result<i128, VaultError> {
        storage::get_weighted_total_deposits(&e)
    }

    pub fn lock_duration_models(
        e: Env,
    ) -> Result<soroban_sdk::Vec<storage::LockDurationModel>, VaultError> {
        storage::require_initialized(&e)?;
        Ok(storage::get_lock_duration_models(&e))
    }

    pub fn set_lock_duration_models(
        e: Env,
        admin: Address,
        models: soroban_sdk::Vec<storage::LockDurationModel>,
    ) -> Result<(), VaultError> {
        storage::require_initialized(&e)?;
        let stored_admin = storage::get_admin(&e)?;
        if admin != stored_admin {
            return Err(AuthorizationError::Unauthorized.into());
        }
        admin.require_auth();

        storage::validate_lock_duration_models(&models)?;
        storage::set_lock_duration_models(&e, &models);
        Ok(())
    }

    pub fn pause_contract(e: Env) -> Result<(), VaultError> {
        storage::require_initialized(&e)?;
        let admin = storage::get_admin(&e)?;
        access::require_stored_admin(&admin)?;
        storage::set_paused(&e, true);
        account_operation(
            &e,
            accounting::AccountingCategory::Governance,
            accounting::AccountingOperation::GovernancePause,
            Some(admin.clone()),
            None,
            0,
            0,
            0,
            accounting::OperationResources::new(2, 1, 2, 0),
        )?;
        events::emit_pause(&e, admin);
        Ok(())
    }

    pub fn unpause_contract(e: Env) -> Result<(), VaultError> {
        storage::require_initialized(&e)?;
        let admin = storage::get_admin(&e)?;
        access::require_stored_admin(&admin)?;
        storage::set_paused(&e, false);
        account_operation(
            &e,
            accounting::AccountingCategory::Governance,
            accounting::AccountingOperation::GovernanceUnpause,
            Some(admin.clone()),
            None,
            0,
            0,
            0,
            accounting::OperationResources::new(2, 1, 2, 0),
        )?;
        events::emit_unpause(&e, admin);
        Ok(())
    }

    pub fn set_penalty_rate(e: Env, admin: Address, rate_bps: u32) -> Result<(), VaultError> {
        storage::require_initialized(&e)?;
        let stored_admin = storage::get_admin(&e)?;
        access::require_admin(&admin, &stored_admin)?;
        if rate_bps > 10000 {
            return Err(ValidationError::InvalidPenaltyRate.into());
        }
        storage::set_penalty_rate_bps(&e, rate_bps);
        account_operation(
            &e,
            accounting::AccountingCategory::Governance,
            accounting::AccountingOperation::GovernanceSetParameter,
            Some(admin.clone()),
            None,
            0,
            0,
            0,
            accounting::OperationResources::new(2, 1, 1, 0),
        )?;
        Ok(())
    }

    pub fn penalty_rate(e: Env) -> Result<u32, VaultError> {
        storage::get_penalty_rate_bps(&e)
    }

    pub fn total_penalties(e: Env) -> Result<i128, VaultError> {
        storage::get_total_penalties(&e)
    }

    pub fn user_penalties(e: Env, user: Address) -> Result<i128, VaultError> {
        storage::get_user_penalty_paid(&e, &user)
    }

    pub fn withdraw_locked_early(e: Env, to: Address, amount: i128) -> Result<i128, VaultError> {
        storage::require_not_paused(&e)?;
        storage::require_initialized(&e)?;
        validate_positive_amount(amount)?;
        access::require_actor(&to)?;

        Self::check_withdrawal(e.clone(), amount).map_err(|_| VaultError::OperationLimitExceeded)?;

        with_non_reentrant(&e, || {
            let (state, position, net_amount, penalty) =
                storage::store_early_withdraw_locked(&e, &to, amount)?;
            update_protocol_metrics(&e, state.total_deposits, 0)?;
            account_operation(
                &e,
                accounting::AccountingCategory::Vault,
                accounting::AccountingOperation::VaultEarlyWithdraw,
                Some(to.clone()),
                Some(state.deposit_token.clone()),
                0,
                net_amount,
                amount,
                accounting::OperationResources::new(7, 6, if penalty > 0 { 3 } else { 2 }, 1),
            )?;
            if penalty > 0 {
                account_operation(
                    &e,
                    accounting::AccountingCategory::Treasury,
                    accounting::AccountingOperation::TreasuryPenalty,
                    Some(to.clone()),
                    Some(state.deposit_token.clone()),
                    penalty,
                    0,
                    penalty,
                    accounting::OperationResources::new(2, 2, 1, 0),
                )?;
            }
            events::emit_withdraw(&e, to.clone(), net_amount, position.balance);
            Self::update_total_deposits(e.clone(), -amount);
            CrossContractClient::token_transfer(
                &e,
                &state.deposit_token,
                &e.current_contract_address(),
                &to,
                net_amount,
            )?;
            Ok(net_amount)
        })
    }

    pub fn upgrade(e: Env, new_wasm_hash: BytesN<32>) -> Result<(), VaultError> {
        storage::require_initialized(&e)?;
        let admin = storage::get_admin(&e)?;
        access::require_stored_admin(&admin)?;

        e.deployer()
            .update_current_contract_wasm(new_wasm_hash.clone());
        account_operation(
            &e,
            accounting::AccountingCategory::Governance,
            accounting::AccountingOperation::GovernanceUpgrade,
            Some(admin.clone()),
            None,
            0,
            0,
            0,
            accounting::OperationResources::new(2, 1, 2, 0),
        )?;
        events::emit_upgrade(&e, admin, new_wasm_hash);

        Ok(())
    }

    // ---------------------------------------------------------------------------
    // Multi-Asset Functions
    // ---------------------------------------------------------------------------

    pub fn add_asset(e: Env, admin: Address, asset: Address) -> Result<(), VaultError> {
        storage::require_initialized(&e)?;
        let stored_admin = storage::get_admin(&e)?;
        access::require_admin(&admin, &stored_admin)?;

        storage::add_supported_asset(&e, &asset)?;
        account_operation(
            &e,
            accounting::AccountingCategory::Governance,
            accounting::AccountingOperation::AssetAdded,
            Some(admin.clone()),
            Some(asset.clone()),
            0,
            0,
            0,
            accounting::OperationResources::new(3, 2, 2, 0),
        )?;
        events::emit_asset_added(&e, asset);
        Ok(())
    }

    pub fn deposit_asset(
        e: Env,
        from: Address,
        asset: Address,
        amount: i128,
    ) -> Result<(), VaultError> {
        storage::require_not_paused(&e)?;
        storage::require_initialized(&e)?;
        validate_positive_amount(amount)?;
        access::require_actor(&from)?;

        if !storage::is_asset_supported(&e, &asset) {
            return Err(ValidationError::InvalidAddress.into());
        }

        Self::check_deposit(e.clone(), amount).map_err(|_| VaultError::OperationLimitExceeded)?;

        with_non_reentrant(&e, || {
            CrossContractClient::token_transfer(
                &e,
                &asset,
                &from,
                &e.current_contract_address(),
                amount,
            )?;

            let _position = storage::store_asset_deposit(&e, &from, &asset, amount)?;
            let total_deposits = storage::get_total_deposits(&e)?;
            update_protocol_metrics(&e, total_deposits, 0)?;
            account_operation(
                &e,
                accounting::AccountingCategory::Vault,
                accounting::AccountingOperation::AssetDeposit,
                Some(from.clone()),
                Some(asset.clone()),
                amount,
                0,
                amount,
                accounting::OperationResources::new(5, 4, 2, 1),
            )?;
            events::emit_asset_deposit(&e, from.clone(), asset.clone(), amount);
            Ok(())
        })
    }

    pub fn withdraw_asset(
        e: Env,
        to: Address,
        asset: Address,
        amount: i128,
    ) -> Result<(), VaultError> {
        storage::require_not_paused(&e)?;
        storage::require_initialized(&e)?;
        validate_positive_amount(amount)?;
        access::require_actor(&to)?;

        if !storage::is_asset_supported(&e, &asset) {
            return Err(ValidationError::InvalidAddress.into());
        }

        Self::check_withdrawal(e.clone(), amount).map_err(|_| VaultError::OperationLimitExceeded)?;

        with_non_reentrant(&e, || {
            let position = storage::store_asset_withdraw(&e, &to, &asset, amount)?;
            let total_deposits = storage::get_total_deposits(&e)?;
            update_protocol_metrics(&e, total_deposits, 0)?;

            account_operation(
                &e,
                accounting::AccountingCategory::Vault,
                accounting::AccountingOperation::AssetWithdraw,
                Some(to.clone()),
                Some(asset.clone()),
                0,
                amount,
                amount,
                accounting::OperationResources::new(5, 4, 2, 1),
            )?;
            events::emit_asset_withdraw(&e, to.clone(), asset.clone(), amount, position.balance);

            CrossContractClient::token_transfer(
                &e,
                &asset,
                &e.current_contract_address(),
                &to,
                amount,
            )?;

            Ok(())
        })
    }

    pub fn distribute_rewards_for_asset(
        e: Env,
        admin: Address,
        asset: Address,
        amount: i128,
    ) -> Result<i128, VaultError> {
        storage::require_initialized(&e)?;
        validate_positive_amount(amount)?;

        const MIN_REWARD_DISTRIBUTION: i128 = 100_000;
        if amount < MIN_REWARD_DISTRIBUTION {
            return Err(ValidationError::InsufficientRewardAmount.into());
        }

        if !storage::is_asset_supported(&e, &asset) {
            return Err(ValidationError::InvalidAddress.into());
        }

        let state = storage::get_state(&e)?;
        let stored_admin = state.admin.clone();
        let reward_token_id = state.reward_token.clone();
        access::require_admin(&admin, &stored_admin)?;

        with_non_reentrant(&e, || {
            CrossContractClient::token_transfer(
                &e,
                &reward_token_id,
                &admin,
                &e.current_contract_address(),
                amount,
            )?;

            let net_amount = collect_protocol_fee(
                &e,
                FeeType::Reward,
                admin.clone(),
                Some(reward_token_id.clone()),
                amount,
                accounting::OperationResources::new(1, 1, 1, 1),
            )?
            .unwrap_or(amount);

            let next_reward_index = storage::store_asset_reward_distribution(&e, &asset, net_amount)?;
            account_operation(
                &e,
                accounting::AccountingCategory::Rewards,
                accounting::AccountingOperation::AssetRewardDistribute,
                Some(admin.clone()),
                Some(asset.clone()),
                net_amount,
                0,
                net_amount,
                accounting::OperationResources::new(5, 3, 2, 1),
            )?;
            events::emit_asset_distribute(&e, admin.clone(), asset.clone(), net_amount);
            Ok(next_reward_index)
        })
    }

    pub fn claim_rewards_for_asset(
        e: Env,
        user: Address,
        asset: Address,
    ) -> Result<i128, VaultError> {
        storage::require_not_paused(&e)?;
        storage::require_initialized(&e)?;
        access::require_actor(&user)?;

        if !storage::is_asset_supported(&e, &asset) {
            return Err(ValidationError::InvalidAddress.into());
        }

        with_non_reentrant(&e, || {
            let amt = storage::store_asset_claimable_rewards(&e, &user, &asset)?;
            if amt <= 0 {
                return Ok(0);
            }

            let total_deposits = storage::get_total_deposits(&e)?;
            update_protocol_metrics(&e, total_deposits, 0)?;

            let reward_token_id = storage::get_reward_token(&e)?;
            let contract_balance = CrossContractClient::token_balance(
                &e,
                &reward_token_id,
                &e.current_contract_address(),
            )?;
            ensure_contract_balance(contract_balance, amt)?;
            CrossContractClient::token_transfer(
                &e,
                &reward_token_id,
                &e.current_contract_address(),
                &user,
                amt,
            )?;

            account_operation(
                &e,
                accounting::AccountingCategory::Rewards,
                accounting::AccountingOperation::AssetRewardClaim,
                Some(user.clone()),
                Some(asset.clone()),
                0,
                amt,
                amt,
                accounting::OperationResources::new(5, 3, 2, 1),
            )?;
            events::emit_asset_claim_rewards(&e, user, asset, amt);
            Ok(amt)
        })
    }

    pub fn balance_of_asset(e: Env, user: Address, asset: Address) -> Result<i128, VaultError> {
        storage::get_user_asset_balance(&e, &user, &asset)
    }

    pub fn total_deposits_of_asset(e: Env, asset: Address) -> Result<i128, VaultError> {
        storage::get_asset_total_deposits(&e, &asset)
    }

    pub fn reward_index_of_asset(e: Env, asset: Address) -> Result<i128, VaultError> {
        storage::get_asset_reward_index(&e, &asset)
    }

    pub fn pending_rewards_for_asset(
        e: Env,
        user: Address,
        asset: Address,
    ) -> Result<i128, VaultError> {
        storage::pending_user_asset_rewards_view(&e, &user, &asset)
    }

    pub fn vested_rewards_for_asset(
        e: Env,
        user: Address,
        asset: Address,
    ) -> Result<i128, VaultError> {
        storage::vested_user_asset_rewards_view(&e, &user, &asset)
    }

    pub fn is_asset_supported(e: Env, asset: Address) -> bool {
        storage::is_asset_supported(&e, &asset)
    }

    // -----------------------------------------------------------------------
    // Delegation Management
    // -----------------------------------------------------------------------

    /// Grant `permissions` to `operator` for the caller's vault positions.
    /// The delegation optionally expires at `expires_at` (0 = never).
    pub fn delegate(
        e: Env,
        delegator: Address,
        operator: Address,
        permissions: u32,
        expires_at: u64,
    ) -> Result<(), VaultError> {
        storage::require_initialized(&e)?;
        delegator.require_auth();

        if operator == delegator {
            return Err(DelegationError::CannotDelegateToSelf.into());
        }
        if expires_at != 0 && expires_at <= e.ledger().timestamp() {
            return Err(DelegationError::InvalidExpiration.into());
        }

        // Enforce max delegations limit.
        let max = storage::get_max_delegations(&e);
        let current_count = storage::delegation_count(&e, &delegator);
        // Allow updating an existing delegation without counting toward the limit.
        let exists = storage::get_delegation(&e, &delegator, &operator).is_some();
        if !exists && current_count >= max {
            return Err(DelegationError::MaxDelegationsExceeded.into());
        }

        storage::set_delegation(&e, &delegator, &operator, permissions, expires_at);
        events::emit_delegate(&e, delegator, operator, permissions, expires_at);
        Ok(())
    }

    /// Revoke a previously granted delegation.
    pub fn revoke_delegation(
        e: Env,
        delegator: Address,
        operator: Address,
    ) -> Result<(), VaultError> {
        storage::require_initialized(&e)?;
        delegator.require_auth();

        storage::remove_delegation(&e, &delegator, &operator);
        events::emit_revoke_delegation(&e, delegator, operator);
        Ok(())
    }

    /// Query a specific delegation entry.
    pub fn get_delegation(
        e: Env,
        delegator: Address,
        operator: Address,
    ) -> Option<storage::Delegation> {
        storage::get_delegation(&e, &delegator, &operator)
    }

    /// List all operators a delegator has granted permissions to, along with their delegation info.
    pub fn get_delegations(e: Env, delegator: Address) -> soroban_sdk::Vec<storage::Delegation> {
        let operators = storage::get_delegation_operators(&e, &delegator);
        let mut result: soroban_sdk::Vec<storage::Delegation> = soroban_sdk::Vec::new(&e);
        for op in operators.iter() {
            if let Some(d) = storage::get_delegation(&e, &delegator, &op) {
                result.push_back(d);
            }
        }
        result
    }

    // -----------------------------------------------------------------------
    // Delegated Actions
    // -----------------------------------------------------------------------

    /// Deposit on behalf of a delegator. The caller must have Deposit permission.
    pub fn delegated_deposit(
        e: Env,
        delegator: Address,
        operator: Address,
        amount: i128,
    ) -> Result<(), VaultError> {
        storage::require_not_paused(&e)?;
        storage::require_initialized(&e)?;
        validate_positive_amount(amount)?;
        storage::authorize_for_user(&e, &delegator, &operator, storage::PERMISSION_DEPOSIT)?;

        with_non_reentrant(&e, || {
            let state = storage::get_state(&e)?;
            CrossContractClient::token_transfer(
                &e,
                &state.deposit_token,
                &operator,
                &e.current_contract_address(),
                amount,
            )?;

            let (_state, _position) = storage::store_deposit(&e, &delegator, amount)?;
            events::emit_delegated_action(
                &e,
                delegator.clone(),
                operator.clone(),
                storage::PERMISSION_DEPOSIT,
                axionvera_events::ACT_DEPOSIT,
            );
            events::emit_deposit(&e, delegator.clone(), amount);
            Ok(())
        })
    }

    /// Withdraw from a delegator's balance. The caller must have Withdraw permission.
    pub fn delegated_withdraw(
        e: Env,
        delegator: Address,
        operator: Address,
        amount: i128,
    ) -> Result<(), VaultError> {
        storage::require_not_paused(&e)?;
        storage::require_initialized(&e)?;
        validate_positive_amount(amount)?;
        storage::authorize_for_user(&e, &delegator, &operator, storage::PERMISSION_WITHDRAW)?;

        with_non_reentrant(&e, || {
            let _state = storage::get_state(&e)?;
            let (state, position) = storage::store_withdraw(&e, &delegator, amount)?;

            events::emit_delegated_action(
                &e,
                delegator.clone(),
                operator.clone(),
                storage::PERMISSION_WITHDRAW,
                axionvera_events::ACT_WITHDRAW,
            );
            events::emit_withdraw(&e, delegator.clone(), amount, position.balance);

            CrossContractClient::token_transfer(
                &e,
                &state.deposit_token,
                &e.current_contract_address(),
                &operator,
                amount,
            )?;

            Ok(())
        })
    }

    /// Lock tokens in a delegator's vault. The caller must have Lock permission.
    pub fn delegated_lock(
        e: Env,
        delegator: Address,
        operator: Address,
        amount: i128,
        duration_seconds: u64,
    ) -> Result<(), VaultError> {
        storage::require_not_paused(&e)?;
        storage::require_initialized(&e)?;
        validate_positive_amount(amount)?;
        if duration_seconds == 0 {
            return Err(ValidationError::InvalidLockDuration.into());
        }
        storage::authorize_for_user(&e, &delegator, &operator, storage::PERMISSION_LOCK)?;

        with_non_reentrant(&e, || {
            let unlock_timestamp = e
                .ledger()
                .timestamp()
                .checked_add(duration_seconds)
                .ok_or(VaultError::MathOverflow)?;
            storage::store_lock(&e, &delegator, amount, duration_seconds)?;
            events::emit_delegated_action(
                &e,
                delegator.clone(),
                operator.clone(),
                storage::PERMISSION_LOCK,
                axionvera_events::ACT_LOCK,
            );
            events::emit_lock(&e, delegator, amount, unlock_timestamp);
            Ok(())
        })
    }

    /// Unlock expired locks for a delegator. The caller must have Unlock permission.
    pub fn delegated_unlock_expired(
        e: Env,
        delegator: Address,
        operator: Address,
        limit: u32,
    ) -> Result<i128, VaultError> {
        storage::require_not_paused(&e)?;
        storage::require_initialized(&e)?;
        storage::authorize_for_user(&e, &delegator, &operator, storage::PERMISSION_UNLOCK)?;

        const MAX_UNLOCK_LIMIT: u32 = 50;
        if limit > MAX_UNLOCK_LIMIT {
            return Err(VaultError::OperationLimitExceeded);
        }

        with_non_reentrant(&e, || {
            let unlocked_amount = storage::unlock_expired_locks(&e, &delegator, limit)?;
            if unlocked_amount > 0 {
                events::emit_delegated_action(
                    &e,
                    delegator.clone(),
                    operator.clone(),
                    storage::PERMISSION_UNLOCK,
                    axionvera_events::ACT_UNLOCK,
                );
                events::emit_unlock(&e, delegator, unlocked_amount);
            }
            Ok(unlocked_amount)
        })
    }

    /// Claim rewards for a delegator. The caller must have Claim permission.
    pub fn delegated_claim_rewards(
        e: Env,
        delegator: Address,
        operator: Address,
    ) -> Result<i128, VaultError> {
        storage::require_not_paused(&e)?;
        storage::require_initialized(&e)?;
        storage::authorize_for_user(&e, &delegator, &operator, storage::PERMISSION_CLAIM)?;

        with_non_reentrant(&e, || {
            let amt = storage::store_claimable_rewards(&e, &delegator)?;
            if amt <= 0 {
                return Ok(0);
            }

            let total_deposits = storage::get_total_deposits(&e)?;
            update_protocol_metrics(&e, total_deposits, 0)?;

            let reward_token_id = storage::get_reward_token(&e)?;
            let contract_balance = CrossContractClient::token_balance(
                &e,
                &reward_token_id,
                &e.current_contract_address(),
            )?;
            ensure_contract_balance(contract_balance, amt)?;
            CrossContractClient::token_transfer(
                &e,
                &reward_token_id,
                &e.current_contract_address(),
                &operator,
                amt,
            )?;

            events::emit_delegated_action(
                &e,
                delegator.clone(),
                operator.clone(),
                storage::PERMISSION_CLAIM,
                axionvera_events::ACT_CLAIM,
            );
            events::emit_claim_rewards(&e, delegator, amt);
            Ok(amt)
        })
    }

    /// Set the maximum number of delegations allowed per user (admin only).
    pub fn set_max_delegations(e: Env, max: u32) -> Result<(), VaultError> {
        storage::require_initialized(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();
        storage::set_max_delegations(&e, max);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Analytics & Metrics
    // -----------------------------------------------------------------------

    pub fn get_protocol_metrics(e: Env) -> metrics::ProtocolMetrics {
        metrics::get_metrics(&e)
    }

    pub fn get_historical_metrics(e: Env) -> soroban_sdk::Vec<metrics::MetricSnapshot> {
        metrics::get_historical_metrics(&e)
    }

    pub fn take_metrics_snapshot(e: Env) -> Result<(), VaultError> {
        storage::require_initialized(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();
        metrics::take_snapshot(&e);
        Ok(())
    }

    pub fn protocol_utilization(e: Env) -> Result<u32, VaultError> {
        let state = storage::get_state(&e)?;
        Ok(metrics::calculate_utilization(state.total_deposits, state.target_deposits))
    }
}

fn validate_positive_amount(amount: i128) -> Result<(), VaultError> {
    if amount < 0 {
        return Err(ValidationError::NegativeAmount.into());
    }
    if amount == 0 {
        return Err(ValidationError::InvalidAmount.into());
    }
    Ok(())
}

fn validate_distinct_token_addresses(
    deposit_token: &Address,
    reward_token: &Address,
) -> Result<(), VaultError> {
    if deposit_token == reward_token {
        return Err(ValidationError::InvalidTokenConfiguration.into());
    }
    Ok(())
}

fn validate_utilization_multipliers(
    multipliers: &soroban_sdk::Vec<storage::MultiplierPoint>,
) -> Result<(), VaultError> {
    if multipliers.is_empty() {
        return Ok(()); // An empty list is valid, which causes rewards to default to 1.0x.
    }

    let mut last_util_bps = 0;
    for point in multipliers.iter() {
        if point.utilization_bps > 10_000
            || point.multiplier_bps == 0
            || point.multiplier_bps > 100_000
        {
            return Err(ValidationError::InvalidUtilizationParameters.into());
        }
        if point.utilization_bps < last_util_bps {
            // The list must be sorted by utilization_bps in ascending order.
            return Err(ValidationError::InvalidUtilizationParameters.into());
        }
        last_util_bps = point.utilization_bps;
    }

    Ok(())
}

fn ensure_contract_balance(balance: i128, requested_amount: i128) -> Result<(), VaultError> {
    if balance < requested_amount {
        return Err(BalanceError::InsufficientContractBalance.into());
    }
    Ok(())
}

fn with_non_reentrant<T, F>(e: &Env, f: F) -> Result<T, VaultError>
where
    F: FnOnce() -> Result<T, VaultError>,
{
    storage::enter_non_reentrant(e)?;
    let result = f();
    storage::exit_non_reentrant(e);
    result
}

fn collect_protocol_fee(
    e: &Env,
    fee_type: FeeType,
    actor: Address,
    asset: Option<Address>,
    gross_amount: i128,
    resources: accounting::OperationResources,
) -> Result<Option<i128>, VaultError> {
    let Some(config) = protocol_storage::get_fee_config(e) else {
        return Ok(None);
    };

    let fee_bps = config.rate_for(fee_type);
    if fee_bps == 0 {
        return Ok(None);
    }

    let receipt = fee_framework::build_fee_receipt(
        fee_type,
        actor,
        config.treasury.clone(),
        asset.clone(),
        gross_amount,
        fee_bps,
        e.ledger().timestamp(),
    )
    .map_err(fee_error_to_vault_error)?;

    if receipt.fee_amount <= 0 {
        return Ok(None);
    }

    let asset_address = asset.ok_or(VaultError::InvalidAddress)?;
    CrossContractClient::token_transfer(
        e,
        &asset_address,
        &e.current_contract_address(),
        &config.treasury,
        receipt.treasury_amount,
    )?;
    fee_framework::record_fee_collection(e, &receipt, resources).map_err(fee_error_to_vault_error)?;
    Ok(Some(receipt.net_amount))
}

fn account_operation(
    e: &Env,
    category: accounting::AccountingCategory,
    operation: accounting::AccountingOperation,
    actor: Option<Address>,
    asset: Option<Address>,
    amount_in: i128,
    amount_out: i128,
    amount_processed: i128,
    resources: accounting::OperationResources,
) -> Result<(), VaultError> {
    accounting::record_operation(
        e,
        accounting::AccountingEntry {
            category,
            operation,
            actor,
            asset,
            amount_in,
            amount_out,
            amount_processed,
            resources,
        },
    )
    .map_err(accounting_error_to_vault_error)
}

fn fee_error_to_vault_error(error: axionvera_interfaces::FeeError) -> VaultError {
    match error {
        axionvera_interfaces::FeeError::InvalidAmount => VaultError::InvalidAmount,
        axionvera_interfaces::FeeError::InvalidFeeRate => VaultError::InvalidFeeRate,
        axionvera_interfaces::FeeError::MathOverflow => VaultError::MathOverflow,
    }
}

fn accounting_error_to_vault_error(error: accounting::AccountingError) -> VaultError {
    match error {
        accounting::AccountingError::NegativeAmount => VaultError::NegativeAmount,
        accounting::AccountingError::Overflow => VaultError::MathOverflow,
        accounting::AccountingError::InconsistentTotals => VaultError::InvalidState,
    }
}

#[cfg(test)]
mod precision_tests {
    use super::errors::VaultError;
    use super::storage::{
        checked_accrued_rewards, checked_reward_index_increment, PRECISION_FACTOR,
    };

    #[test]
    fn increment_basic() {
        let inc = checked_reward_index_increment(400, 400).unwrap();
        assert_eq!(inc, PRECISION_FACTOR);
    }

    #[test]
    fn increment_small_reward_large_deposits_retains_precision() {
        let inc = checked_reward_index_increment(1, 1_000_000).unwrap();
        assert!(inc > 0, "precision lost: increment rounded to zero");
        assert_eq!(inc, PRECISION_FACTOR / 1_000_000);
    }

    #[test]
    fn increment_rejects_zero_deposits() {
        assert_eq!(
            checked_reward_index_increment(100, 0),
            Err(VaultError::NoDeposits)
        );
    }

    #[test]
    fn increment_rejects_negative_deposits() {
        assert_eq!(
            checked_reward_index_increment(100, -1),
            Err(VaultError::NoDeposits)
        );
    }

    #[test]
    fn accrued_proportional_equal_deposits() {
        let delta = checked_reward_index_increment(400, 200).unwrap();
        let reward = checked_accrued_rewards(100, delta).unwrap();
        assert_eq!(reward, 200);
    }

    #[test]
    fn accrued_vastly_different_deposits_user_a_tiny() {
        let total = 1_000_001_i128;
        let rewards = 1_000_001_i128;
        let delta = checked_reward_index_increment(rewards, total).unwrap();

        let reward_a = checked_accrued_rewards(1, delta).unwrap();
        assert_eq!(reward_a, 1);

        let reward_b = checked_accrued_rewards(1_000_000, delta).unwrap();
        assert_eq!(reward_b, 1_000_000);
    }

    #[test]
    fn accrued_zero_balance_returns_zero() {
        let delta = checked_reward_index_increment(1000, 500).unwrap();
        assert_eq!(checked_accrued_rewards(0, delta).unwrap(), 0);
    }

    #[test]
    fn accrued_zero_delta_returns_zero() {
        assert_eq!(checked_accrued_rewards(1_000_000, 0).unwrap(), 0);
    }

    #[test]
    fn precision_factor_value() {
        assert_eq!(PRECISION_FACTOR, 1_000_000_000);
    }

    #[test]
    fn round_trip_proportionality() {
        let total = 1_000_000_i128;
        let rewards = 1_000_000_i128;
        let delta = checked_reward_index_increment(rewards, total).unwrap();

        assert_eq!(checked_accrued_rewards(1, delta).unwrap(), 1);
        assert_eq!(checked_accrued_rewards(999_999, delta).unwrap(), 999_999);
    }
}
