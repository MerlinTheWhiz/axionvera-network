#![no_std]

use soroban_sdk::{Address, Env};
use axionvera_auth::{AccessPolicy, PolicyViolation};

pub struct Authenticated<Context> {
    get_address: fn(&Context) -> Address,
}

impl<Context> Authenticated<Context> {
    pub fn new(get_address: fn(&Context) -> Address) -> Self {
        Self { get_address }
    }
}

impl<Context> AccessPolicy<Context> for Authenticated<Context> {
    fn enforce(&self, context: &Context) -> Result<(), PolicyViolation> {
        let address = (self.get_address)(context);
        address.require_auth();
        Ok(())
    }
}

pub struct MatchAddress<Context> {
    get_caller: fn(&Context) -> Address,
    get_expected: fn(&Context) -> Address,
    violation: PolicyViolation,
}

impl<Context> MatchAddress<Context> {
    pub fn new(
        get_caller: fn(&Context) -> Address,
        get_expected: fn(&Context) -> Address,
        violation: PolicyViolation,
    ) -> Self {
        Self {
            get_caller,
            get_expected,
            violation,
        }
    }
}

impl<Context> AccessPolicy<Context> for MatchAddress<Context> {
    fn enforce(&self, context: &Context) -> Result<(), PolicyViolation> {
        let caller = (self.get_caller)(context);
        let expected = (self.get_expected)(context);
        if caller == expected {
            Ok(())
        } else {
            Err(self.violation)
        }
    }
}

pub struct PredicatePolicy<Context> {
    predicate: fn(&Context) -> bool,
    violation: PolicyViolation,
}

impl<Context> PredicatePolicy<Context> {
    pub fn new(predicate: fn(&Context) -> bool, violation: PolicyViolation) -> Self {
        Self {
            predicate,
            violation,
        }
    }
}

impl<Context> AccessPolicy<Context> for PredicatePolicy<Context> {
    fn enforce(&self, context: &Context) -> Result<(), PolicyViolation> {
        if (self.predicate)(context) {
            Ok(())
        } else {
            Err(self.violation)
        }
    }
}

pub fn require_actor(address: &Address) {
    address.require_auth();
}

pub fn require_stored_admin(admin: &Address) {
    admin.require_auth();
}

pub fn require_pending_admin(new_admin: &Address, pending_admin: Option<Address>) -> Result<(), ()> {
    new_admin.require_auth();
    if let Some(pending) = pending_admin {
        if new_admin == &pending {
            return Ok(());
        }
    }
    Err(())
}

pub fn require_admin(caller: &Address, admin: &Address) -> Result<(), ()> {
    caller.require_auth();
    if caller == admin {
        return Ok(());
    }
    Err(())
}
