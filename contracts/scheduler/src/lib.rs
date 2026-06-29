#![no_std]

pub mod errors;
mod events;
mod storage;
#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env, InvokeError, Symbol, Val, Vec};
use axionvera_interfaces::{
    ExecutionWindow, ScheduledTask, ScheduledTaskStatus, SchedulerEngine, SchedulerError,
};
use crate::errors::SchedulerError as Error;
use crate::storage;

const MAX_TASK_NAME_LEN: u32 = 64;

#[contract]
pub struct SchedulerContract;

#[contractimpl]
impl SchedulerContract {
    /// Returns the contract version.
    pub fn version() -> u32 {
        1
    }
}

#[contractimpl]
impl SchedulerEngine for SchedulerContract {
    fn initialize(e: Env, admin: Address) -> Result<(), Error> {
        if storage::is_initialized(&e) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        storage::initialize(&e, &admin);
        events::emit_initialized(&e, admin);
        Ok(())
    }

    fn schedule_task(e: Env, task: ScheduledTask) -> Result<(), Error> {
        storage::require_initialized(&e)?;
        storage::require_not_paused(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();

        validate_task(&e, &task)?;

        if storage::has_task(&e, &task.id) {
            return Err(Error::TaskAlreadyExists);
        }

        storage::set_task(&e, &task);
        events::emit_task_scheduled(&e, task.id.clone(), task.name.clone(), task.priority, admin);
        Ok(())
    }

    fn update_task(e: Env, task: ScheduledTask) -> Result<(), Error> {
        storage::require_initialized(&e)?;
        storage::require_not_paused(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();

        validate_task(&e, &task)?;

        if !storage::has_task(&e, &task.id) {
            return Err(Error::TaskNotFound);
        }

        storage::set_task(&e, &task);
        events::emit_task_updated(&e, task.id.clone(), task.name.clone(), admin);
        Ok(())
    }

    fn cancel_task(e: Env, task_id: BytesN<32>) -> Result<(), Error> {
        storage::require_initialized(&e)?;
        storage::require_not_paused(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();

        let mut task = storage::get_task(&e, &task_id)?;
        if task.status != ScheduledTaskStatus::Pending {
            return Err(Error::TaskNotPending);
        }

        task.status = ScheduledTaskStatus::Canceled;
        storage::set_task(&e, &task);
        events::emit_task_canceled(&e, task_id, admin);
        Ok(())
    }

    fn execute_ready_tasks(e: Env) -> Result<Vec<BytesN<32>>, Error> {
        storage::require_initialized(&e)?;
        storage::require_not_paused(&e)?;

        let mut executed_tasks = Vec::new(&e);
        let task_ids = storage::get_task_list(&e);
        let current_time = e.ledger().timestamp();

        // First, collect all ready tasks
        let mut ready_tasks = Vec::new(&e);
        for task_id in task_ids.iter() {
            if let Ok(task) = storage::get_task(&e, &task_id) {
                if is_task_ready(&e, &task, current_time) {
                    ready_tasks.push_back(task);
                }
            }
        }

        // Sort tasks by priority (descending)
        let mut sorted_tasks = sort_tasks_by_priority(ready_tasks);

        // Execute tasks in order
        for mut task in sorted_tasks.iter() {
            // Mark task as executing
            task.status = ScheduledTaskStatus::Executing;
            storage::set_task(&e, &task);

            // Execute the task
            let success = match e.try_invoke_contract::<(), InvokeError>(
                &task.target_contract,
                &task.target_function,
                task.args.clone(),
            ) {
                Ok(Ok(())) => true,
                _ => false,
            };

            // Update task status
            task.last_executed_at = Some(current_time);
            task.execution_count += 1;

            if success {
                // Check if task should recur
                if should_recur(&task) {
                    // Prepare next recurrence
                    task.status = ScheduledTaskStatus::Pending;
                    // Note: In a real implementation, you would update the window start time
                } else {
                    task.status = ScheduledTaskStatus::Success;
                }
                events::emit_task_executed(&e, task.id.clone(), task.name.clone(), task.execution_count);
                executed_tasks.push_back(task.id.clone());
            } else {
                task.status = ScheduledTaskStatus::Failed;
                events::emit_task_failed(&e, task.id.clone(), task.name.clone());
            }

            storage::set_task(&e, &task);
        }

        Ok(executed_tasks)
    }

    fn get_task(e: Env, task_id: BytesN<32>) -> Result<ScheduledTask, Error> {
        storage::require_initialized(&e)?;
        storage::get_task(&e, &task_id)
    }

    fn list_tasks(e: Env) -> Result<Vec<ScheduledTask>, Error> {
        storage::require_initialized(&e)?;
        let task_ids = storage::get_task_list(&e);
        let mut tasks = Vec::new(&e);
        for id in task_ids.iter() {
            if let Ok(task) = storage::get_task(&e, &id) {
                tasks.push_back(task);
            }
        }
        Ok(tasks)
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

// ---------------------------------------------------------------------------
// Validation and helper functions
// ---------------------------------------------------------------------------

fn validate_task(e: &Env, task: &ScheduledTask) -> Result<(), Error> {
    // Validate task name
    if task.name.len() == 0 || task.name.len() > MAX_TASK_NAME_LEN {
        return Err(Error::InvalidTaskName);
    }

    // Validate execution window
    if task.window.start_time >= task.window.end_time {
        return Err(Error::InvalidExecutionWindow);
    }

    Ok(())
}

fn is_task_ready(e: &Env, task: &ScheduledTask, current_time: u64) -> bool {
    // Check status
    if task.status != ScheduledTaskStatus::Pending {
        return false;
    }

    // Check if in execution window
    if current_time < task.window.start_time || current_time > task.window.end_time {
        return false;
    }

    // Check if max recurrences reached
    if let Some(max) = task.window.max_recurrences {
        if task.execution_count >= max {
            return false;
        }
    }

    // Check dependencies
    for dep_id in task.dependencies.iter() {
        if let Ok(dep_task) = storage::get_task(e, &dep_id) {
            if dep_task.status != ScheduledTaskStatus::Success {
                return false;
            }
        } else {
            // Dependency not found, can't run
            return false;
        }
    }

    true
}

fn should_recur(task: &ScheduledTask) -> bool {
    // Check if recurrence is configured
    if task.window.recurrence_interval.is_none() {
        return false;
    }

    // Check if max recurrences not reached
    if let Some(max) = task.window.max_recurrences {
        if task.execution_count >= max {
            return false;
        }
    }

    true
}

fn sort_tasks_by_priority(mut tasks: Vec<ScheduledTask>) -> Vec<ScheduledTask> {
    // Simple bubble sort for descending priority (since we have no stdlib)
    let n = tasks.len();
    for i in 0..n {
        for j in 0..(n - i - 1) {
            let a = tasks.get(j).unwrap();
            let b = tasks.get(j + 1).unwrap();
            if a.priority < b.priority {
                let temp = tasks.remove(j).unwrap();
                tasks.insert(j, tasks.remove(j).unwrap());
                tasks.insert(j + 1, temp);
            }
        }
    }
    tasks
}
