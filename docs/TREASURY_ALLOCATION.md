# Treasury Allocation Strategy Engine

The treasury contract distributes protocol-owned assets from the treasury
contract address according to governance-configured allocation strategies.

## Allocation Model

An `AllocationStrategy` is a deterministic list of `AllocationRule` entries.
Each rule contains a recipient address and a share in basis points. The sum of
all rule shares must equal `10000`, where `10000` is 100%.

When a distribution executes, the contract:

1. Verifies the caller is the configured treasury admin.
2. Loads and validates the selected strategy.
3. Checks the treasury token balance.
4. Calculates each recipient transfer from the requested amount.
5. Assigns any integer division remainder to the final rule.
6. Transfers assets and stores a `TreasuryDistributionReceipt`.

This keeps allocation deterministic and auditable while avoiding stranded
rounding dust.

## Validation

Strategies are rejected when they are empty, exceed 16 rules, contain duplicate
recipients, include a zero share, or do not sum to 100%. Distributions are
rejected when the amount is non-positive, the strategy does not exist, the
distribution id has already been used, or the treasury does not hold enough of
the configured asset.

## Storage And Events

The contract stores the admin, managed asset, configured strategies,
distribution receipts, total distributed amount, and cumulative amount
distributed per recipient.

Treasury events use the existing two-topic AxionVera event convention:

- `(AxVault, tr_init)` when the treasury is initialized;
- `(AxVault, tr_strat)` when a strategy is configured;
- `(AxVault, tr_dist)` when a distribution completes.
