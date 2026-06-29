#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, contracterror};
use axionvera_events::{PROTOCOL, ledger_timestamp};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RiskError {
    TooSmall = 1,
    TooLarge = 2,
    CapReached = 3,
    Unauthorized = 4,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RiskParameters {
    pub max_deposit_amount: i128,
    pub min_deposit_amount: i128,
    pub max_withdrawal_amount: i128,
    pub global_cap: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RiskEventPayload {
    pub action: Symbol,
    pub amount: i128,
    pub limit: i128,
    pub timestamp: u64,
}

#[contracttype]
pub enum DataKey {
    Admin,
    RiskParams,
    CurrentTotalDeposits,
}

#[contract]
pub struct RiskManagement;

#[contractimpl]
impl RiskManagement {
    pub fn initialize_risk(e: Env, admin: Address, params: RiskParameters) {
        if e.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        e.storage().instance().set(&DataKey::Admin, &admin);
        e.storage().instance().set(&DataKey::RiskParams, &params);
        e.storage().instance().set(&DataKey::CurrentTotalDeposits, &0_i128);
    }

    pub fn set_risk_params(e: Env, admin: Address, params: RiskParameters) {
        admin.require_auth();
        let stored_admin: Address = e.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("Unauthorized");
        }
        e.storage().instance().set(&DataKey::RiskParams, &params);
    }

    pub fn get_risk_params(e: Env) -> RiskParameters {
        e.storage().instance().get(&DataKey::RiskParams).unwrap()
    }

    pub fn check_deposit(e: Env, amount: i128) -> Result<(), RiskError> {
        let params: RiskParameters = e.storage().instance().get(&DataKey::RiskParams).unwrap();
        let current_total: i128 = e.storage().instance().get(&DataKey::CurrentTotalDeposits).unwrap_or(0);

        if amount < params.min_deposit_amount {
            emit_risk_violation(&e, symbol_short!("min_dep"), amount, params.min_deposit_amount);
            return Err(RiskError::TooSmall);
        }
        if amount > params.max_deposit_amount && params.max_deposit_amount > 0 {
            emit_risk_violation(&e, symbol_short!("max_dep"), amount, params.max_deposit_amount);
            return Err(RiskError::TooLarge);
        }
        if current_total + amount > params.global_cap && params.global_cap > 0 {
            emit_risk_violation(&e, symbol_short!("glob_cap"), amount, params.global_cap);
            return Err(RiskError::CapReached);
        }
        Ok(())
    }

    pub fn check_withdrawal(e: Env, amount: i128) -> Result<(), RiskError> {
        let params: RiskParameters = e.storage().instance().get(&DataKey::RiskParams).unwrap();
        if amount > params.max_withdrawal_amount && params.max_withdrawal_amount > 0 {
            emit_risk_violation(&e, symbol_short!("max_wd"), amount, params.max_withdrawal_amount);
            return Err(RiskError::TooLarge);
        }
        Ok(())
    }

    pub fn update_total_deposits(e: Env, delta: i128) {
        let current_total: i128 = e.storage().instance().get(&DataKey::CurrentTotalDeposits).unwrap_or(0);
        e.storage().instance().set(&DataKey::CurrentTotalDeposits, &(current_total + delta));
    }
}

fn emit_risk_violation(e: &Env, action: Symbol, amount: i128, limit: i128) {
    e.events().publish(
        (PROTOCOL, symbol_short!("risk_err")),
        RiskEventPayload {
            action,
            amount,
            limit,
            timestamp: ledger_timestamp(e),
        }
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Events};

    #[test]
    fn test_initialize() {
        let e = Env::default();
        let admin = Address::generate(&e);
        let params = RiskParameters {
            max_deposit_amount: 1000,
            min_deposit_amount: 10,
            max_withdrawal_amount: 500,
            global_cap: 5000,
        };

        let contract_id = e.register(RiskManagement, ());
        let client = RiskManagementClient::new(&e, &contract_id);
        client.initialize_risk(&admin, &params);

        assert_eq!(client.get_risk_params(), params);
    }

    #[test]
    fn test_check_deposit() {
        let e = Env::default();
        let admin = Address::generate(&e);
        let params = RiskParameters {
            max_deposit_amount: 1000,
            min_deposit_amount: 10,
            max_withdrawal_amount: 500,
            global_cap: 5000,
        };

        let contract_id = e.register(RiskManagement, ());
        let client = RiskManagementClient::new(&e, &contract_id);
        client.initialize_risk(&admin, &params);

        // Valid deposit
        assert!(client.try_check_deposit(&50).is_ok());

        // Too small
        assert_eq!(client.try_check_deposit(&5), Err(Ok(RiskError::TooSmall)));

        // Too large
        assert_eq!(client.try_check_deposit(&1500), Err(Ok(RiskError::TooLarge)));

        // Global cap
        client.update_total_deposits(&4900);
        assert_eq!(client.try_check_deposit(&150), Err(Ok(RiskError::CapReached)));
    }

    #[test]
    fn test_check_withdrawal() {
        let e = Env::default();
        let admin = Address::generate(&e);
        let params = RiskParameters {
            max_deposit_amount: 1000,
            min_deposit_amount: 10,
            max_withdrawal_amount: 500,
            global_cap: 5000,
        };

        let contract_id = e.register(RiskManagement, ());
        let client = RiskManagementClient::new(&e, &contract_id);
        client.initialize_risk(&admin, &params);

        // Valid withdrawal
        assert!(client.try_check_withdrawal(&100).is_ok());

        // Too large
        assert_eq!(client.try_check_withdrawal(&600), Err(Ok(RiskError::TooLarge)));
    }
}
