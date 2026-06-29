# Protocol Integration Test Harness

## Overview

The Protocol Integration Test Harness validates interactions between all major protocol contracts under realistic end-to-end scenarios. It complements existing unit and integration tests by verifying that contracts work correctly together.

## Architecture

```
tests/integration/
  fixtures/
    protocol.ts           Reusable test fixtures, configurations, and utilities
  protocol/
    cross-contract-vault.test.ts    Vault ↔ Reward ↔ Staking flows
    multi-asset.test.ts             Multi-asset deposit/withdraw/reward workflows
    governance-parameter.test.ts    Chain parameter governance & upgrades
  scenarios/
    failure.test.ts                 Failure scenarios & edge cases
  PROTOCOL_INTEGRATION.md           This document
```

## Covered Workflows

### 1. Cross-Contract Vault Interactions

- **Vault ↔ Reward Contract**: Deposit tokens, distribute rewards, claim rewards, verify state propagation
- **Vault ↔ Staking Contract**: Lock tokens for staking, earn rewards on staked positions, verify proportional reward allocation
- **Vault ↔ Governance Contract**: Query chain parameters, submit parameter upgrades, list pending upgrades
- **Complete Lifecycle**: Deposit → Stake → Reward Distribution → Claim → Withdraw across contracts

### 2. Multi-Asset Protocol Workflows

- **Cross-Asset Deposits**: Deposit multiple token types (USDC, AXN, ETH) for a single user
- **Independent Balance Tracking**: Verify each asset balance is tracked independently
- **Per-Asset Withdrawals**: Withdraw assets without affecting other asset balances
- **Multi-User Multi-Asset**: Concurrent deposits by multiple users across different assets
- **Asset-Specific Reward Distribution**: Proportional rewards per asset pool

### 3. Governance & Parameter Management

- **Chain Parameter Queries**: Retrieve active and genesis parameters
- **Parameter Upgrades**: Submit upgrades with partial or full parameter patches
- **DAO Multi-Sig Governance**: Parameter upgrades with DAO voter approval
- **Transaction History**: Query and filter transactions by type (deposit, withdraw)
- **Network Status**: Health checks, node info, peer list validation

### 4. Failure Scenarios

- **Authorization**: Invalid signatures, mismatched users, unauthorized admin operations
- **Validation**: Zero/negative amounts, non-numeric values, excessive withdrawals
- **Replay Protection**: Duplicate nonce rejection, sequential nonce acceptance
- **Address Validation**: Empty/malformed user and token addresses
- **Edge Cases**: Missing request fields, extremely large amounts, decimal amounts
- **Concurrent Operations**: Rapid deposit-withdraw cycles, concurrent deposits, parallel queries

## Reusable Fixtures

The `fixtures/protocol.ts` module provides:

### Configuration (`PROTOCOL_CONFIG`)

- Host/port for the test node
- Contract addresses (vault, reward, governance, staking)
- Token definitions (USDC, AXN, ETH, Reward) with decimals
- Admin wallet credentials
- Predefined test amounts and durations

### Test Users (`PROTOCOL_USERS`)

Five predefined users (Alice, Bob, Charlie, Dave, Eve) with addresses and nonce tracking.

### Utilities

- `createClient(host, port)` - gRPC client factory
- `callRpc<T>(client, method, request)` - Promise-based RPC wrapper
- `generateSignature(userAddress, nonce)` - Mock signature generator
- `attempt<T>(operation, fallbackMessage)` - Graceful try-catch for unavailable services
- `expectRpcSuccess<T>` - Assert RPC success

## Running Tests

```bash
# Run all protocol integration tests
npx vitest run --config vitest.integration.config.ts tests/integration/protocol/ tests/integration/scenarios/

# Run a specific test suite
npx vitest run --config vitest.integration.config.ts tests/integration/protocol/cross-contract-vault.test.ts

# Run failure scenarios only
npx vitest run --config vitest.integration.config.ts tests/integration/scenarios/failure.test.ts

# Run with npm script
npm run test:integration
```

## CI Integration

Tests are executed automatically via the `protocol-integration-tests.yml` workflow:

1. **Matrix Strategy**: Four suites run in parallel (cross-contract, multi-asset, governance, failure scenarios)
2. **Comprehensive Suite**: Aggregated run of all protocol and scenario tests
3. **Artifacts**: Test results are uploaded for each matrix job
4. **Docker Cleanup**: Automatic cleanup of test containers and networks

## Adding New Tests

1. Add new fixture data to `tests/integration/fixtures/protocol.ts` if needed
2. Create a new test file in `tests/integration/protocol/` or `tests/integration/scenarios/`
3. Follow the existing pattern using `callRpc`, `attempt`, and `generateSignature` utilities
4. Use `attempt()` for calls that may fail gracefully when services are unavailable
5. Register the test in the CI matrix if it should run as a separate job

## Best Practices

- Use `attempt()` for RPC calls that may not be available in all environments
- Track nonces per-user to simulate realistic interaction patterns
- Validate both success cases and failure modes
- Keep tests independent and order-agnostic
- Use `PROTOCOL_CONFIG` for all addresses and amounts to maintain consistency
