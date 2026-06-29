# Protocol Resource Lifecycle Manager

## 📋 Summary
Standardized lifecycle management for protocol-managed resources — tracking creation, active use, archival, and retirement throughout each resource's operational lifetime.

## 📂 Relevant Paths
- `contracts/resources/src/lib.rs` — Resource state machine definitions, transition validation, storage, and event emission.
- `contracts/events/src/lib.rs` — Resource lifecycle event types and action symbols.
- `contracts/storage/src/lib.rs` — Persistent storage integration for protocol state machines.
- `contracts/core/src/lib.rs` — Resource lifecycle facade functions.
- `tests/resources/resource_lifecycle.test.ts` — TypeScript test suite verifying all valid/invalid transitions and event emissions.

---

## 🖼️ State Diagram

```
                    ┌──────────┐
                    │ Created  │
                    └────┬─────┘
                         │ (activate)
                         ▼
              ┌───────────────────┐
      ┌──────>│      Active      │<──────┐
      │       └──┬──────┬────────┘       │
      │          │      │                │
(suspend)        │      │ (archive)      │ (resume)
      │          │      │                │
      │     ┌────▼──┐   │                │
      └──────┤Suspended  │               │
             └────┬──┘                    │
                  │ (archive)             │
                  ▼                       │
             ┌──────────┐                │
             │ Archived  │────────────────┘
             └────┬─────┘
                  │ (retire)
                  ▼
             ┌──────────┐
             │ Retired  │  ← TERMINAL
             └──────────┘
```

---

## 🧮 Transition Matrix

| Current State | Target State | Valid? | Condition / Action |
| :--- | :--- | :---: | :--- |
| `Created` | `Active` | ✅ | Admin activates the resource |
| `Active` | `Suspended` | ✅ | Admin suspends resource operations |
| `Active` | `Archived` | ✅ | Admin archives an active resource |
| `Suspended` | `Active` | ✅ | Admin resumes a suspended resource |
| `Suspended` | `Archived` | ✅ | Admin archives a suspended resource |
| `Archived` | `Retired` | ✅ | Admin permanently decommissions resource |
| *Any* | *Self* | ❌ | Rejected with `ResourceError::AlreadyInState (2002)` |
| *Any* | *Other* | ❌ | Rejected with `ResourceError::InvalidTransition (2001)` |

---

## 🛡️ Validation Rules & Error Handling

1. **Identical State Protection**: Attempting to transition to the current state returns `ResourceError::AlreadyInState (2002)`.
2. **Path Verification**: Any transition not explicitly listed in the valid mapping returns `ResourceError::InvalidTransition (2001)`.
3. **Authorization**: Only the designated admin can create resources or initiate transitions. Unauthorized callers receive `ResourceError::Unauthorized (2003)`.
4. **Uniqueness**: Duplicate resource IDs are rejected with `ResourceError::AlreadyExists (2005)`.
5. **Existence Check**: Operations on non-existent resources return `ResourceError::NotFound (2004)`.
6. **Terminal State**: `Retired` is a terminal state — no further transitions are permitted.

---

## 📢 Events & Telemetry

Every lifecycle event emits a standardized Soroban event following the protocol's two-topic architecture: `(PROTOCOL_RESOURCES, ACTION_SYMBOL)`.

### Protocol Identifier
`PROTOCOL_RESOURCES = symbol_short!("AxRes")`

### Action Symbols
| Symbol | Value | Triggered By |
| :--- | :--- | :--- |
| `ACT_RSRC_CREATE` | `rsrc_new` | Resource creation |
| `ACT_RSRC_ACTIVATE` | `rsrc_act` | Created → Active |
| `ACT_RSRC_SUSPEND` | `rsrc_susp` | Active → Suspended |
| `ACT_RSRC_RESUME` | `rsrc_res` | Suspended → Active |
| `ACT_RSRC_ARCHIVE` | `rsrc_arch` | Active/Suspended → Archived |
| `ACT_RSRC_RETIRE` | `rsrc_ret` | Archived → Retired |

### Struct Payloads

```rust
pub struct ResourceLifecycleEvent {
    pub event_version: u32,    // Always 1 for current schema
    pub resource_id: Symbol,   // Unique resource identifier
    pub old_state: u32,        // Previous ResourceState enum value
    pub new_state: u32,        // New ResourceState enum value
    pub caller: Address,       // Address initiating the transition
    pub timestamp: u64,        // Ledger timestamp
}

pub struct ResourceCreatedEvent {
    pub event_version: u32,
    pub resource_id: Symbol,
    pub caller: Address,
    pub timestamp: u64,
}

pub struct ResourceRetiredEvent {
    pub event_version: u32,
    pub resource_id: Symbol,
    pub caller: Address,
    pub timestamp: u64,
}
```

---

## 🏗️ Architecture Decisions

1. **Dedicated Crate**: The resource lifecycle lives in `contracts/resources/` (`axionvera-resources`) as a standalone crate with no dependency on other protocol crates besides `axionvera-events`. This keeps it lightweight and reusable.

2. **Self-Contained Storage**: Each `ResourceLifecycle` instance manages its own storage via `DataKey::Resource(Symbol)` and `DataKey::ResourceList`, following the pattern established by `contracts/registry/src/storage.rs` and `contracts/vault-contract/src/storage.rs`.

3. **Admin-Guarded**: All mutations require admin authorization, consistent with existing protocol patterns (asset registry, config contract).

4. **Event Standardization**: Two-topic `(PROTOCOL, ACTION)` architecture consistent with all existing protocol contracts.

5. **Integration Facade**: `axionvera-core` provides a convenience facade (`transition_resource`, `current_resource_state`, etc.) matching the existing state machine integration pattern.

6. **Error Code Range**: Resource errors use codes 2001–2005 to avoid collision with protocol state machine errors (1001–1003).

---

## ✅ Testing Coverage

### Rust Unit Tests (`contracts/resources/src/test.rs`)
- `test_create_resource` — Basic creation flow
- `test_create_duplicate_rejected` — Uniqueness enforcement
- `test_create_unauthorized_rejected` — Access control
- `test_complete_lifecycle` — Full Created → Active → Suspended → Active → Archived → Retired flow
- `test_invalid_transitions_rejected` — All invalid transition paths
- `test_already_in_state_rejected` — Self-transition rejection
- `test_unauthorized_transition_rejected` — Authorization on transitions
- `test_get_state_and_info` — Query correctness
- `test_get_nonexistent_resource` — Not-found handling
- `test_list_resources` — Resource enumeration
- `test_exists` — Existence check
- `test_is_terminal` — Terminal state detection
- `test_with_metadata` — Optional metadata payload
- `test_pure_state_machine_transitions` — Direct enum transition validation

### TypeScript Tests (`tests/resources/resource_lifecycle.test.ts`)
- Resource creation with event emission
- Full lifecycle transitions
- Invalid transition rejection (6 cases)
- AlreadyInState detection
- Non-existent resource handling
- Event emission verification per transition
- Resource count tracking
- Documentation matrix verification
