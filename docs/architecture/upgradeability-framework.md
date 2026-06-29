# Upgradeability Framework

Axionvera uses Soroban's in-place WASM upgrade mechanism for protocol evolution. The framework is intentionally conservative: a new implementation may be installed only after compatibility checks prove that existing state remains readable, authorization remains admin-bound, and post-upgrade verification can be performed without migrating users to a new contract address.

## Upgrade model comparison

| Model | State preservation | Operational risk | Fit for Axionvera |
| --- | --- | --- | --- |
| Immutable redeploy | Requires user and integration migration | High migration and liquidity fragmentation risk | Not preferred for core vault upgrades |
| Proxy-style dispatcher | Preserves state behind stable address | Adds dispatch surface and storage coupling | Useful in EVM, less idiomatic for Soroban |
| Soroban WASM swap | Preserves instance and persistent storage at the same contract ID | Requires strict storage-key compatibility | Preferred model |

The selected model is **Soroban WASM swap** through `Env::deployer().update_current_contract_wasm`. This keeps the contract ID and all storage entries in place while replacing executable code.

## Authorization rules

Every upgrade must enforce all of the following rules:

1. The contract is initialized before upgrade execution.
2. The stored admin address is read from instance storage.
3. The stored admin must authorize the transaction with `require_auth`.
4. The implementation must compare the authenticated signer to the stored admin; caller-supplied admin values are not trusted.
5. An upgrade event must be emitted with the admin and target WASM hash for off-chain monitoring.

The `axionvera-upgrades` crate models these gates as mandatory authorization checks so missing controls block compatibility approval.

## Storage compatibility policy

Storage compatibility is evaluated at the key level before any production upgrade:

- Removing a required key is **breaking** because existing state may become unreadable.
- Moving a key between instance and persistent storage is **breaking**.
- Changing a stored value type is **breaking** unless an explicit adapter migration is shipped and tested.
- Adding a key is a **warning** and is allowed only when the new implementation lazily initializes it or includes a bounded migration step.
- Optional legacy keys may be deprecated, but the new implementation must continue ignoring or clearing them safely.

The current vault storage keys are append-only. New variants should be added after existing `DataKey` variants to avoid invalidating serialized keys.

## Migration workflow

The standard workflow is:

1. **Proposed**: publish the target WASM hash, compatibility report, and runbook.
2. **Validated**: run storage, event, interface, and authorization compatibility checks.
3. **Authorized**: require stored-admin authorization for the upgrade transaction.
4. **Executed**: call `update_current_contract_wasm` on the existing contract ID.
5. **Verified**: query critical state such as admin, total deposits, balances, reward index, and version after execution.

Most upgrades should be lazy migrations: new fields are populated on first use or by bounded admin calls. Bulk rewrites of user state should be avoided because they increase budget risk and make rollback harder.

## Required tests

Upgrade pull requests must include tests for:

- Required storage keys remain present with the same scope and type.
- Additive storage keys produce warnings, not blockers.
- Removed keys, type changes, or scope changes block approval.
- Upgrade authorization fails without both runtime auth and stored-admin matching.
- State snapshots before and after upgrade preserve balances, totals, reward indices, and admin.
- The upgrade event is emitted so indexers can track implementation changes.

## Security review notes

Primary risks and mitigations:

- **State corruption**: mitigate with append-only storage layout and compatibility tests.
- **Unauthorized upgrade**: mitigate with stored-admin authorization, two-step admin transfer, and event monitoring.
- **Unbounded migration**: mitigate with lazy initialization and budgeted, idempotent migration steps.
- **Indexer breakage**: mitigate by keeping existing events stable and treating removed events as breaking.
- **Bad WASM hash**: mitigate with reproducible builds, testnet rehearsal, and post-upgrade verification queries.
