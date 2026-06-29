#![no_std]

pub mod errors;
mod events;
mod storage;
#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env, Symbol, Vec};
use axionvera_interfaces::{
    Policy, PolicyEvaluationRequest, PolicyEvaluationResult, PolicyEngine, PolicyError,
    PolicyStatus,
};
use crate::errors::PolicyError as Error;
use crate::storage;

const MAX_POLICY_NAME_LEN: u32 = 64;

#[contract]
pub struct PolicyContract;

#[contractimpl]
impl PolicyContract {
    /// Returns the contract version.
    pub fn version() -> u32 {
        1
    }
}

#[contractimpl]
impl PolicyEngine for PolicyContract {
    fn initialize(e: Env, admin: Address) -> Result<(), Error> {
        if storage::is_initialized(&e) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        storage::initialize(&e, &admin);
        events::emit_initialized(&e, admin);
        Ok(())
    }

    fn add_policy(e: Env, policy: Policy) -> Result<(), Error> {
        storage::require_initialized(&e)?;
        storage::require_not_paused(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();
        
        validate_policy(&e, &policy)?;
        
        if storage::has_policy(&e, &policy.id) {
            return Err(Error::PolicyAlreadyExists);
        }
        
        storage::set_policy(&e, &policy);
        events::emit_policy_added(&e, policy.id.clone(), policy.name.clone(), admin);
        Ok(())
    }

    fn update_policy(e: Env, policy: Policy) -> Result<(), Error> {
        storage::require_initialized(&e)?;
        storage::require_not_paused(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();
        
        validate_policy(&e, &policy)?;
        
        if !storage::has_policy(&e, &policy.id) {
            return Err(Error::PolicyNotFound);
        }
        
        storage::set_policy(&e, &policy);
        events::emit_policy_updated(&e, policy.id.clone(), policy.name.clone(), admin);
        Ok(())
    }

    fn delete_policy(e: Env, policy_id: BytesN<32>) -> Result<(), Error> {
        storage::require_initialized(&e)?;
        storage::require_not_paused(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();
        
        if !storage::has_policy(&e, &policy_id) {
            return Err(Error::PolicyNotFound);
        }
        
        storage::delete_policy(&e, &policy_id);
        events::emit_policy_deleted(&e, policy_id, admin);
        Ok(())
    }

    fn get_policy(e: Env, policy_id: BytesN<32>) -> Result<Policy, Error> {
        storage::require_initialized(&e)?;
        storage::get_policy(&e, &policy_id)
    }

    fn list_policies(e: Env) -> Result<Vec<Policy>, Error> {
        storage::require_initialized(&e)?;
        let policy_ids = storage::get_policy_list(&e);
        let mut policies = Vec::new(&e);
        for id in policy_ids.iter() {
            if let Ok(policy) = storage::get_policy(&e, &id) {
                policies.push_back(policy);
            }
        }
        Ok(policies)
    }

    fn evaluate(e: Env, request: PolicyEvaluationRequest) -> Result<PolicyEvaluationResult, Error> {
        storage::require_initialized(&e)?;
        
        let policy_ids = storage::get_policy_list(&e);
        let mut active_policies = Vec::new(&e);
        
        // Collect all active policies that match the request
        for id in policy_ids.iter() {
            if let Ok(policy) = storage::get_policy(&e, &id) {
                if policy.status == PolicyStatus::Active && policy_matches_request(&policy, &request) {
                    active_policies.push_back(policy);
                }
            }
        }
        
        // Sort policies by priority (highest first)
        let mut sorted_policies: Vec<Policy> = Vec::new(&e);
        let mut remaining = active_policies;
        while !remaining.is_empty() {
            let mut highest_idx = 0;
            let mut highest_priority = remaining.get(0).unwrap().priority;
            for (i, policy) in remaining.iter().enumerate() {
                if policy.priority > highest_priority {
                    highest_priority = policy.priority;
                    highest_idx = i;
                }
            }
            let policy = remaining.remove(highest_idx as u32).unwrap();
            sorted_policies.push_back(policy);
        }
        
        // Evaluate policies - for now, all active matching policies pass (this is a placeholder)
        // In a real implementation, you would have actual policy logic here
        for policy in sorted_policies.iter() {
            // Example: if policy type is AllowDeny and target_contract is set, let's say it passes
            // This is just a demo - real policies would have actual logic
        }
        
        let result = PolicyEvaluationResult {
            passed: true,
            failed_policy_id: None,
            message: Bytes::from_str(&e, "All policies passed"),
        };
        
        events::emit_policy_evaluated(
            &e,
            request.caller.clone(),
            request.target_contract.clone(),
            request.target_function.clone(),
            result.passed,
            result.failed_policy_id.clone(),
        );
        
        Ok(result)
    }

    fn admin(e: Env) -> Result<Address, Error> {
        storage::require_initialized(&e)?;
        storage::get_admin(&e)
    }

    fn propose_new_admin(e: Env, new_admin: Address) -> Result<(), Error> {
        storage::require_initialized(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();
        storage::set_pending_admin(&e, &new_admin);
        events::emit_admin_transfer_proposed(&e, admin, new_admin);
        Ok(())
    }

    fn accept_admin(e: Env, new_admin: Address) -> Result<(), Error> {
        storage::require_initialized(&e)?;
        new_admin.require_auth();
        let previous_admin = storage::get_admin(&e)?;
        let pending = storage::get_pending_admin(&e).ok_or(Error::NoPendingAdmin)?;
        if pending != new_admin {
            return Err(Error::Unauthorized);
        }
        storage::set_admin(&e, &new_admin);
        storage::clear_pending_admin(&e);
        events::emit_admin_transfer_accepted(&e, previous_admin, new_admin);
        Ok(())
    }

    fn pause_contract(e: Env) -> Result<(), Error> {
        storage::require_initialized(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();
        storage::set_paused(&e, &true);
        events::emit_paused(&e, admin);
        Ok(())
    }

    fn unpause_contract(e: Env) -> Result<(), Error> {
        storage::require_initialized(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();
        storage::set_paused(&e, &false);
        events::emit_unpaused(&e, admin);
        Ok(())
    }

    fn is_paused(e: Env) -> bool {
        storage::get_is_paused(&e)
    }
}

fn validate_policy(e: &Env, policy: &Policy) -> Result<(), Error> {
    if policy.name.len() == 0 || policy.name.len() > MAX_POLICY_NAME_LEN {
        return Err(Error::InvalidPolicyName);
    }
    Ok(())
}

fn policy_matches_request(policy: &Policy, request: &PolicyEvaluationRequest) -> bool {
    // Check if policy applies to target contract
    if let Some(target_contract) = &policy.target_contract {
        if target_contract != &request.target_contract {
            return false;
        }
    }
    
    // Check if policy applies to target function
    if let Some(target_function) = &policy.target_function {
        if target_function != &request.target_function {
            return false;
        }
    }
    
    true
}
