# Treasury Management Architecture

The treasury contract tracks a single protocol-owned asset, records incoming fees,
and distributes balances according to administrator-configured allocation
strategies.

## Storage model

- `Admin` stores the authorized treasury operator.
- `Asset` stores the protocol-owned token managed by the treasury.
- `Fee(fee_id)` stores immutable fee receipts keyed by unique identifiers.
- `TotalFeesRecorded` stores cumulative incoming protocol fees.
- `Strategy(strategy_id)` stores allocation rules whose basis points must total
  10,000.
- `Distribution(distribution_id)` stores immutable distribution receipts keyed by
  unique identifiers.
- `TotalDistributed` and `RecipientDistributed(address)` store cumulative
  outgoing accounting totals.

## Accounting methodology

Fee recording transfers tokens from a payer into the contract, stores a receipt,
and increments cumulative fee totals. Distribution first verifies the live token
balance, calculates rule-based recipient transfers, assigns rounding remainder to
the final rule, transfers funds, and records cumulative totals.

## Events

Treasury operations emit versioned Soroban events for initialization, strategy
configuration, fee recording, and distributions. Fee events include the payer,
asset, amount, post-fee treasury balance, and timestamp for auditability.
