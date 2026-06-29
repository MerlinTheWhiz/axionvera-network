#![no_std]

use soroban_sdk::{contracterror, contracttype, Address, BytesN, Env, Symbol, Val, Vec};

// ---------------------------------------------------------------------------
// Vault event emitter contract interface
// ---------------------------------------------------------------------------

/// Trait that all event emitters must implement.
/// Ensures each action emits a well-formed event with the standard two-topic pattern.
pub trait VaultEventEmitter {
    fn emit_initialize(e: &Env, admin: Address, deposit_token: Address, reward_token: Address);
    fn emit_deposit(e: &Env, user: Address, amount: i128);
    fn emit_withdraw(e: &Env, user: Address, amount: i128, remaining_balance: i128);
    fn emit_distribute(e: &Env, caller: Address, amount: i128);
    fn emit_claim_rewards(e: &Env, user: Address, amount: i128);
    fn emit_lock(e: &Env, user: Address, amount: i128, unlock_timestamp: u64);
    fn emit_unlock(e: &Env, user: Address, amount: i128);
    fn emit_admin_transfer_proposed(e: &Env, current_admin: Address, pending_admin: Address);
    fn emit_admin_transfer_accepted(e: &Env, previous_admin: Address, new_admin: Address);
    fn emit_upgrade(e: &Env, admin: Address, new_wasm_hash: BytesN<32>);
    fn emit_pause(e: &Env, admin: Address);
    fn emit_unpause(e: &Env, admin: Address);
    fn emit_delegation_granted(
        e: &Env,
        delegator: Address,
        delegatee: Address,
        operation: Symbol,
        expiration: u64,
    );
    fn emit_delegation_revoked(e: &Env, delegator: Address, delegatee: Address, operation: Symbol);
    fn emit_asset_added(e: &Env, asset: Address);
    fn emit_asset_deposit(e: &Env, user: Address, asset: Address, amount: i128);
    fn emit_asset_withdraw(
        e: &Env,
        user: Address,
        asset: Address,
        amount: i128,
        remaining_balance: i128,
    );
    fn emit_asset_distribute(e: &Env, caller: Address, asset: Address, amount: i128);
    fn emit_asset_claim_rewards(e: &Env, user: Address, asset: Address, amount: i128);
}

// ---------------------------------------------------------------------------
// Treasury allocation framework
// ---------------------------------------------------------------------------

/// Basis point denominator used by treasury and fee calculations.
pub const TREASURY_BPS_DENOMINATOR: u32 = 10_000;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllocationRule {
    pub recipient: Address,
    pub share_bps: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllocationStrategy {
    pub id: BytesN<32>,
    pub rules: Vec<AllocationRule>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllocationTransfer {
    pub recipient: Address,
    pub amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreasuryDistributionReceipt {
    pub distribution_id: BytesN<32>,
    pub strategy_id: BytesN<32>,
    pub asset: Address,
    pub total_amount: i128,
    pub transfers: Vec<AllocationTransfer>,
    pub timestamp: u64,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum TreasuryError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidAmount = 4,
    InvalidShare = 5,
    InvalidShareTotal = 6,
    EmptyStrategy = 7,
    TooManyRules = 8,
    DuplicateRecipient = 9,
    StrategyNotFound = 10,
    DuplicateDistribution = 11,
    InsufficientBalance = 12,
}

/// Interface implemented by contracts that allocate treasury distributions.
pub trait TreasuryAllocator {
    fn initialize(e: Env, admin: Address, asset: Address) -> Result<(), TreasuryError>;
    fn configure_strategy(
        e: Env,
        admin: Address,
        strategy: AllocationStrategy,
    ) -> Result<(), TreasuryError>;
    fn distribute(
        e: Env,
        admin: Address,
        distribution_id: BytesN<32>,
        strategy_id: BytesN<32>,
        amount: i128,
    ) -> Result<TreasuryDistributionReceipt, TreasuryError>;
    fn strategy(e: Env, strategy_id: BytesN<32>) -> Option<AllocationStrategy>;
    fn distribution_receipt(
        e: Env,
        distribution_id: BytesN<32>,
    ) -> Option<TreasuryDistributionReceipt>;
    fn recipient_distributed(e: Env, recipient: Address) -> i128;
    fn total_distributed(e: Env) -> i128;
}

// ---------------------------------------------------------------------------
// Protocol fee framework
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FeeType {
    Deposit,
    Withdrawal,
    Reward,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeeConfig {
    pub treasury: Address,
    pub deposit_fee_bps: u32,
    pub withdrawal_fee_bps: u32,
    pub reward_fee_bps: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeeReceipt {
    pub fee_type: FeeType,
    pub actor: Address,
    pub treasury: Address,
    pub asset: Option<Address>,
    pub gross_amount: i128,
    pub fee_bps: u32,
    pub fee_amount: i128,
    pub net_amount: i128,
    pub treasury_amount: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FeeTotals {
    pub operation_count: u64,
    pub collected_amount: i128,
    pub treasury_amount: i128,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum FeeError {
    InvalidAmount = 1,
    InvalidFeeRate = 2,
    MathOverflow = 3,
}

impl FeeConfig {
    pub fn rate_for(&self, fee_type: FeeType) -> u32 {
        match fee_type {
            FeeType::Deposit => self.deposit_fee_bps,
            FeeType::Withdrawal => self.withdrawal_fee_bps,
            FeeType::Reward => self.reward_fee_bps,
        }
    }
}

// ---------------------------------------------------------------------------
// Cross-contract orchestration
// ---------------------------------------------------------------------------

/// A single operation inside a cross-contract execution plan.
///
/// `depends_on` lists operation ids that must appear earlier in the same plan.
/// `rollback` contains zero or one compensating calls that are scheduled if
/// this operation completed before a later step failed.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchestrationOperation {
    pub id: u32,
    pub target: Address,
    pub function: Symbol,
    pub args: Vec<Val>,
    pub depends_on: Vec<u32>,
    pub rollback: Vec<RollbackOperation>,
}

/// A compensating call for an executed operation.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RollbackOperation {
    pub target: Address,
    pub function: Symbol,
    pub args: Vec<Val>,
}

/// A deterministic execution plan for coordinating multiple contract calls.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionPlan {
    pub id: BytesN<32>,
    pub caller: Address,
    pub operations: Vec<OrchestrationOperation>,
}

/// State recorded for a single operation in an execution receipt.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperationStatus {
    Pending,
    Executed,
    RolledBack,
}

/// Final state of an execution plan.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutionStatus {
    Succeeded,
    Failed,
    RolledBack,
}

/// Per-operation receipt data persisted by the orchestrator.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationReceipt {
    pub operation_id: u32,
    pub status: OperationStatus,
}

/// Receipt persisted after every attempted orchestration run.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionReceipt {
    pub plan_id: BytesN<32>,
    pub caller: Address,
    pub status: ExecutionStatus,
    pub executed: Vec<OperationReceipt>,
    pub rollback: Vec<OperationReceipt>,
    pub failed_operation: Option<u32>,
    pub timestamp: u64,
}

/// Errors returned by orchestration validation and execution.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum OrchestrationError {
    EmptyPlan = 1,
    TooManyOperations = 2,
    DuplicateOperationId = 3,
    InvalidTarget = 4,
    InvalidDependency = 5,
    DependencyNotOrdered = 6,
    OperationFailed = 7,
    RollbackFailed = 8,
}

/// Interface implemented by contracts that coordinate execution plans.
pub trait TransactionOrchestrator {
    fn validate_plan(e: Env, plan: ExecutionPlan) -> Result<(), OrchestrationError>;
    fn execute_plan(e: Env, plan: ExecutionPlan) -> Result<ExecutionReceipt, OrchestrationError>;
    fn execution_receipt(e: Env, plan_id: BytesN<32>) -> Option<ExecutionReceipt>;
}

// ===========================================================================
// EXTERNAL SOROBAN CONTRACT INTEGRATION INTERFACES
// ===========================================================================

/// Security policy for standardized calls into external Soroban contracts.
///
/// Protocol contracts should keep allow-lists narrow and explicitly enumerate
/// both trusted target addresses and approved function symbols. `allow_self_call`
/// defaults should remain false in gateway implementations to prevent accidental
/// recursion into the calling protocol contract.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalCallPolicy {
    pub allowed_targets: Vec<Address>,
    pub allowed_functions: Vec<Symbol>,
    pub max_arguments: u32,
    pub allow_self_call: bool,
}

/// Standard envelope for an external contract call.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalCall {
    pub target: Address,
    pub function: Symbol,
    pub args: Vec<Val>,
}

/// Audit receipt emitted or stored by integration gateways after attempts.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalCallReceipt {
    pub target: Address,
    pub function: Symbol,
    pub success: bool,
    pub timestamp: u64,
}

/// Validation and execution failures for external integrations.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum IntegrationError {
    EmptyTargetAllowList = 1,
    EmptyFunctionAllowList = 2,
    TargetNotAllowed = 3,
    FunctionNotAllowed = 4,
    TooManyArguments = 5,
    SelfCallBlocked = 6,
    ExternalInvocationFailed = 7,
}

/// Reusable interface for validating and invoking external contract calls.
pub trait ExternalContractGateway {
    fn validate_call(
        e: &Env,
        call: &ExternalCall,
        policy: &ExternalCallPolicy,
    ) -> Result<(), IntegrationError>;

    fn invoke_void(
        e: &Env,
        call: &ExternalCall,
        policy: &ExternalCallPolicy,
    ) -> Result<ExternalCallReceipt, IntegrationError>;
}

// ---------------------------------------------------------------------------
// Event Replay Framework Types
// ---------------------------------------------------------------------------

/// The status of a replayed event.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplayEventStatus {
    /// Event is pending replay.
    Pending,
    /// Event was successfully replayed.
    Success,
    /// Event replay failed.
    Failed,
    /// Event was skipped (e.g., not applicable).
    Skipped,
}

/// A single event in the replay log.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayEvent {
    /// Unique identifier for this event in the replay log.
    pub id: u64,
    /// Protocol identifier (e.g., PROTOCOL, PROTOCOL_CONFIG).
    pub protocol: Symbol,
    /// Action symbol (e.g., ACT_INIT, ACT_DEPOSIT).
    pub action: Symbol,
    /// Timestamp of the original event.
    pub timestamp: u64,
    /// Raw event payload (encoded as Val).
    pub payload: Val,
    /// Status of this event in the replay.
    pub status: ReplayEventStatus,
    /// Error message if replay failed (empty if success).
    pub error_message: Bytes,
}

/// Result of a full replay run.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayReport {
    /// Unique identifier for this replay run.
    pub run_id: BytesN<32>,
    /// Total number of events processed.
    pub total_events: u64,
    /// Number of successful events.
    pub successful_events: u64,
    /// Number of failed events.
    pub failed_events: u64,
    /// Number of skipped events.
    pub skipped_events: u64,
    /// Timestamp when replay started.
    pub started_at: u64,
    /// Timestamp when replay ended.
    pub ended_at: u64,
    /// Whether the entire replay was considered successful.
    pub success: bool,
}

/// Errors returned by replay engine operations.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ReplayError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    EventAlreadyAdded = 4,
    InvalidEvent = 5,
    ReplayInProgress = 6,
    ReplayFailed = 7,
}

/// Interface implemented by the event replay engine.
pub trait EventReplayEngine {
    /// Initializes the replay engine with an admin.
    fn initialize(e: Env, admin: Address) -> Result<(), ReplayError>;

    /// Adds a historical event to the replay log.
    fn add_event(
        e: Env,
        protocol: Symbol,
        action: Symbol,
        timestamp: u64,
        payload: Val,
    ) -> Result<u64, ReplayError>;

    /// Starts replaying events from the beginning or last checkpoint.
    fn start_replay(e: Env) -> Result<ReplayReport, ReplayError>;

    /// Gets a replay event by ID.
    fn get_event(e: Env, event_id: u64) -> Result<ReplayEvent, ReplayError>;

    /// Lists all replay events.
    fn list_events(e: Env) -> Result<Vec<ReplayEvent>, ReplayError>;

    /// Gets a replay report by run ID.
    fn get_report(e: Env, run_id: BytesN<32>) -> Result<ReplayReport, ReplayError>;

    /// Gets the current admin.
    fn admin(e: Env) -> Result<Address, ReplayError>;
}

// ---------------------------------------------------------------------------
// Scheduling Engine Types
// ---------------------------------------------------------------------------

/// The status of a scheduled task.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScheduledTaskStatus {
    /// Task is scheduled but not yet executed.
    Pending,
    /// Task is currently executing.
    Executing,
    /// Task executed successfully.
    Success,
    /// Task execution failed.
    Failed,
    /// Task was canceled.
    Canceled,
}

/// An execution window that defines when a task can run.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionWindow {
    /// Start timestamp (inclusive) of the window.
    pub start_time: u64,
    /// End timestamp (inclusive) of the window.
    pub end_time: u64,
    /// Optional: recurrence interval (for recurring tasks).
    pub recurrence_interval: Option<u64>,
    /// Optional: maximum number of recurrences.
    pub max_recurrences: Option<u32>,
}

/// A scheduled task that coordinates protocol operations.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduledTask {
    /// Unique identifier for the task.
    pub id: BytesN<32>,
    /// Human-readable name for the task.
    pub name: Bytes,
    /// Priority (higher = executed first when multiple tasks are ready).
    pub priority: u32,
    /// Execution window for the task.
    pub window: ExecutionWindow,
    /// Contract address to call.
    pub target_contract: Address,
    /// Function symbol to call.
    pub target_function: Symbol,
    /// Arguments for the function.
    pub args: Vec<Val>,
    /// List of task IDs that must complete before this task runs.
    pub dependencies: Vec<BytesN<32>>,
    /// Current status of the task.
    pub status: ScheduledTaskStatus,
    /// Number of times the task has executed.
    pub execution_count: u32,
    /// Timestamp when the task was created.
    pub created_at: u64,
    /// Timestamp when the task was last executed (if any).
    pub last_executed_at: Option<u64>,
}

/// Errors returned by the scheduling engine.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum SchedulerError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    TaskAlreadyExists = 4,
    TaskNotFound = 5,
    InvalidTaskName = 6,
    InvalidExecutionWindow = 7,
    TaskNotPending = 8,
    ConflictingTasks = 9,
    DependenciesNotMet = 10,
    NotInExecutionWindow = 11,
    ContractPaused = 12,
    NoPendingAdmin = 13,
}

/// Interface implemented by the scheduling engine.
pub trait SchedulerEngine {
    /// Initializes the scheduler with an admin.
    fn initialize(e: Env, admin: Address) -> Result<(), SchedulerError>;

    /// Schedules a new task.
    fn schedule_task(e: Env, task: ScheduledTask) -> Result<(), SchedulerError>;

    /// Updates an existing task.
    fn update_task(e: Env, task: ScheduledTask) -> Result<(), SchedulerError>;

    /// Cancels a pending task.
    fn cancel_task(e: Env, task_id: BytesN<32>) -> Result<(), SchedulerError>;

    /// Executes all ready tasks (in priority order).
    fn execute_ready_tasks(e: Env) -> Result<Vec<BytesN<32>>, SchedulerError>;

    /// Gets a task by ID.
    fn get_task(e: Env, task_id: BytesN<32>) -> Result<ScheduledTask, SchedulerError>;

    /// Lists all tasks.
    fn list_tasks(e: Env) -> Result<Vec<ScheduledTask>, SchedulerError>;

    /// Gets the current admin.
    fn admin(e: Env) -> Result<Address, SchedulerError>;

    /// Proposes a new admin.
    fn propose_new_admin(e: Env, new_admin: Address) -> Result<(), SchedulerError>;

    /// Accepts the admin role.
    fn accept_admin(e: Env, new_admin: Address) -> Result<(), SchedulerError>;

    /// Pauses the scheduler.
    fn pause_contract(e: Env) -> Result<(), SchedulerError>;

    /// Unpauses the scheduler.
    fn unpause_contract(e: Env) -> Result<(), SchedulerError>;

    /// Checks if the scheduler is paused.
    fn is_paused(e: Env) -> bool;
}
