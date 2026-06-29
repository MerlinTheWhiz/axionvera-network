#![cfg(test)]

use crate::{ResourceError, ResourceState};

#[test]
fn test_pure_state_machine_transitions() {
    // Valid transitions
    assert_eq!(
        ResourceState::Created.transition(ResourceState::Active),
        Ok(ResourceState::Active)
    );
    assert_eq!(
        ResourceState::Active.transition(ResourceState::Suspended),
        Ok(ResourceState::Suspended)
    );
    assert_eq!(
        ResourceState::Active.transition(ResourceState::Archived),
        Ok(ResourceState::Archived)
    );
    assert_eq!(
        ResourceState::Suspended.transition(ResourceState::Active),
        Ok(ResourceState::Active)
    );
    assert_eq!(
        ResourceState::Suspended.transition(ResourceState::Archived),
        Ok(ResourceState::Archived)
    );
    assert_eq!(
        ResourceState::Archived.transition(ResourceState::Retired),
        Ok(ResourceState::Retired)
    );

    // Invalid transitions
    assert_eq!(
        ResourceState::Created.transition(ResourceState::Archived),
        Err(ResourceError::InvalidTransition)
    );
    assert_eq!(
        ResourceState::Created.transition(ResourceState::Retired),
        Err(ResourceError::InvalidTransition)
    );
    assert_eq!(
        ResourceState::Active.transition(ResourceState::Retired),
        Err(ResourceError::InvalidTransition)
    );
    assert_eq!(
        ResourceState::Suspended.transition(ResourceState::Retired),
        Err(ResourceError::InvalidTransition)
    );
    assert_eq!(
        ResourceState::Archived.transition(ResourceState::Active),
        Err(ResourceError::InvalidTransition)
    );
    assert_eq!(
        ResourceState::Archived.transition(ResourceState::Created),
        Err(ResourceError::InvalidTransition)
    );
    assert_eq!(
        ResourceState::Retired.transition(ResourceState::Active),
        Err(ResourceError::InvalidTransition)
    );
    assert_eq!(
        ResourceState::Retired.transition(ResourceState::Archived),
        Err(ResourceError::InvalidTransition)
    );

    // Self transitions
    assert_eq!(
        ResourceState::Created.transition(ResourceState::Created),
        Err(ResourceError::AlreadyInState)
    );
    assert_eq!(
        ResourceState::Active.transition(ResourceState::Active),
        Err(ResourceError::AlreadyInState)
    );
    assert_eq!(
        ResourceState::Retired.transition(ResourceState::Retired),
        Err(ResourceError::AlreadyInState)
    );
}

#[test]
fn test_transition_chain() {
    // Full lifecycle chain: Created -> Active -> Suspended -> Active -> Archived -> Retired
    let mut state = ResourceState::Created;

    state = state.transition(ResourceState::Active).unwrap();
    assert_eq!(state, ResourceState::Active);

    state = state.transition(ResourceState::Suspended).unwrap();
    assert_eq!(state, ResourceState::Suspended);

    state = state.transition(ResourceState::Active).unwrap();
    assert_eq!(state, ResourceState::Active);

    state = state.transition(ResourceState::Archived).unwrap();
    assert_eq!(state, ResourceState::Archived);

    state = state.transition(ResourceState::Retired).unwrap();
    assert_eq!(state, ResourceState::Retired);
}

#[test]
fn test_alternative_path_suspended_to_archived() {
    let mut state = ResourceState::Created;
    state = state.transition(ResourceState::Active).unwrap();
    state = state.transition(ResourceState::Suspended).unwrap();
    state = state.transition(ResourceState::Archived).unwrap();
    assert_eq!(state, ResourceState::Archived);
}

#[test]
fn test_is_terminal() {
    assert!(!ResourceState::Created.is_terminal());
    assert!(!ResourceState::Active.is_terminal());
    assert!(!ResourceState::Suspended.is_terminal());
    assert!(!ResourceState::Archived.is_terminal());
    assert!(ResourceState::Retired.is_terminal());
}

#[test]
fn test_from_u32() {
    assert_eq!(ResourceState::from(0u32), ResourceState::Created);
    assert_eq!(ResourceState::from(1u32), ResourceState::Active);
    assert_eq!(ResourceState::from(2u32), ResourceState::Suspended);
    assert_eq!(ResourceState::from(3u32), ResourceState::Archived);
    assert_eq!(ResourceState::from(4u32), ResourceState::Retired);
}

#[test]
#[should_panic(expected = "invalid resource state")]
fn test_from_u32_invalid_panics() {
    let _ = ResourceState::from(99u32);
}
