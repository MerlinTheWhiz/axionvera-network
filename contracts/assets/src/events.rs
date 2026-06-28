use soroban_sdk::{Address, Bytes, Env};

use axionvera_events::{
    self, AssetDeregisteredEvent, AssetRegisteredEvent, AssetRegistryAdminTransferAcceptedEvent,
    AssetRegistryAdminTransferProposedEvent, AssetRegistryPausedEvent,
    AssetRegistryUnpausedEvent, AssetStatusChangedEvent, EVENT_VERSION, PROTOCOL_ASSETS,
    ACT_ASSET_ADM_A, ACT_ASSET_ADM_P, ACT_ASSET_DEREG, ACT_ASSET_PAUSE, ACT_ASSET_REG,
    ACT_ASSET_STATUS, ACT_ASSET_UNPAU,
};

pub fn emit_asset_registered(
    e: &Env,
    asset: Address,
    name: Bytes,
    symbol: Bytes,
    decimals: u32,
    registered_by: Address,
) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL_ASSETS, ACT_ASSET_REG),
        AssetRegisteredEvent {
            event_version: EVENT_VERSION,
            asset,
            name,
            symbol,
            decimals,
            registered_by,
            timestamp: ts,
        },
    );
}

pub fn emit_asset_deregistered(e: &Env, asset: Address, deregistered_by: Address) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL_ASSETS, ACT_ASSET_DEREG),
        AssetDeregisteredEvent {
            event_version: EVENT_VERSION,
            asset,
            deregistered_by,
            timestamp: ts,
        },
    );
}

pub fn emit_asset_status_changed(e: &Env, asset: Address, is_active: bool, changed_by: Address) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL_ASSETS, ACT_ASSET_STATUS),
        AssetStatusChangedEvent {
            event_version: EVENT_VERSION,
            asset,
            is_active,
            changed_by,
            timestamp: ts,
        },
    );
}

pub fn emit_admin_transfer_proposed(e: &Env, current_admin: Address, pending_admin: Address) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL_ASSETS, ACT_ASSET_ADM_P),
        AssetRegistryAdminTransferProposedEvent {
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
        (PROTOCOL_ASSETS, ACT_ASSET_ADM_A),
        AssetRegistryAdminTransferAcceptedEvent {
            event_version: EVENT_VERSION,
            previous_admin,
            new_admin,
            timestamp: ts,
        },
    );
}

pub fn emit_paused(e: &Env, admin: Address) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL_ASSETS, ACT_ASSET_PAUSE),
        AssetRegistryPausedEvent {
            event_version: EVENT_VERSION,
            admin,
            timestamp: ts,
        },
    );
}

pub fn emit_unpaused(e: &Env, admin: Address) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL_ASSETS, ACT_ASSET_UNPAU),
        AssetRegistryUnpausedEvent {
            event_version: EVENT_VERSION,
            admin,
            timestamp: ts,
        },
    );
}
