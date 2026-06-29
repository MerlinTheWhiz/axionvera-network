#![no_std]

use soroban_sdk::{contracttype, symbol_short, Address, Bytes, BytesN, Env, Symbol};

/// Current event schema version.
pub const EVENT_VERSION: u32 = 1;

/// Protocol identifier used as Topic 1 for all vault events.
pub const PROTOCOL: Symbol = symbol_short!("AxVault");

// ---------------------------------------------------------------------------
// Action Symbols — used as Topic 2 for all events
// ---------------------------------------------------------------------------
pub const ACT_INIT: Symbol = symbol_short!("init");
pub const ACT_DEPOSIT: Symbol = symbol_short!("deposit");
pub const ACT_WITHDRAW: Symbol = symbol_short!("withdraw");
pub const ACT_DISTRIBUTE: Symbol = symbol_short!("distrib");
pub const ACT_CLAIM: Symbol = symbol_short!("claim");
pub const ACT_LOCK: Symbol = symbol_short!("lock");
pub const ACT_UNLOCK: Symbol = symbol_short!("unlock");
pub const ACT_ADMIN_PROPOSED: Symbol = symbol_short!("admin_prp");
pub const ACT_ADMIN_ACCEPTED: Symbol = symbol_short!("adm_acpt");
pub const ACT_UPGRADE: Symbol = symbol_short!("upgrade");
pub const ACT_PAUSE: Symbol = symbol_short!("pause");
pub const ACT_UNPAUSE: Symbol = symbol_short!("unpause");
pub const ACT_ASSET_ADDED: Symbol = symbol_short!("asset_add");
pub const ACT_ASSET_DEPOSIT: Symbol = symbol_short!("asset_dep");
pub const ACT_ASSET_WITHDRAW: Symbol = symbol_short!("asset_wd");
pub const ACT_ASSET_DISTRIBUTE: Symbol = symbol_short!("ast_dist");
pub const ACT_ASSET_CLAIM: Symbol = symbol_short!("asset_clm");
pub const ACT_DELEGATE: Symbol = symbol_short!("delegate");
pub const ACT_REVOKE_DELEGATION: Symbol = symbol_short!("rvk_dlg");
pub const ACT_DELEGATED_ACTION: Symbol = symbol_short!("deleg_act");

// ---------------------------------------------------------------------------
// Storage keys used by the indexing layer
// ---------------------------------------------------------------------------
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Global event log (Vec<EventLogEntry>)
    EventLog,
    /// Per-user event log keyed by address (Vec<EventLogEntry>)
    UserEventLog(Address),
    /// Set of all users who have ever interacted (Map<Address, bool>)
    InteractingUsers,
}

// ---------------------------------------------------------------------------
// Event payload structs
// All events follow the two-topic (PROTOCOL, ACTION) design
// and include an `event_version` field for schema evolution.
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitializeEvent {
    pub event_version: u32,
    pub admin: Address,
    pub deposit_token: Address,
    pub reward_token: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DepositEvent {
    pub event_version: u32,
    pub user: Address,
    pub amount: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WithdrawEvent {
    pub event_version: u32,
    pub user: Address,
    pub amount: i128,
    pub remaining_balance: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DistributeEvent {
    pub event_version: u32,
    pub caller: Address,
    pub amount: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimEvent {
    pub event_version: u32,
    pub user: Address,
    pub amount: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminTransferProposedEvent {
    pub event_version: u32,
    pub current_admin: Address,
    pub pending_admin: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminTransferAcceptedEvent {
    pub event_version: u32,
    pub previous_admin: Address,
    pub new_admin: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpgradeEvent {
    pub event_version: u32,
    pub admin: Address,
    pub new_wasm_hash: BytesN<32>,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PauseEvent {
    pub event_version: u32,
    pub admin: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnpauseEvent {
    pub event_version: u32,
    pub admin: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetAddedEvent {
    pub event_version: u32,
    pub asset: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetDepositEvent {
    pub event_version: u32,
    pub user: Address,
    pub asset: Address,
    pub amount: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetWithdrawEvent {
    pub event_version: u32,
    pub user: Address,
    pub asset: Address,
    pub amount: i128,
    pub remaining_balance: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetDistributeEvent {
    pub event_version: u32,
    pub caller: Address,
    pub asset: Address,
    pub amount: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetClaimEvent {
    pub event_version: u32,
    pub user: Address,
    pub asset: Address,
    pub amount: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LockEvent {
    pub event_version: u32,
    pub user: Address,
    pub amount: i128,
    pub unlock_timestamp: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnlockEvent {
    pub event_version: u32,
    pub user: Address,
    pub amount: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelegateEvent {
    pub event_version: u32,
    pub delegator: Address,
    pub operator: Address,
    pub permissions: u32,
    pub expires_at: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevokeDelegationEvent {
    pub event_version: u32,
    pub delegator: Address,
    pub operator: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelegatedActionEvent {
    pub event_version: u32,
    pub delegator: Address,
    pub operator: Address,
    pub permission: u32,
    pub action: Symbol,
    pub timestamp: u64,
}

// ---------------------------------------------------------------------------
// Helper: get the ledger timestamp
// ---------------------------------------------------------------------------

pub fn ledger_timestamp(e: &Env) -> u64 {
    e.ledger().timestamp()
}

// ---------------------------------------------------------------------------
// Config contract — protocol identifier and action symbols
// ---------------------------------------------------------------------------

/// Protocol identifier used as Topic 1 for all config contract events.
pub const PROTOCOL_CONFIG: Symbol = symbol_short!("AxCfg");

pub const ACT_CFG_INIT: Symbol = symbol_short!("cfg_init");
pub const ACT_CFG_PR_UPD: Symbol = symbol_short!("pr_upd");
pub const ACT_CFG_VP_UPD: Symbol = symbol_short!("vp_upd");
pub const ACT_CFG_TD_UPD: Symbol = symbol_short!("td_upd");
pub const ACT_CFG_MR_UPD: Symbol = symbol_short!("mr_upd");
pub const ACT_CFG_MU_UPD: Symbol = symbol_short!("mu_upd");
pub const ACT_CFG_WU_UPD: Symbol = symbol_short!("wu_upd");
pub const ACT_CFG_MA_UPD: Symbol = symbol_short!("ma_upd");
pub const ACT_CFG_ADM_P: Symbol = symbol_short!("cfg_adm_p");
pub const ACT_CFG_ADM_A: Symbol = symbol_short!("cfg_adm_a");
pub const ACT_CFG_PAUSE: Symbol = symbol_short!("cfg_pause");
pub const ACT_CFG_UNPAU: Symbol = symbol_short!("cfg_unpau");

// ---------------------------------------------------------------------------
// Config event payload structs
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigInitializedEvent {
    pub event_version: u32,
    pub admin: Address,
    pub penalty_rate_bps: u32,
    pub vesting_period: u64,
    pub target_deposits: i128,
    pub min_reward_distribution: i128,
    pub max_unlock_limit: u32,
    pub withdraw_unlock_limit: u32,
    pub max_assets: u32,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PenaltyRateUpdatedEvent {
    pub event_version: u32,
    pub admin: Address,
    pub old_rate_bps: u32,
    pub new_rate_bps: u32,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingPeriodUpdatedEvent {
    pub event_version: u32,
    pub admin: Address,
    pub old_period: u64,
    pub new_period: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetDepositsUpdatedEvent {
    pub event_version: u32,
    pub admin: Address,
    pub old_amount: i128,
    pub new_amount: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MinRewardDistributionUpdatedEvent {
    pub event_version: u32,
    pub admin: Address,
    pub old_amount: i128,
    pub new_amount: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaxUnlockLimitUpdatedEvent {
    pub event_version: u32,
    pub admin: Address,
    pub old_limit: u32,
    pub new_limit: u32,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WithdrawUnlockLimitUpdatedEvent {
    pub event_version: u32,
    pub admin: Address,
    pub old_limit: u32,
    pub new_limit: u32,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaxAssetsUpdatedEvent {
    pub event_version: u32,
    pub admin: Address,
    pub old_max: u32,
    pub new_max: u32,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigAdminTransferProposedEvent {
    pub event_version: u32,
    pub current_admin: Address,
    pub pending_admin: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigAdminTransferAcceptedEvent {
    pub event_version: u32,
    pub previous_admin: Address,
    pub new_admin: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigPausedEvent {
    pub event_version: u32,
    pub admin: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigUnpausedEvent {
    pub event_version: u32,
    pub admin: Address,
    pub timestamp: u64,
}

// ---------------------------------------------------------------------------
// Asset registry — protocol identifier and action symbols
// ---------------------------------------------------------------------------

/// Protocol identifier used as Topic 1 for all asset registry events.
pub const PROTOCOL_ASSETS: Symbol = symbol_short!("AxAsset");

pub const ACT_ASSET_REG: Symbol = symbol_short!("ast_reg");
pub const ACT_ASSET_DEREG: Symbol = symbol_short!("ast_dreg");
pub const ACT_ASSET_STATUS: Symbol = symbol_short!("ast_stat");
pub const ACT_ASSET_ADM_P: Symbol = symbol_short!("ast_adm_p");
pub const ACT_ASSET_ADM_A: Symbol = symbol_short!("ast_adm_a");
pub const ACT_ASSET_PAUSE: Symbol = symbol_short!("ast_pause");
pub const ACT_ASSET_UNPAU: Symbol = symbol_short!("ast_unpau");

// ---------------------------------------------------------------------------
// Asset registry event payload structs
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetRegisteredEvent {
    pub event_version: u32,
    pub asset: Address,
    pub name: Bytes,
    pub symbol: Bytes,
    pub decimals: u32,
    pub registered_by: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetDeregisteredEvent {
    pub event_version: u32,
    pub asset: Address,
    pub deregistered_by: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetStatusChangedEvent {
    pub event_version: u32,
    pub asset: Address,
    pub is_active: bool,
    pub changed_by: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetRegistryAdminTransferProposedEvent {
    pub event_version: u32,
    pub current_admin: Address,
    pub pending_admin: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetRegistryAdminTransferAcceptedEvent {
    pub event_version: u32,
    pub previous_admin: Address,
    pub new_admin: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetRegistryPausedEvent {
    pub event_version: u32,
    pub admin: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetRegistryUnpausedEvent {
    pub event_version: u32,
    pub admin: Address,
    pub timestamp: u64,
}

// ---------------------------------------------------------------------------
// Policy Engine — protocol identifier and action symbols
// ---------------------------------------------------------------------------

/// Protocol identifier used as Topic 1 for all policy engine events.
pub const PROTOCOL_POLICY: Symbol = symbol_short!("AxPolicy");

pub const ACT_POL_INIT: Symbol = symbol_short!("pol_init");
pub const ACT_POL_ADD: Symbol = symbol_short!("pol_add");
pub const ACT_POL_UPD: Symbol = symbol_short!("pol_upd");
pub const ACT_POL_DEL: Symbol = symbol_short!("pol_del");
pub const ACT_POL_EVAL: Symbol = symbol_short!("pol_eval");
pub const ACT_POL_ADM_P: Symbol = symbol_short!("pol_adm_p");
pub const ACT_POL_ADM_A: Symbol = symbol_short!("pol_adm_a");
pub const ACT_POL_PAUSE: Symbol = symbol_short!("pol_pause");
pub const ACT_POL_UNPAU: Symbol = symbol_short!("pol_unpau");

// ---------------------------------------------------------------------------
// Policy Engine event payload structs
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyInitializedEvent {
    pub event_version: u32,
    pub admin: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyAddedEvent {
    pub event_version: u32,
    pub policy_id: BytesN<32>,
    pub policy_name: Bytes,
    pub added_by: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyUpdatedEvent {
    pub event_version: u32,
    pub policy_id: BytesN<32>,
    pub policy_name: Bytes,
    pub updated_by: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyDeletedEvent {
    pub event_version: u32,
    pub policy_id: BytesN<32>,
    pub deleted_by: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyEvaluatedEvent {
    pub event_version: u32,
    pub request_caller: Address,
    pub target_contract: Address,
    pub target_function: Symbol,
    pub passed: bool,
    pub failed_policy_id: Option<BytesN<32>>,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyAdminTransferProposedEvent {
    pub event_version: u32,
    pub current_admin: Address,
    pub pending_admin: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyAdminTransferAcceptedEvent {
    pub event_version: u32,
    pub previous_admin: Address,
    pub new_admin: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyPausedEvent {
    pub event_version: u32,
    pub admin: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyUnpausedEvent {
    pub event_version: u32,
    pub admin: Address,
    pub timestamp: u64,
}

// ---------------------------------------------------------------------------
// Replay Engine — protocol identifier and action symbols
// ---------------------------------------------------------------------------

/// Protocol identifier used as Topic 1 for all replay engine events.
pub const PROTOCOL_REPLAY: Symbol = symbol_short!("AxReplay");

pub const ACT_REPLAY_INIT: Symbol = symbol_short!("rep_init");
pub const ACT_REPLAY_START: Symbol = symbol_short!("rep_start");
pub const ACT_REPLAY_COMPLETE: Symbol = symbol_short!("rep_complete");

// ---------------------------------------------------------------------------
// Replay Engine event payload structs
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayInitializedEvent {
    pub event_version: u32,
    pub admin: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayStartedEvent {
    pub event_version: u32,
    pub run_id: BytesN<32>,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayCompletedEvent {
    pub event_version: u32,
    pub run_id: BytesN<32>,
    pub success: bool,
    pub total_events: u64,
    pub successful_events: u64,
    pub timestamp: u64,
}

// ---------------------------------------------------------------------------
// Scheduler Engine — protocol identifier and action symbols
// ---------------------------------------------------------------------------

/// Protocol identifier used as Topic 1 for all scheduler events.
pub const PROTOCOL_SCHEDULER: Symbol = symbol_short!("AxSched");

pub const ACT_SCHED_INIT: Symbol = symbol_short!("sched_init");
pub const ACT_SCHED_TASK_SCHEDULED: Symbol = symbol_short!("task_sched");
pub const ACT_SCHED_TASK_UPDATED: Symbol = symbol_short!("task_upd");
pub const ACT_SCHED_TASK_CANCELED: Symbol = symbol_short!("task_cancel");
pub const ACT_SCHED_TASK_EXECUTED: Symbol = symbol_short!("task_exec");
pub const ACT_SCHED_TASK_FAILED: Symbol = symbol_short!("task_fail");
pub const ACT_SCHED_ADMIN_P: Symbol = symbol_short!("sched_adm_p");
pub const ACT_SCHED_ADMIN_A: Symbol = symbol_short!("sched_adm_a");
pub const ACT_SCHED_PAUSE: Symbol = symbol_short!("sched_pause");
pub const ACT_SCHED_UNPAUSE: Symbol = symbol_short!("sched_unpau");

// ---------------------------------------------------------------------------
// Scheduler event payload structs
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulerInitializedEvent {
    pub event_version: u32,
    pub admin: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskScheduledEvent {
    pub event_version: u32,
    pub task_id: BytesN<32>,
    pub task_name: Bytes,
    pub priority: u32,
    pub created_by: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskUpdatedEvent {
    pub event_version: u32,
    pub task_id: BytesN<32>,
    pub task_name: Bytes,
    pub updated_by: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskCanceledEvent {
    pub event_version: u32,
    pub task_id: BytesN<32>,
    pub canceled_by: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskExecutedEvent {
    pub event_version: u32,
    pub task_id: BytesN<32>,
    pub task_name: Bytes,
    pub execution_count: u32,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskFailedEvent {
    pub event_version: u32,
    pub task_id: BytesN<32>,
    pub task_name: Bytes,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulerAdminTransferProposedEvent {
    pub event_version: u32,
    pub current_admin: Address,
    pub pending_admin: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulerAdminTransferAcceptedEvent {
    pub event_version: u32,
    pub previous_admin: Address,
    pub new_admin: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulerPausedEvent {
    pub event_version: u32,
    pub admin: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulerUnpausedEvent {
    pub event_version: u32,
    pub admin: Address,
    pub timestamp: u64,
}
