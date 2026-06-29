# Multi-Asset Vault Implementation

## Overview

The vault contract has been extended to support multiple Stellar assets while maintaining backwards compatibility with the existing single-asset functionality.

## Changes Made

### 1. Storage Layer (`storage.rs`)

#### New Data Keys

- `SupportedAssets`: Map of supported asset addresses
- `AssetTotalDeposits(Address)`: Total deposits per asset
- `AssetRewardIndex(Address)`: Global reward index per asset
- `UserAssetBalance(Address, Address)`: User balance per asset (user, asset)
- `UserAssetRewardIndex(Address, Address)`: User's last synced reward index per asset
- `UserAssetAccruedRewards(Address, Address)`: User's accrued but unvested rewards per asset
- `UserAssetLastRewardTimestamp(Address, Address)`: User's last reward timestamp per asset

#### New Functions

- `add_supported_asset()`: Add a new asset to the vault
- `get_supported_assets()`: Get all supported assets
- `is_asset_supported()`: Check if an asset is supported
- `get_asset_total_deposits()`: Get total deposits for a specific asset
- `set_asset_total_deposits()`: Set total deposits for a specific asset
- `get_asset_reward_index()`: Get reward index for a specific asset
- `set_asset_reward_index()`: Set reward index for a specific asset
- `get_user_asset_position()`: Get user position for a specific asset
- `set_user_asset_position()`: Set user position for a specific asset
- `get_user_asset_balance()`: Get user balance for a specific asset
- `store_asset_deposit()`: Process deposit for a specific asset
- `store_asset_withdraw()`: Process withdrawal for a specific asset
- `store_asset_reward_distribution()`: Process reward distribution for a specific asset
- `store_asset_claimable_rewards()`: Process reward claiming for a specific asset
- `preview_user_asset_rewards()`: Preview rewards without state modification
- `pending_user_asset_rewards_view()`: View pending rewards for an asset
- `vested_user_asset_rewards_view()`: View vested rewards for an asset
- `accrue_asset_position_rewards()`: Accrue rewards for a specific asset position

### 2. Contract Interface (`lib.rs`)

#### New Public Functions

- `add_asset(admin, asset)`: Add a new supported asset (admin only)
- `deposit_asset(from, asset, amount)`: Deposit a specific asset
- `withdraw_asset(to, asset, amount)`: Withdraw a specific asset
- `distribute_rewards_for_asset(admin, asset, amount)`: Distribute rewards for a specific asset
- `claim_rewards_for_asset(user, asset)`: Claim rewards for a specific asset
- `balance_of_asset(user, asset)`: Get user balance for a specific asset
- `total_deposits_of_asset(asset)`: Get total deposits for a specific asset
- `reward_index_of_asset(asset)`: Get reward index for a specific asset
- `pending_rewards_for_asset(user, asset)`: Get pending rewards for a specific asset
- `vested_rewards_for_asset(user, asset)`: Get vested rewards for a specific asset
- `is_asset_supported(asset)`: Check if an asset is supported

### 3. Events (`events.rs`)

#### New Event Types

- `AssetAddedEvent`: Emitted when a new asset is added
- `AssetDepositEvent`: Emitted when a user deposits an asset
- `AssetWithdrawEvent`: Emitted when a user withdraws an asset
- `AssetDistributeEvent`: Emitted when rewards are distributed for an asset
- `AssetClaimEvent`: Emitted when rewards are claimed for an asset

#### New Event Functions

- `emit_asset_added()`: Emit asset added event
- `emit_asset_deposit()`: Emit asset deposit event
- `emit_asset_withdraw()`: Emit asset withdraw event
- `emit_asset_distribute()`: Emit asset reward distribution event
- `emit_asset_claim_rewards()`: Emit asset reward claim event

### 4. Tests (`test.rs`)

#### New Test Cases

- `test_add_asset()`: Verify asset addition
- `test_multiple_asset_deposits()`: Test depositing multiple assets
- `test_multiple_asset_withdrawals()`: Test withdrawing from multiple assets
- `test_asset_reward_distribution()`: Test reward distribution for specific assets
- `test_asset_reward_claiming()`: Test claiming rewards for specific assets
- `test_independent_asset_tracking()`: Verify independent balance tracking per asset
- `test_unsupported_asset_fails()`: Verify operations on unsupported assets fail

## Key Features

### Independent Tracking

Each asset has its own:

- Total deposits counter
- Reward index
- User balances
- User reward accrual state

### Backwards Compatibility

The original single-asset functions remain unchanged:

- `initialize()`
- `deposit()`
- `withdraw()`
- `distribute_rewards()`
- `claim_rewards()`
- `balance()`
- `total_deposits()`
- etc.

### Security

- Only admin can add new assets
- All existing security features (reentrancy guards, pause mechanism, etc.) apply to multi-asset operations
- Asset validation ensures operations only proceed on supported assets

### Reward Mechanics

- Each asset maintains its own reward index for proportional distribution
- Vesting periods apply per-asset
- Users can claim rewards independently for each asset
- Reward calculations are isolated per asset to prevent cross-contamination

## Usage Examples

### Adding a New Asset

```rust
// Admin adds USDC as a supported asset
vault.add_asset(admin, usdc_address);
```

### Depositing Multiple Assets

```rust
// User deposits 100 USDC
vault.deposit_asset(user, usdc_address, 100);

// User deposits 200 XLM
vault.deposit_asset(user, xlm_address, 200);
```

### Distributing Rewards Per Asset

```rust
// Admin distributes 1000 reward tokens to USDC stakers
vault.distribute_rewards_for_asset(admin, usdc_address, 1000);

// Admin distributes 2000 reward tokens to XLM stakers
vault.distribute_rewards_for_asset(admin, xlm_address, 2000);
```

### Claiming Rewards Per Asset

```rust
// User claims USDC staking rewards
let usdc_rewards = vault.claim_rewards_for_asset(user, usdc_address);

// User claims XLM staking rewards
let xlm_rewards = vault.claim_rewards_for_asset(user, xlm_address);
```

### Checking Balances

```rust
// Check user's USDC balance
let usdc_balance = vault.balance_of_asset(user, usdc_address);

// Check user's XLM balance
let xlm_balance = vault.balance_of_asset(user, xlm_address);
```

## Migration Path

Existing deployments can:

1. Continue using the original single-asset functions
2. Gradually migrate to multi-asset by:
   - Adding the original deposit token as a supported asset
   - Directing new deposits to use `deposit_asset()` instead of `deposit()`
   - Transitioning reward distribution to `distribute_rewards_for_asset()`

## Testing

Comprehensive test suite covers:

- ✅ Adding assets
- ✅ Depositing multiple asset types
- ✅ Withdrawing from multiple assets
- ✅ Independent balance tracking
- ✅ Reward distribution per asset
- ✅ Reward claiming per asset
- ✅ Proportional reward calculation
- ✅ Vesting mechanics per asset
- ✅ Unsupported asset rejection

## Acceptance Criteria Status

- ✅ **Users can deposit multiple asset types**: Implemented via `deposit_asset()`
- ✅ **Balances are tracked independently**: Each asset has separate storage keys for balances and state
- ✅ **Tests cover multi-asset scenarios**: 7 new test cases added covering all multi-asset operations
- ✅ **Backwards compatibility maintained**: All original functions remain unchanged and operational

## Unified Protocol Interface and Accounting Design

The multi-asset vault keeps the original protocol surface intact while adding asset-scoped entrypoints for callers that need explicit asset selection. Legacy single-asset calls continue to read `DepositToken`, `TotalDeposits`, `RewardIndex`, and user-level reward snapshots. New multi-asset calls use the same accounting lifecycle against asset-scoped storage keys:

1. Validate that the asset is registered in `SupportedAssets`.
2. Accrue rewards for the user's existing asset balance before any balance mutation.
3. Transfer the Stellar asset through the standard token interface.
4. Update `AssetTotalDeposits(asset)`, `AssetRewardIndex(asset)`, and the user's asset-scoped position.
5. Emit asset-specific events and record accounting entries with the affected asset address.

This design keeps reward math consistent across all assets: each asset has an independent index, but every index uses the same precision factor, vesting calculation, checked arithmetic, and no-deposits/zero-increment validation as the legacy vault path.

## Migration Considerations

Existing deployments can remain on legacy calls without a storage migration. To adopt multi-asset flows, register the legacy deposit token with `add_asset`, then direct new integrations to `deposit_asset`, `withdraw_asset`, `distribute_rewards_for_asset`, and `claim_rewards_for_asset`. Historical legacy balances remain in the legacy storage namespace unless a future migration explicitly moves them into `UserAssetBalance` and `AssetTotalDeposits` for the registered legacy asset.
