#![no_std]

use soroban_sdk::{Address, Env, InvokeError, Symbol, Vec};

/// Maximum arguments permitted for a single external contract call.
pub const DEFAULT_MAX_ARGUMENTS: u32 = 8;

pub use axionvera_interfaces::{
    ExternalCall, ExternalCallPolicy, ExternalCallReceipt, ExternalContractGateway,
    IntegrationError,
};

/// Stateless gateway implementation suitable for reuse from protocol contracts.
pub struct SorobanExternalGateway;

impl ExternalContractGateway for SorobanExternalGateway {
    fn validate_call(
        e: &Env,
        call: &ExternalCall,
        policy: &ExternalCallPolicy,
    ) -> Result<(), IntegrationError> {
        validate_policy(policy)?;
        if !policy.allow_self_call && call.target == e.current_contract_address() {
            return Err(IntegrationError::SelfCallBlocked);
        }
        if call.args.len() > policy.max_arguments {
            return Err(IntegrationError::TooManyArguments);
        }
        if !contains_address(&policy.allowed_targets, &call.target) {
            return Err(IntegrationError::TargetNotAllowed);
        }
        if !contains_symbol(&policy.allowed_functions, &call.function) {
            return Err(IntegrationError::FunctionNotAllowed);
        }
        Ok(())
    }

    fn invoke_void(
        e: &Env,
        call: &ExternalCall,
        policy: &ExternalCallPolicy,
    ) -> Result<ExternalCallReceipt, IntegrationError> {
        Self::validate_call(e, call, policy)?;
        e.try_invoke_contract::<(), InvokeError>(&call.target, &call.function, call.args.clone())
            .map_err(|_| IntegrationError::ExternalInvocationFailed)?
            .map_err(|_| IntegrationError::ExternalInvocationFailed)?;

        Ok(ExternalCallReceipt {
            target: call.target.clone(),
            function: call.function.clone(),
            success: true,
            timestamp: e.ledger().timestamp(),
        })
    }
}

/// Build a restrictive policy for one target with a bounded set of functions.
pub fn single_target_policy(
    e: &Env,
    target: Address,
    functions: Vec<Symbol>,
) -> ExternalCallPolicy {
    let mut targets = Vec::new(e);
    targets.push_back(target);
    ExternalCallPolicy {
        allowed_targets: targets,
        allowed_functions: functions,
        max_arguments: DEFAULT_MAX_ARGUMENTS,
        allow_self_call: false,
    }
}

fn validate_policy(policy: &ExternalCallPolicy) -> Result<(), IntegrationError> {
    if policy.allowed_targets.is_empty() {
        return Err(IntegrationError::EmptyTargetAllowList);
    }
    if policy.allowed_functions.is_empty() {
        return Err(IntegrationError::EmptyFunctionAllowList);
    }
    Ok(())
}

fn contains_address(values: &Vec<Address>, needle: &Address) -> bool {
    for value in values.iter() {
        if &value == needle {
            return true;
        }
    }
    false
}

fn contains_symbol(values: &Vec<Symbol>, needle: &Symbol) -> bool {
    for value in values.iter() {
        if &value == needle {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod test;
