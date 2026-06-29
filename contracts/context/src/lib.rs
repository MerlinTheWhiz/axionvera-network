#![no_std]

use soroban_sdk::{contracterror, contracttype, symbol_short, Address, BytesN, Env, Symbol};

const MAX_DEPTH: u32 = 8;

pub const PROTOCOL: Symbol = symbol_short!("AxCtx");
pub const ACT_CTX_CREATED: Symbol = symbol_short!("ctx_creat");
pub const ACT_CTX_PUSHED: Symbol = symbol_short!("ctx_push");
pub const ACT_CTX_VALID: Symbol = symbol_short!("ctx_valid");

/// Standard execution context propagated across contract calls.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionContext {
    pub original_caller: Address,
    pub current_caller: Address,
    pub protocol_version: u32,
    pub depth: u32,
    pub timestamp: u64,
    pub ledger_sequence: u32,
    pub operation_id: Option<u32>,
    pub plan_id: Option<BytesN<32>>,
}

impl ExecutionContext {
    pub fn new(e: &Env, caller: Address) -> Self {
        Self {
            original_caller: caller.clone(),
            current_caller: caller,
            protocol_version: 1,
            depth: 0,
            timestamp: e.ledger().timestamp(),
            ledger_sequence: e.ledger().sequence(),
            operation_id: None,
            plan_id: None,
        }
    }

    pub fn push(&self, e: &Env, caller: Address) -> Result<Self, ContextError> {
        if self.depth >= MAX_DEPTH {
            return Err(ContextError::MaxDepthExceeded);
        }
        Ok(Self {
            original_caller: self.original_caller.clone(),
            current_caller: caller,
            protocol_version: self.protocol_version,
            depth: self.depth + 1,
            timestamp: e.ledger().timestamp(),
            ledger_sequence: e.ledger().sequence(),
            operation_id: self.operation_id,
            plan_id: self.plan_id.clone(),
        })
    }

    pub fn with_operation(mut self, operation_id: u32, plan_id: BytesN<32>) -> Self {
        self.operation_id = Some(operation_id);
        self.plan_id = Some(plan_id);
        self
    }

    pub fn validate(&self, e: &Env) -> Result<(), ContextError> {
        let ts = e.ledger().timestamp();
        let seq = e.ledger().sequence();
        if self.timestamp > ts {
            return Err(ContextError::TimestampInconsistency);
        }
        if self.ledger_sequence > seq {
            return Err(ContextError::LedgerInconsistency);
        }
        Ok(())
    }
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContextError {
    MaxDepthExceeded = 1,
    TimestampInconsistency = 2,
    LedgerInconsistency = 3,
}

pub fn create_context(e: &Env, caller: Address) -> ExecutionContext {
    let ctx = ExecutionContext::new(e, caller);
    emit_context_created(e, &ctx);
    ctx
}

pub fn push_context(e: &Env, parent: &ExecutionContext, caller: Address) -> Result<ExecutionContext, ContextError> {
    let ctx = parent.push(e, caller)?;
    emit_context_pushed(e, &ctx, parent.depth);
    Ok(ctx)
}

pub fn emit_context_created(e: &Env, ctx: &ExecutionContext) {
    e.events().publish(
        (PROTOCOL, ACT_CTX_CREATED),
        ContextCreatedEvent {
            original_caller: ctx.original_caller.clone(),
            current_caller: ctx.current_caller.clone(),
            protocol_version: ctx.protocol_version,
            depth: ctx.depth,
            timestamp: ctx.timestamp,
            ledger_sequence: ctx.ledger_sequence,
        },
    );
}

pub fn emit_context_pushed(e: &Env, ctx: &ExecutionContext, parent_depth: u32) {
    e.events().publish(
        (PROTOCOL, ACT_CTX_PUSHED),
        ContextPushedEvent {
            original_caller: ctx.original_caller.clone(),
            current_caller: ctx.current_caller.clone(),
            depth: ctx.depth,
            parent_depth,
            timestamp: ctx.timestamp,
            ledger_sequence: ctx.ledger_sequence,
        },
    );
}

pub fn emit_context_validated(e: &Env, ctx: &ExecutionContext) {
    e.events().publish(
        (PROTOCOL, ACT_CTX_VALID),
        ContextValidatedEvent {
            original_caller: ctx.original_caller.clone(),
            depth: ctx.depth,
            timestamp: ctx.timestamp,
            ledger_sequence: ctx.ledger_sequence,
        },
    );
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextCreatedEvent {
    pub original_caller: Address,
    pub current_caller: Address,
    pub protocol_version: u32,
    pub depth: u32,
    pub timestamp: u64,
    pub ledger_sequence: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextPushedEvent {
    pub original_caller: Address,
    pub current_caller: Address,
    pub depth: u32,
    pub parent_depth: u32,
    pub timestamp: u64,
    pub ledger_sequence: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextValidatedEvent {
    pub original_caller: Address,
    pub depth: u32,
    pub timestamp: u64,
    pub ledger_sequence: u32,
}

#[cfg(test)]
mod test;
