# Delegation Framework

## Summary

Implements a delegation framework that allows vault owners to authorize trusted operators to perform specific management actions on their behalf, without transferring ownership.

## Permission Model

Permissions are defined as a bitmask with the following granular actions:

| Permission     | Bit   | Value | Description                      |
|----------------|-------|-------|----------------------------------|
| `DEPOSIT`      | 1 << 0 | 1    | Deposit on behalf of delegator   |
| `WITHDRAW`     | 1 << 1 | 2    | Withdraw from delegator's balance |
| `LOCK`         | 1 << 2 | 4    | Lock delegator's funds           |
| `UNLOCK`       | 1 << 3 | 8    | Unlock delegator's expired locks |
| `CLAIM`        | 1 << 4 | 16   | Claim rewards on delegator's behalf |

Operators can be granted a combination of permissions via a single `delegate()` call. `PERMISSION_ALL_USER` (31) covers all user-level actions.

## Security Considerations

- **Self-delegation is forbidden**: `CannotDelegateToSelf` error prevents redundant delegations.
- **Expiration support**: Delegations can have a `expires_at` timestamp (0 = never expires). Expired delegations are automatically rejected.
- **Max delegation limit**: Configurable per-vault limit (default 20) prevents operator list bloat.
- **No admin-action delegation**: Sensitive operations (distribute, pause, upgrade, addAsset, setPenaltyRate) remain strictly admin-only.
- **Authorization isolation**: Each delegated action independently validates the operator's permission bit before execution.
- **Revocation**: Delegations can be revoked at any time by the delegator, immediately invalidating all future actions.

## Delegation Lifecycle

1. **Create** — `delegate(delegator, operator, permissions, expires_at)`
   - Requires delegator auth
   - Validates operator ≠ delegator and expiration is not in the past
   - Checks max delegation limit
   - Emits `DelegateEvent`

2. **Use** — Delegated action functions (`delegated_deposit`, `delegated_withdraw`, `delegated_lock`, `delegated_unlock_expired`, `delegated_claim_rewards`)
   - Requires operator auth (not delegator auth)
   - Validates delegation exists, is not expired, and has the required permission bit
   - Delegated funds flow: deposit tokens come from the operator, but the position is credited to the delegator; for withdrawals and claims, tokens are sent to the operator
   - Emits `DelegatedActionEvent` before the standard action event

3. **Revoke** — `revoke_delegation(delegator, operator)`
   - Requires delegator auth
   - Removes delegation entry and cleans up operator list
   - Emits `RevokeDelegationEvent`

4. **Query** — `get_delegation(delegator, operator)` and `get_delegations(delegator)`
   - Read-only view functions

## Files Changed

### `contracts/vault-contract/`
- **`src/lib.rs`** — Added delegation management functions (`delegate`, `revoke_delegation`, `set_max_delegations`, `get_delegation`, `get_delegations`) and delegated action functions (`delegated_deposit`, `delegated_withdraw`, `delegated_lock`, `delegated_unlock_expired`, `delegated_claim_rewards`)
- **`src/storage.rs`** — Added `DataKey` variants for delegation storage, `Delegation` struct, permission constants, and helper functions (`authorize_for_user`, `check_delegation_permission`)
- **`src/errors.rs`** — Added `DelegationError` enum (NotFound, Expired, InsufficientPermissions, MaxDelegationsExceeded, CannotDelegateToSelf, InvalidExpiration) and new `VaultError` variants (25-30)
- **`src/events.rs`** — Added event emission helpers for delegate, revoke delegation, and delegated action events
- **`src/test.rs`** — Added comprehensive test suite covering delegation lifecycle, permission enforcement, revocation, expiration, and edge cases

### `contracts/events/`
- **`src/lib.rs`** — Added action symbols (`ACT_DELEGATE`, `ACT_REVOKE_DELEGATION`, `ACT_DELEGATED_ACTION`) and event structs (`DelegateEvent`, `RevokeDelegationEvent`, `DelegatedActionEvent`)

## Testing Summary

| Test Case | Description |
|-----------|------------|
| `test_create_delegation` | Verify delegation can be created and stored correctly |
| `test_revoke_delegation` | Verify delegation can be revoked and is removed from storage |
| `test_cannot_delegate_to_self` | Verify self-delegation is rejected |
| `test_expired_delegation_rejected` | Verify past expiration is rejected at creation time |
| `test_delegated_action_requires_correct_permission` | Verify operator with wrong permission cannot act |
| `test_delegated_deposit` | Verify operator can deposit on delegator's behalf |
| `test_delegated_withdraw` | Verify operator can withdraw from delegator's balance |
| `test_delegated_claim_rewards` | Verify operator can claim rewards for delegator |
| `test_list_delegations` | Verify all delegations are returned correctly |
| `test_delegation_events` | Verify delegate and revocation events are emitted |
| `test_unauthorized_operator_rejected` | Verify operator without delegation is rejected |

## Out of Scope

- Governance delegation
- Frontend delegation management UI
- Third-party integrations (e.g., Gnosis Safe, multisig)
