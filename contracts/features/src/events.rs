use soroban_sdk::{Address, Env, Symbol};

use axionvera_events::{
    self, FeatureAdminTransferAcceptedEvent, FeatureAdminTransferProposedEvent,
    FeatureDisabledEvent, FeatureEnabledEvent, FeatureInitializedEvent, FeaturePausedEvent,
    FeatureRegisteredEvent, FeatureRolloutUpdatedEvent, FeatureUnpausedEvent, EVENT_VERSION,
    PROTOCOL_FEATURES, ACT_FEAT_ADM_A, ACT_FEAT_ADM_P, ACT_FEAT_DIS, ACT_FEAT_EN,
    ACT_FEAT_INIT, ACT_FEAT_PAUSE, ACT_FEAT_REG, ACT_FEAT_ROLL, ACT_FEAT_UNPAU,
};

pub fn emit_initialized(e: &Env, admin: Address) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL_FEATURES, ACT_FEAT_INIT),
        FeatureInitializedEvent {
            event_version: EVENT_VERSION,
            admin,
            timestamp: ts,
        },
    );
}

pub fn emit_feature_registered(e: &Env, name: Symbol, admin: Address) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL_FEATURES, ACT_FEAT_REG),
        FeatureRegisteredEvent {
            event_version: EVENT_VERSION,
            name,
            admin,
            timestamp: ts,
        },
    );
}

pub fn emit_feature_enabled(e: &Env, name: Symbol, admin: Address) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL_FEATURES, ACT_FEAT_EN),
        FeatureEnabledEvent {
            event_version: EVENT_VERSION,
            name,
            admin,
            timestamp: ts,
        },
    );
}

pub fn emit_feature_disabled(e: &Env, name: Symbol, admin: Address) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL_FEATURES, ACT_FEAT_DIS),
        FeatureDisabledEvent {
            event_version: EVENT_VERSION,
            name,
            admin,
            timestamp: ts,
        },
    );
}

pub fn emit_feature_rollout_updated(
    e: &Env,
    name: Symbol,
    admin: Address,
    old_pct: u32,
    new_pct: u32,
) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL_FEATURES, ACT_FEAT_ROLL),
        FeatureRolloutUpdatedEvent {
            event_version: EVENT_VERSION,
            name,
            admin,
            old_pct,
            new_pct,
            timestamp: ts,
        },
    );
}

pub fn emit_admin_transfer_proposed(e: &Env, current_admin: Address, pending_admin: Address) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL_FEATURES, ACT_FEAT_ADM_P),
        FeatureAdminTransferProposedEvent {
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
        (PROTOCOL_FEATURES, ACT_FEAT_ADM_A),
        FeatureAdminTransferAcceptedEvent {
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
        (PROTOCOL_FEATURES, ACT_FEAT_PAUSE),
        FeaturePausedEvent {
            event_version: EVENT_VERSION,
            admin,
            timestamp: ts,
        },
    );
}

pub fn emit_unpaused(e: &Env, admin: Address) {
    let ts = axionvera_events::ledger_timestamp(e);
    e.events().publish(
        (PROTOCOL_FEATURES, ACT_FEAT_UNPAU),
        FeatureUnpausedEvent {
            event_version: EVENT_VERSION,
            admin,
            timestamp: ts,
        },
    );
}
