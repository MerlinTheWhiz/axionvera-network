# Vault Contract Storage Guide

This document explains how the vault stores data on-chain and how those values change during deposits, withdrawals, and reward distribution.

## Why This Matters

Most contract changes touch storage indirectly. If you understand the storage keys and when they change, the rest of the contract becomes much easier to reason about.

## Storage Buckets

The contract uses two Soroban storage areas:

- instance storage for global contract configuration and totals
- persistent storage for per-user balances and reward snapshots

## Storage Keys

The keys are defined in [storage.rs](../contracts/vault-contract/src/storage.rs) as the `DataKey` enum.

### Global Keys

- `Initialized`
- `Admin`
- `DepositToken`
- `RewardToken`
- `TotalDeposits`
- `RewardIndex`

### Per-User Keys

- `UserLiquidBalance(Address)`
- `UserLocks(Address)`
- `UserRewardIndex(Address)`
- `UserRewards(Address)`

## What Each Key Means

### `Initialized`

Marks whether `initialize` has already run.

### `Admin`

The address authorized to call `distribute_rewards`.

### `DepositToken`

The token users deposit into the vault and withdraw later.

### `RewardToken`

The token used for reward distributions and reward claims.

### `TotalDeposits`

The total deposited amount across all users.

### `RewardIndex`

The cumulative rewards-per-share index, scaled by `1e18`.

### `UserLiquidBalance(Address)`

The portion of a user's deposit that is not locked and is available for immediate withdrawal.

### `UserLocks(Address)`

A list of time-locked fund portions for a user. Each lock has an amount and an unlock timestamp.

### `UserRewardIndex(Address)`

The last global `RewardIndex` the user has been synced to.

### `UserRewards(Address)`

Rewards already accrued for the user but not yet claimed.

## Missing Values Default To Zero

New users do not need explicit setup rows. Missing balances, user reward indices, and user reward totals are treated as `0`.

## How Storage Changes During Each Action

### During `initialize`

Writes:

- `Initialized = true`
- `Admin = admin`
- `DepositToken = deposit_token`
- `RewardToken = reward_token`
- `TotalDeposits = 0`
- `RewardIndex = 0`

### During `deposit`

Writes:

- `UserRewards(user)` may increase
- `UserRewardIndex(user)` is synced to the current global reward index
- `UserLiquidBalance(user)` increases by `amount`
- `TotalDeposits` increases by `amount`

### During `withdraw`

Writes:

- `UserRewards(user)` may increase
- `UserRewardIndex(user)` is synced to the current global reward index
- `UserLiquidBalance(user)` decreases by `amount` after checking for and processing any expired locks.
- `TotalDeposits` decreases by `amount`

### During `lock`

Writes:

- `UserLiquidBalance(user)` decreases by `amount`
- `UserLocks(user)` is updated with a new lock entry.
- `TotalDeposits` remains unchanged.

### During `unlock_expired`

Writes:

- `UserLocks(user)` is modified to remove expired lock entries.
- `UserLiquidBalance(user)` increases by the total amount of the expired locks.

### During `distribute_rewards`

Writes:

- `RewardIndex` increases

Important detail:

- user-specific balances are not updated here
- user rewards are realized lazily on later interactions

### During `claim_rewards`

Writes:

- `UserRewards(user)` is reset to `0` after payout
- `UserRewardIndex(user)` remains synced with the global index

## Reward Accrual Walkthrough

When a user interacts, reward accrual roughly follows this logic:

1. Read global `RewardIndex`.
2. Read that user's `UserRewardIndex`.
3. Compute `delta = global - user`.
4. Multiply `delta` by the user's total balance (liquid + locked).
5. Divide by `1e18`.
6. Add the result to `UserRewards(user)`.
7. Set `UserRewardIndex(user)` to the global value.

## Storage Invariants

- `TotalDeposits` should match the sum of all active user balances.
- A user cannot withdraw more than their available `UserLiquidBalance(user)`.
- `RewardIndex` should never decrease.
- `UserRewards(user)` should only decrease when rewards are claimed.
- The contract should never require iterating through all users for reward distribution.

## Where To Read Next

- [contract-spec.md](contract-spec.md)
- [lib.rs](../contracts/vault-contract/src/lib.rs)
- [storage.rs](../contracts/vault-contract/src/storage.rs)
