use soroban_sdk::{Address, BytesN, Env};

use axionvera_core;
use axionvera_events::{
    self, AdminTransferAcceptedEvent, AdminTransferProposedEvent, AssetAddedEvent, AssetClaimEvent,
    AssetDepositEvent, AssetDistributeEvent, AssetWithdrawEvent, ClaimEvent,
    DelegateActionEvent, DelegateAuthorizedEvent, DelegateRevokedEvent, DepositEvent,
    DistributeEvent, InitializeEvent, LockEvent, PauseEvent, UnlockEvent, UnpauseEvent,
    UpgradeEvent, WithdrawEvent, ACT_ADMIN_ACCEPTED, ACT_ADMIN_PROPOSED, ACT_ASSET_ADDED,
    ACT_ASSET_CLAIM, ACT_ASSET_DEPOSIT, ACT_ASSET_DISTRIBUTE, ACT_ASSET_WITHDRAW, ACT_CLAIM,
    ACT_DELEGATE, ACT_DELEGATED_ACTION, ACT_DEPOSIT, ACT_DISTRIBUTE, ACT_INIT, ACT_LOCK, ACT_PAUSE,
    ACT_REVOKE_DELEGATION, ACT_UNLOCK, ACT_UNPAUSE, ACT_UPGRADE, ACT_VESTING_CLAIMED,
    ACT_VESTING_CREATED, ACT_WITHDRAW, EVENT_VERSION, PROTOCOL,
};

pub fn emit_initialize(e: &Env, admin: Address, deposit_token: Address, reward_token: Address) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL, ACT_INIT),
        InitializeEvent {
            event_version: EVENT_VERSION,
            admin: admin.clone(),
            deposit_token,
            reward_token,
            timestamp: ts,
        },
    );
}

pub fn emit_deposit(e: &Env, user: Address, amount: i128) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL, ACT_DEPOSIT),
        DepositEvent {
            event_version: EVENT_VERSION,
            user: user.clone(),
            amount,
            timestamp: ts,
        },
    );
    axionvera_core::index_event(e, ACT_DEPOSIT, Some(user.clone()), None, amount);
    axionvera_core::record_interacting_user(e, &user);
}

pub fn emit_withdraw(e: &Env, user: Address, amount: i128, remaining_balance: i128) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL, ACT_WITHDRAW),
        WithdrawEvent {
            event_version: EVENT_VERSION,
            user: user.clone(),
            amount,
            remaining_balance,
            timestamp: ts,
        },
    );
    axionvera_core::index_event(e, ACT_WITHDRAW, Some(user.clone()), None, amount);
}

pub fn emit_distribute(e: &Env, caller: Address, amount: i128) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL, ACT_DISTRIBUTE),
        DistributeEvent {
            event_version: EVENT_VERSION,
            caller,
            amount,
            timestamp: ts,
        },
    );
}

pub fn emit_claim_rewards(e: &Env, user: Address, amount: i128) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL, ACT_CLAIM),
        ClaimEvent {
            event_version: EVENT_VERSION,
            user: user.clone(),
            amount,
            timestamp: ts,
        },
    );
    axionvera_core::index_event(e, ACT_CLAIM, Some(user.clone()), None, amount);
}

pub fn emit_vesting_created(
    e: &Env,
    user: Address,
    asset: Option<Address>,
    amount: i128,
    start_timestamp: u64,
    duration: u64,
) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL, ACT_VESTING_CREATED),
        axionvera_events::VestingCreatedEvent {
            event_version: EVENT_VERSION,
            user,
            asset,
            amount,
            start_timestamp,
            duration,
            timestamp: ts,
        },
    );
}

pub fn emit_vesting_claimed(
    e: &Env,
    user: Address,
    asset: Option<Address>,
    amount: i128,
    remaining_unclaimed: i128,
) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL, ACT_VESTING_CLAIMED),
        axionvera_events::VestingClaimedEvent {
            event_version: EVENT_VERSION,
            user,
            asset,
            amount,
            remaining_unclaimed,
            timestamp: ts,
        },
    );
}

pub fn emit_lock(e: &Env, user: Address, amount: i128, unlock_timestamp: u64) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL, ACT_LOCK),
        LockEvent {
            event_version: EVENT_VERSION,
            user: user.clone(),
            amount,
            unlock_timestamp,
            timestamp: ts,
        },
    );
    axionvera_core::index_event(e, ACT_LOCK, Some(user.clone()), None, amount);
}

pub fn emit_unlock(e: &Env, user: Address, amount: i128) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL, ACT_UNLOCK),
        UnlockEvent {
            event_version: EVENT_VERSION,
            user: user.clone(),
            amount,
            timestamp: ts,
        },
    );
    axionvera_core::index_event(e, ACT_UNLOCK, Some(user.clone()), None, amount);
}

pub fn emit_admin_transfer_proposed(e: &Env, current_admin: Address, pending_admin: Address) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL, ACT_ADMIN_PROPOSED),
        AdminTransferProposedEvent {
            event_version: EVENT_VERSION,
            current_admin,
            pending_admin,
            timestamp: ts,
        },
    );
}

pub fn emit_admin_transfer_accepted(e: &Env, previous_admin: Address, new_admin: Address) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL, ACT_ADMIN_ACCEPTED),
        AdminTransferAcceptedEvent {
            event_version: EVENT_VERSION,
            previous_admin,
            new_admin,
            timestamp: ts,
        },
    );
}

pub fn emit_delegate(
    e: &Env,
    delegator: Address,
    operator: Address,
    permissions: u32,
    expires_at: u64,
) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL, ACT_DELEGATE),
        axionvera_events::DelegateEvent {
            event_version: EVENT_VERSION,
            delegator,
            operator,
            permissions,
            expires_at,
            timestamp: ts,
        },
    );
}

pub fn emit_revoke_delegation(e: &Env, delegator: Address, operator: Address) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL, ACT_REVOKE_DELEGATION),
        axionvera_events::RevokeDelegationEvent {
            event_version: EVENT_VERSION,
            delegator,
            operator,
            timestamp: ts,
        },
    );
}

pub fn emit_delegated_action(
    e: &Env,
    delegator: Address,
    operator: Address,
    permission: u32,
    action: soroban_sdk::Symbol,
) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL, ACT_DELEGATED_ACTION),
        axionvera_events::DelegatedActionEvent {
            event_version: EVENT_VERSION,
            delegator,
            operator,
            permission,
            action,
            timestamp: ts,
        },
    );
}

pub fn emit_upgrade(e: &Env, admin: Address, new_wasm_hash: BytesN<32>) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL, ACT_UPGRADE),
        UpgradeEvent {
            event_version: EVENT_VERSION,
            admin,
            new_wasm_hash,
            timestamp: ts,
        },
    );
}

pub fn emit_pause(e: &Env, admin: Address) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL, ACT_PAUSE),
        PauseEvent {
            event_version: EVENT_VERSION,
            admin,
            timestamp: ts,
        },
    );
}

pub fn emit_unpause(e: &Env, admin: Address) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL, ACT_UNPAUSE),
        UnpauseEvent {
            event_version: EVENT_VERSION,
            admin,
            timestamp: ts,
        },
    );
}

pub fn emit_asset_added(e: &Env, asset: Address) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL, ACT_ASSET_ADDED),
        AssetAddedEvent {
            event_version: EVENT_VERSION,
            asset,
            timestamp: ts,
        },
    );
}

pub fn emit_asset_deposit(e: &Env, user: Address, asset: Address, amount: i128) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL, ACT_ASSET_DEPOSIT),
        AssetDepositEvent {
            event_version: EVENT_VERSION,
            user: user.clone(),
            asset: asset.clone(),
            amount,
            timestamp: ts,
        },
    );
    axionvera_core::index_event(
        e,
        ACT_ASSET_DEPOSIT,
        Some(user.clone()),
        Some(asset),
        amount,
    );
}

pub fn emit_asset_withdraw(
    e: &Env,
    user: Address,
    asset: Address,
    amount: i128,
    remaining_balance: i128,
) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL, ACT_ASSET_WITHDRAW),
        AssetWithdrawEvent {
            event_version: EVENT_VERSION,
            user: user.clone(),
            asset: asset.clone(),
            amount,
            remaining_balance,
            timestamp: ts,
        },
    );
    axionvera_core::index_event(
        e,
        ACT_ASSET_WITHDRAW,
        Some(user.clone()),
        Some(asset),
        amount,
    );
}

pub fn emit_asset_distribute(e: &Env, caller: Address, asset: Address, amount: i128) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL, ACT_ASSET_DISTRIBUTE),
        AssetDistributeEvent {
            event_version: EVENT_VERSION,
            caller,
            asset,
            amount,
            timestamp: ts,
        },
    );
}

pub fn emit_asset_claim_rewards(e: &Env, user: Address, asset: Address, amount: i128) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL, ACT_ASSET_CLAIM),
        AssetClaimEvent {
            event_version: EVENT_VERSION,
            user: user.clone(),
            asset: asset.clone(),
            amount,
            timestamp: ts,
        },
    );
    axionvera_core::index_event(e, ACT_ASSET_CLAIM, Some(user.clone()), Some(asset), amount);
}

pub fn emit_delegate_authorized(e: &Env, owner: Address, delegate: Address, permissions: u32) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL, ACT_DELEGATE_AUTH),
        DelegateAuthorizedEvent {
            event_version: EVENT_VERSION,
            owner: owner.clone(),
            delegate: delegate.clone(),
            permissions,
            timestamp: ts,
        },
    );
    axionvera_core::index_event(
        e,
        ACT_DELEGATE_AUTH,
        Some(owner),
        Some(delegate),
        permissions as i128,
    );
}

pub fn emit_delegate_revoked(e: &Env, owner: Address, delegate: Address) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL, ACT_DELEGATE_REVOKE),
        DelegateRevokedEvent {
            event_version: EVENT_VERSION,
            owner: owner.clone(),
            delegate: delegate.clone(),
            timestamp: ts,
        },
    );
    axionvera_core::index_event(e, ACT_DELEGATE_REVOKE, Some(owner), Some(delegate), 0);
}

pub fn emit_delegate_action(
    e: &Env,
    owner: Address,
    delegate: Address,
    action: soroban_sdk::Symbol,
) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL, ACT_DELEGATED_ACTION),
        DelegateActionEvent {
            event_version: EVENT_VERSION,
            owner: owner.clone(),
            delegate: delegate.clone(),
            action,
            timestamp: ts,
        },
    );
    axionvera_core::index_event(e, ACT_DELEGATED_ACTION, Some(owner), Some(delegate), 1);
}
