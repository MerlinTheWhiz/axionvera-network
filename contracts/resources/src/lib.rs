#![no_std]

#[cfg(test)]
mod test;

use soroban_sdk::{contracterror, contracttype, Address, Bytes, Env, Symbol};

use axionvera_events as events;

// ===========================================================================
// Error Types
// ===========================================================================

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ResourceError {
    InvalidTransition = 2001,
    AlreadyInState = 2002,
    Unauthorized = 2003,
    NotFound = 2004,
    AlreadyExists = 2005,
}

// ===========================================================================
// Resource State Machine
// ===========================================================================

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ResourceState {
    Created = 0,
    Active = 1,
    Suspended = 2,
    Archived = 3,
    Retired = 4,
}

impl ResourceState {
    /// Validates and returns the next state or returns a ResourceError if invalid.
    pub fn transition(&self, next: ResourceState) -> Result<ResourceState, ResourceError> {
        if *self == next {
            return Err(ResourceError::AlreadyInState);
        }
        match (*self, next) {
            (ResourceState::Created, ResourceState::Active) => Ok(next),
            (ResourceState::Active, ResourceState::Suspended) => Ok(next),
            (ResourceState::Active, ResourceState::Archived) => Ok(next),
            (ResourceState::Suspended, ResourceState::Active) => Ok(next),
            (ResourceState::Suspended, ResourceState::Archived) => Ok(next),
            (ResourceState::Archived, ResourceState::Retired) => Ok(next),
            _ => Err(ResourceError::InvalidTransition),
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, ResourceState::Retired)
    }
}

impl From<u32> for ResourceState {
    fn from(v: u32) -> Self {
        match v {
            0 => ResourceState::Created,
            1 => ResourceState::Active,
            2 => ResourceState::Suspended,
            3 => ResourceState::Archived,
            4 => ResourceState::Retired,
            _ => panic!("invalid resource state"),
        }
    }
}

// ===========================================================================
// Resource Info
// ===========================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceInfo {
    pub id: Symbol,
    pub state: ResourceState,
    pub created_at: u64,
    pub updated_at: u64,
    pub metadata: Option<Bytes>,
}

// ===========================================================================
// Storage Keys (for use by storage contracts)
// ===========================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Resource(Symbol),
    ResourceList,
}

// ===========================================================================
// Event Emission Helpers
// ===========================================================================

/// Emit a resource lifecycle transition event with the appropriate action symbol.
pub fn emit_resource_transition_event(
    e: &Env,
    from: ResourceState,
    to: ResourceState,
    resource_id: &Symbol,
    caller: &Address,
) {
    let action = action_symbol_for_transition(from, to);
    let event = events::ResourceLifecycleEvent {
        event_version: events::EVENT_VERSION,
        resource_id: resource_id.clone(),
        old_state: from as u32,
        new_state: to as u32,
        caller: caller.clone(),
        timestamp: e.ledger().timestamp(),
    };
    e.events().publish((events::PROTOCOL_RESOURCES, action), event);
}

/// Emit a resource created event.
pub fn emit_resource_created_event(e: &Env, resource_id: &Symbol, caller: &Address) {
    let event = events::ResourceCreatedEvent {
        event_version: events::EVENT_VERSION,
        resource_id: resource_id.clone(),
        caller: caller.clone(),
        timestamp: e.ledger().timestamp(),
    };
    e.events()
        .publish((events::PROTOCOL_RESOURCES, events::ACT_RSRC_CREATE), event);
}

/// Emit a resource retired event.
pub fn emit_resource_retired_event(e: &Env, resource_id: &Symbol, caller: &Address) {
    let event = events::ResourceRetiredEvent {
        event_version: events::EVENT_VERSION,
        resource_id: resource_id.clone(),
        caller: caller.clone(),
        timestamp: e.ledger().timestamp(),
    };
    e.events()
        .publish((events::PROTOCOL_RESOURCES, events::ACT_RSRC_RETIRE), event);
}

/// Return the correct action symbol for a given transition.
pub fn action_symbol_for_transition(from: ResourceState, to: ResourceState) -> Symbol {
    match (from, to) {
        (ResourceState::Created, ResourceState::Active) => events::ACT_RSRC_ACTIVATE,
        (ResourceState::Active, ResourceState::Suspended) => events::ACT_RSRC_SUSPEND,
        (ResourceState::Suspended, ResourceState::Active) => events::ACT_RSRC_RESUME,
        (ResourceState::Active, ResourceState::Archived) => events::ACT_RSRC_ARCHIVE,
        (ResourceState::Suspended, ResourceState::Archived) => events::ACT_RSRC_ARCHIVE,
        (ResourceState::Archived, ResourceState::Retired) => events::ACT_RSRC_RETIRE,
        _ => events::ACT_RSRC_ACTIVATE,
    }
}
