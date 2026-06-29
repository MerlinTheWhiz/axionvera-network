# Protocol Contract Security Hardening Review

## Scope

Reviewed protocol contract surfaces under `contracts/`, with focused remediation in shared storage/state helpers and the vault contract authorization, delegation, parameter validation, and accounting paths.

## Findings and Mitigations

| Area                              | Risk                                                                                                                             | Mitigation                                                                                                                                                                                                    |
| --------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Shared storage manifest           | Workspace builds could not parse `contracts/storage`, preventing automated regression checks for protocol crates.                | Added an explicit `axionvera-storage` package manifest so storage/state helpers are buildable by dependent contracts.                                                                                         |
| State transition storage          | Protocol state helpers needed a single storage facade with transition validation and admin gating.                               | Added typed storage keys, guarded admin initialization, transition validation through `axionvera-state`, TTL extension, and transition event emission.                                                        |
| Vault reward/deposit accounting   | Deposit accounting referenced the updated state without binding it, and withdrawal transfer used an unbound token variable.      | Bound state returned by storage mutations and cloned the deposit token before transfer.                                                                                                                       |
| Delegation storage                | Legacy delegation helpers referenced removed data structures and did not enforce the richer delegation constraints consistently. | Routed compatibility helpers through canonical persistent `Delegation` records, enforced no self-delegation, non-zero known permission masks, max delegation limits, expiration checks, and canonical errors. |
| Utilization multiplier validation | Reward multiplier parameters accepted out-of-range utilization or zero/excessive multiplier values.                              | Added bounds checks for utilization basis points and multiplier basis points while preserving sorted-curve validation.                                                                                        |

## Residual Risk and Recommendations

- Run the full workspace test suite once the remaining pre-existing vault test module syntax issues are reconciled.
- Add time-bounded delegate authorization to the older `authorize_delegate` public API if external callers still depend on it; newer delegation APIs already support expirations.
- Consider requiring explicit admin initialization for the shared storage facade in deployment scripts, because the facade currently permits ungated transitions until an admin is configured to preserve backwards compatibility.
- Schedule an external audit before production deployment; this review is an internal hardening pass only.
