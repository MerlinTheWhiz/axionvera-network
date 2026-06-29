use super::*;

use soroban_sdk::{
    contract, contracterror, contractimpl, symbol_short, testutils::Ledger, vec, Env, IntoVal,
    Symbol, Val, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum TargetError {
    Failed = 1,
}

#[contract]
pub struct TargetContract;

#[contractimpl]
impl TargetContract {
    pub fn record(e: Env, key: Symbol, value: u32) {
        let mut entries: Vec<u32> = e
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(&e));
        entries.push_back(value);
        e.storage().persistent().set(&key, &entries);
    }

    pub fn fail(_e: Env) -> Result<(), TargetError> {
        Err(TargetError::Failed)
    }
}

fn call_args(e: &Env, key: Symbol, value: u32) -> Vec<Val> {
    (key, value).into_val(e)
}

#[test]
fn validates_allowlisted_external_call() {
    let e = Env::default();
    let target = e.register(TargetContract, ());
    let policy = single_target_policy(&e, target.clone(), vec![&e, symbol_short!("record")]);
    let call = ExternalCall {
        target,
        function: symbol_short!("record"),
        args: call_args(&e, symbol_short!("main"), 10),
    };

    assert_eq!(
        SorobanExternalGateway::validate_call(&e, &call, &policy),
        Ok(())
    );
}

#[test]
fn rejects_unallowlisted_function() {
    let e = Env::default();
    let target = e.register(TargetContract, ());
    let policy = single_target_policy(&e, target.clone(), vec![&e, symbol_short!("record")]);
    let call = ExternalCall {
        target,
        function: symbol_short!("fail"),
        args: Vec::new(&e),
    };

    assert_eq!(
        SorobanExternalGateway::validate_call(&e, &call, &policy),
        Err(IntegrationError::FunctionNotAllowed)
    );
}

#[test]
fn safely_maps_external_failures() {
    let e = Env::default();
    e.ledger().set_timestamp(99);
    let target = e.register(TargetContract, ());
    let policy = single_target_policy(&e, target.clone(), vec![&e, symbol_short!("fail")]);
    let call = ExternalCall {
        target,
        function: symbol_short!("fail"),
        args: Vec::new(&e),
    };

    assert_eq!(
        SorobanExternalGateway::invoke_void(&e, &call, &policy),
        Err(IntegrationError::ExternalInvocationFailed)
    );
}
