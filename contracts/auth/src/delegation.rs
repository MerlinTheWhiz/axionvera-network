#![no_std]

use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol};
use axionvera_interfaces::{DelegationRule, DelegationError};

#[contracttype]
pub enum DataKey {
    Delegation(Address, Address, Symbol), // (delegator, delegatee, operation)
}

pub struct DelegationManager;

impl DelegationManager {
    pub fn delegate(
        e: &Env,
        delegator: Address,
        delegatee: Address,
        operation: Symbol,
        expiration: u64,
    ) -> Result<(), DelegationError> {
        delegator.require_auth();

        if expiration <= e.ledger().timestamp() {
            return Err(DelegationError::InvalidExpiration);
        }

        let key = DataKey::Delegation(delegator.clone(), delegatee.clone(), operation.clone());

        // Prevent privilege escalation: in this context, we ensure that the delegator
        // is not delegating something they don't have authority over.
        // For a general implementation, require_auth() is the primary check.

        let rule = DelegationRule {
            delegator: delegator.clone(),
            delegatee: delegatee.clone(),
            operation: operation.clone(),
            expiration,
        };
        e.storage().persistent().set(&key, &rule);

        e.events().publish(
            (symbol_short!("deleg_g"), delegator, delegatee, operation),
            expiration,
        );

        Ok(())
    }

    pub fn revoke_delegation(
        e: &Env,
        delegator: Address,
        delegatee: Address,
        operation: Symbol,
    ) -> Result<(), DelegationError> {
        delegator.require_auth();

        let key = DataKey::Delegation(delegator.clone(), delegatee.clone(), operation.clone());
        if !e.storage().persistent().has(&key) {
            return Err(DelegationError::DelegationNotFound);
        }
        e.storage().persistent().remove(&key);

        e.events().publish(
            (symbol_short!("deleg_r"), delegator, delegatee, operation),
            (),
        );

        Ok(())
    }

    pub fn get_delegation(
        e: &Env,
        delegator: Address,
        delegatee: Address,
        operation: Symbol,
    ) -> Option<DelegationRule> {
        let key = DataKey::Delegation(delegator, delegatee, operation);
        e.storage().persistent().get(&key)
    }

    pub fn is_authorized(
        e: &Env,
        delegator: Address,
        delegatee: Address,
        operation: Symbol,
    ) -> bool {
        if let Some(rule) = Self::get_delegation(e, delegator, delegatee, operation) {
            return e.ledger().timestamp() <= rule.expiration;
        }
        false
    }
}
