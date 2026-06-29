use axionvera_auth::{AccessPolicy, PolicyViolation};
use axionvera_security::{Authenticated, MatchAddress, PredicatePolicy};
use soroban_sdk::Address;

use crate::errors::{AuthorizationError, VaultError};

#[derive(Clone)]
struct ActorContext {
    actor: Address,
}

#[derive(Clone)]
struct AdminContext {
    actor: Address,
    admin: Address,
}

#[derive(Clone)]
struct PendingAdminContext {
    actor: Address,
    pending_admin: Option<Address>,
}

fn actor_address(context: &ActorContext) -> Address {
    context.actor.clone()
}

fn admin_actor(context: &AdminContext) -> Address {
    context.actor.clone()
}

fn admin_address(context: &AdminContext) -> Address {
    context.admin.clone()
}

fn pending_admin_actor(context: &PendingAdminContext) -> Address {
    context.actor.clone()
}

fn pending_admin_matches(context: &PendingAdminContext) -> bool {
    matches!(
        context.pending_admin.as_ref(),
        Some(pending_admin) if pending_admin == &context.actor
    )
}

pub fn require_actor(actor: &Address) -> Result<(), VaultError> {
    let context = ActorContext {
        actor: actor.clone(),
    };

    Authenticated::new(actor_address)
        .enforce(&context)
        .map_err(map_violation)
}

pub fn require_admin(actor: &Address, admin: &Address) -> Result<(), VaultError> {
    let context = AdminContext {
        actor: actor.clone(),
        admin: admin.clone(),
    };

    Authenticated::new(admin_actor)
        .and(MatchAddress::new(
            admin_actor,
            admin_address,
            PolicyViolation::AddressMismatch,
        ))
        .enforce(&context)
        .map_err(map_violation)
}

pub fn require_stored_admin(admin: &Address) -> Result<(), VaultError> {
    require_admin(admin, admin)
}

pub fn require_pending_admin(
    actor: &Address,
    pending_admin: Option<Address>,
) -> Result<(), VaultError> {
    let context = PendingAdminContext {
        actor: actor.clone(),
        pending_admin,
    };

    Authenticated::new(pending_admin_actor)
        .and(PredicatePolicy::new(
            pending_admin_matches,
            PolicyViolation::Unauthorized,
        ))
        .enforce(&context)
        .map_err(map_violation)
}

fn map_violation(_: PolicyViolation) -> VaultError {
    AuthorizationError::Unauthorized.into()
}
