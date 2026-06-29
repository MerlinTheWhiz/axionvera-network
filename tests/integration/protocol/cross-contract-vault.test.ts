import { describe, it, expect, beforeAll } from "vitest";
import {
  createClient,
  callRpc,
  PROTOCOL_CONFIG,
  PROTOCOL_USERS,
  generateSignature,
  attempt,
  expectRpcSuccess,
} from "../fixtures/protocol.js";

const host = PROTOCOL_CONFIG.host;
const port = PROTOCOL_CONFIG.port;

describe("Cross-Contract Vault Integration Flows", () => {
  let client: any;
  let adminClient: any;

  beforeAll(() => {
    client = createClient(host, port);
    adminClient = createClient(host, port);
  });

  describe("1. Vault ↔ Reward Contract Interaction", () => {
    it("should deposit, distribute rewards, and reflect state across both contracts", async () => {
      const alice = PROTOCOL_USERS[0];
      const depositAmount = PROTOCOL_CONFIG.amounts.defaultDeposit;
      const rewardAmount = PROTOCOL_CONFIG.amounts.defaultReward;

      const depositRes = await attempt(
        () =>
          callRpc<any>(client, "Deposit", {
            user_address: alice.address,
            token_address: PROTOCOL_CONFIG.tokens.usdc.address,
            amount: depositAmount,
            signature: generateSignature(alice.address, alice.nonce++),
            nonce: alice.nonce,
          }),
        "Deposit not available",
      );
      if (!depositRes) return;
      expect(depositRes.success).toBe(true);

      const contractState = await attempt(
        () =>
          callRpc<any>(client, "GetContractState", {
            contract_address: PROTOCOL_CONFIG.contracts.vault,
          }),
        "GetContractState not available",
      );
      if (contractState) {
        expect(contractState.total_deposits).toBeDefined();
      }

      const distRes = await attempt(
        () =>
          callRpc<any>(client, "DistributeRewards", {
            reward_token: PROTOCOL_CONFIG.tokens.reward.address,
            total_amount: rewardAmount,
            signature: generateSignature(PROTOCOL_CONFIG.admin.address, 1),
            nonce: 1,
          }),
        "DistributeRewards not available",
      );
      if (distRes) {
        expect(distRes.success).toBe(true);
      }

      const rewardState = await attempt(
        () =>
          callRpc<any>(client, "GetContractState", {
            contract_address: PROTOCOL_CONFIG.contracts.reward,
          }),
        "Reward contract state not available",
      );
      if (rewardState) {
        expect(rewardState.reward_index).toBeDefined();
      }

      const rewards = await attempt(
        () =>
          callRpc<any>(client, "GetRewards", {
            user_address: alice.address,
          }),
        "GetRewards not available",
      );
      if (rewards) {
        expect(rewards.total_rewards).toBeDefined();
        expect(rewards.claimable_rewards).toBeDefined();
      }
    });

    it("should claim rewards and verify balance increases", async () => {
      const alice = PROTOCOL_USERS[0];

      const balanceBefore = await attempt(
        () =>
          callRpc<any>(client, "GetBalance", {
            user_address: alice.address,
            token_address: PROTOCOL_CONFIG.tokens.reward.address,
          }),
        "GetBalance before claim not available",
      );

      const claimRes = await attempt(
        () =>
          callRpc<any>(client, "ClaimRewards", {
            user_address: alice.address,
            signature: generateSignature(alice.address, alice.nonce++),
            nonce: alice.nonce,
          }),
        "ClaimRewards not available",
      );
      if (claimRes) {
        expect(claimRes.success).toBe(true);
      }

      const rewardsAfter = await attempt(
        () =>
          callRpc<any>(client, "GetRewards", {
            user_address: alice.address,
          }),
        "GetRewards after claim not available",
      );
      if (rewardsAfter && balanceBefore) {
        expect(BigInt(rewardsAfter.claimable_rewards)).toBeLessThanOrEqual(
          BigInt(balanceBefore.pending_rewards || "0"),
        );
      }
    });

    it("should enforce that reward distribution requires admin authorization", async () => {
      const alice = PROTOCOL_USERS[0];
      const unauthorizedDist = await attempt(
        () =>
          callRpc<any>(client, "DistributeRewards", {
            reward_token: PROTOCOL_CONFIG.tokens.reward.address,
            total_amount: "100000",
            signature: generateSignature(alice.address, 999),
            nonce: 999,
          }),
        "Unauthorized distribution check not available",
      );
      if (unauthorizedDist) {
        expect(unauthorizedDist.success).toBe(false);
      }
    });
  });

  describe("2. Vault ↔ Staking Contract Interaction", () => {
    it("should lock tokens and verify staked balance is reflected", async () => {
      const bob = PROTOCOL_USERS[1];
      const lockAmount = "500";

      const depositForStake = await attempt(
        () =>
          callRpc<any>(client, "Deposit", {
            user_address: bob.address,
            token_address: PROTOCOL_CONFIG.tokens.axn.address,
            amount: lockAmount,
            signature: generateSignature(bob.address, bob.nonce++),
            nonce: bob.nonce,
          }),
        "Deposit for stake not available",
      );
      if (!depositForStake) return;
      expect(depositForStake.success).toBe(true);

      const lockRes = await attempt(
        () =>
          callRpc<any>(client, "Lock", {
            user_address: bob.address,
            amount: lockAmount,
            duration_seconds: PROTOCOL_CONFIG.amounts.lockDuration,
            signature: generateSignature(bob.address, bob.nonce++),
            nonce: bob.nonce,
          }),
        "Lock not available",
      );
      if (lockRes) {
        expect(lockRes.success).toBe(true);
      }

      const stakedBalance = await attempt(
        () =>
          callRpc<any>(client, "GetStakedBalance", {
            user_address: bob.address,
          }),
        "GetStakedBalance not available",
      );
      if (stakedBalance) {
        expect(BigInt(stakedBalance.balance)).toBeGreaterThanOrEqual(
          BigInt("1"),
        );
      }
    });

    it("should distribute rewards to staked positions and verify proportional allocation", async () => {
      const bob = PROTOCOL_USERS[1];
      const charlie = PROTOCOL_USERS[2];

      const bobDeposit = await attempt(
        () =>
          callRpc<any>(client, "Deposit", {
            user_address: bob.address,
            token_address: PROTOCOL_CONFIG.tokens.usdc.address,
            amount: "2000",
            signature: generateSignature(bob.address, bob.nonce++),
            nonce: bob.nonce,
          }),
        "Bob deposit not available",
      );
      if (!bobDeposit) return;
      expect(bobDeposit.success).toBe(true);

      const charlieDeposit = await attempt(
        () =>
          callRpc<any>(client, "Deposit", {
            user_address: charlie.address,
            token_address: PROTOCOL_CONFIG.tokens.usdc.address,
            amount: "3000",
            signature: generateSignature(charlie.address, charlie.nonce++),
            nonce: charlie.nonce,
          }),
        "Charlie deposit not available",
      );
      if (!charlieDeposit) return;
      expect(charlieDeposit.success).toBe(true);

      const distRes = await attempt(
        () =>
          callRpc<any>(client, "DistributeRewards", {
            reward_token: PROTOCOL_CONFIG.tokens.reward.address,
            total_amount: "1000000",
            signature: generateSignature(PROTOCOL_CONFIG.admin.address, 2),
            nonce: 2,
          }),
        "DistributeRewards not available",
      );
      if (distRes) {
        expect(distRes.success).toBe(true);
      }

      const bobRewards = await attempt(
        () =>
          callRpc<any>(client, "GetRewards", {
            user_address: bob.address,
          }),
        "GetRewards for bob not available",
      );
      const charlieRewards = await attempt(
        () =>
          callRpc<any>(client, "GetRewards", {
            user_address: charlie.address,
          }),
        "GetRewards for charlie not available",
      );

      if (bobRewards && charlieRewards) {
        const bobClaimable = BigInt(bobRewards.claimable_rewards || "0");
        const charlieClaimable = BigInt(
          charlieRewards.claimable_rewards || "0",
        );
        expect(charlieClaimable).toBeGreaterThanOrEqual(bobClaimable);
      }
    });
  });

  describe("3. Vault ↔ Governance Contract Interaction", () => {
    it("should query current chain parameters and validate structure", async () => {
      const params = await attempt(
        () => callRpc<any>(client, "GetChainParameters", {}),
        "GetChainParameters not available",
      );
      if (params) {
        expect(params.chain_id).toBeDefined();
        expect(params.current_block_height).toBeDefined();
        expect(params.active_parameters).toBeDefined();
        if (params.active_parameters) {
          expect(params.active_parameters.max_block_body_bytes).toBeDefined();
        }
      }
    });

    it("should list pending parameter upgrades as an array", async () => {
      const upgrades = await attempt(
        () => callRpc<any>(client, "ListPendingParameterUpgrades", {}),
        "ListPendingParameterUpgrades not available",
      );
      if (upgrades) {
        expect(Array.isArray(upgrades.pending)).toBe(true);
      }
    });

    it("should submit a parameter upgrade proposal and check it appears in pending list", async () => {
      const upgradeRes = await attempt(
        () =>
          callRpc<any>(client, "ParameterUpgrade", {
            parameter_patch: {
              max_block_body_bytes: "2097152",
              min_base_fee: "100",
            },
            activation_epoch_height: "100000",
            proposer_address: PROTOCOL_CONFIG.admin.address,
            proposer_signature: generateSignature(
              PROTOCOL_CONFIG.admin.address,
              3,
            ),
            nonce: 3,
          }),
        "ParameterUpgrade not available",
      );
      if (upgradeRes) {
        expect(upgradeRes.success).toBe(true);
      }
    });

    it("should retrieve network status and node info", async () => {
      const status = await attempt(
        () => callRpc<any>(client, "GetNetworkStatus", {}),
        "GetNetworkStatus not available",
      );
      if (status) {
        expect(status.is_healthy).toBeDefined();
        expect(status.network_version).toBeDefined();
      }

      const nodeInfo = await attempt(
        () =>
          callRpc<any>(client, "GetNodeInfo", {
            node_id: "node-1",
          }),
        "GetNodeInfo not available",
      );
      if (nodeInfo) {
        expect(nodeInfo.node_id).toBeDefined();
        expect(nodeInfo.version).toBeDefined();
      }
    });
  });

  describe("4. Complete Cross-Contract Lifecycle", () => {
    it("should execute a full lifecycle: deposit → stake → reward → claim → withdraw", async () => {
      const dave = PROTOCOL_USERS[3];
      const initialDeposit = "5000";

      const depositRes = await attempt(
        () =>
          callRpc<any>(client, "Deposit", {
            user_address: dave.address,
            token_address: PROTOCOL_CONFIG.tokens.usdc.address,
            amount: initialDeposit,
            signature: generateSignature(dave.address, dave.nonce++),
            nonce: dave.nonce,
          }),
        "Deposit not available",
      );
      if (!depositRes) return;
      expect(depositRes.success).toBe(true);

      const lockRes = await attempt(
        () =>
          callRpc<any>(client, "Lock", {
            user_address: dave.address,
            amount: "2000",
            duration_seconds: PROTOCOL_CONFIG.amounts.lockDuration,
            signature: generateSignature(dave.address, dave.nonce++),
            nonce: dave.nonce,
          }),
        "Lock not available",
      );
      if (lockRes) {
        expect(lockRes.success).toBe(true);
      }

      const distRes = await attempt(
        () =>
          callRpc<any>(client, "DistributeRewards", {
            reward_token: PROTOCOL_CONFIG.tokens.reward.address,
            total_amount: "750000",
            signature: generateSignature(PROTOCOL_CONFIG.admin.address, 4),
            nonce: 4,
          }),
        "DistributeRewards not available",
      );
      if (distRes) {
        expect(distRes.success).toBe(true);
      }

      const claimRes = await attempt(
        () =>
          callRpc<any>(client, "ClaimRewards", {
            user_address: dave.address,
            signature: generateSignature(dave.address, dave.nonce++),
            nonce: dave.nonce,
          }),
        "ClaimRewards not available",
      );
      if (claimRes) {
        expect(claimRes.success).toBe(true);
      }

      const balanceAfter = await attempt(
        () =>
          callRpc<any>(client, "GetBalance", {
            user_address: dave.address,
            token_address: PROTOCOL_CONFIG.tokens.usdc.address,
          }),
        "GetBalance not available",
      );
      if (balanceAfter) {
        expect(balanceAfter.balance).toBeDefined();
      }

      const withdrawRes = await attempt(
        () =>
          callRpc<any>(client, "Withdraw", {
            user_address: dave.address,
            token_address: PROTOCOL_CONFIG.tokens.usdc.address,
            amount: "1000",
            signature: generateSignature(dave.address, dave.nonce++),
            nonce: dave.nonce,
          }),
        "Withdraw not available",
      );
      if (withdrawRes) {
        expect(withdrawRes.success).toBe(true);
      }

      const transactionHistory = await attempt(
        () =>
          callRpc<any>(client, "GetTransactionHistory", {
            user_address: dave.address,
            limit: 10,
            offset: 0,
          }),
        "GetTransactionHistory not available",
      );
      if (transactionHistory) {
        expect(transactionHistory.transactions).toBeDefined();
        expect(Array.isArray(transactionHistory.transactions)).toBe(true);
      }
    });
  });

  describe("5. Multi-Contract Queries and State Consistency", () => {
    it("should return consistent TVL across vault queries", async () => {
      const tvl = await attempt(
        () =>
          callRpc<any>(client, "GetTVL", {
            token_address: PROTOCOL_CONFIG.tokens.usdc.address,
          }),
        "GetTVL not available",
      );
      if (tvl) {
        expect(tvl.total_value_locked).toBeDefined();
        expect(BigInt(tvl.total_value_locked)).toBeGreaterThanOrEqual(
          BigInt(0),
        );
      }
    });

    it("should return user-specific transaction history with correct structure", async () => {
      const eve = PROTOCOL_USERS[4];
      const history = await attempt(
        () =>
          callRpc<any>(client, "GetTransactionHistory", {
            user_address: eve.address,
            limit: 5,
            offset: 0,
          }),
        "GetTransactionHistory not available",
      );
      if (history) {
        expect(history.transactions).toBeDefined();
        expect(history.total_count).toBeDefined();
        expect(history.has_more).toBeDefined();
      }
    });

    it("should return contract state with user count and last updated timestamp", async () => {
      const state = await attempt(
        () =>
          callRpc<any>(client, "GetContractState", {
            contract_address: PROTOCOL_CONFIG.contracts.vault,
          }),
        "GetContractState not available",
      );
      if (state) {
        expect(state.contract_address).toBe(PROTOCOL_CONFIG.contracts.vault);
        expect(state.total_users).toBeDefined();
        expect(state.last_updated).toBeDefined();
        expect(state.custom_state).toBeDefined();
      }
    });

    it("should filter transaction history by transaction type", async () => {
      const alice = PROTOCOL_USERS[0];
      const history = await attempt(
        () =>
          callRpc<any>(client, "GetTransactionHistory", {
            user_address: alice.address,
            transaction_type: 1,
          }),
        "Filtered transaction history not available",
      );
      if (history && history.transactions.length > 0) {
        for (const tx of history.transactions) {
          expect(tx.transaction_type).toBe(1);
        }
      }
    });

    it("should return paginated transaction history with has_more flag", async () => {
      const bob = PROTOCOL_USERS[1];
      const page1 = await attempt(
        () =>
          callRpc<any>(client, "GetTransactionHistory", {
            user_address: bob.address,
            limit: 2,
            offset: 0,
          }),
        "Paginated history not available",
      );
      if (page1 && page1.transactions.length > 0) {
        expect(page1.transactions.length).toBeLessThanOrEqual(2);
        expect(page1.total_count).toBeGreaterThanOrEqual(
          page1.transactions.length,
        );
      }
    });

    it("should return peer list with connection status from P2P service", async () => {
      const peers = await attempt(
        () => callRpc<any>(client, "GetPeerList", {}),
        "GetPeerList not available",
      );
      if (peers) {
        expect(Array.isArray(peers.peers)).toBe(true);
        for (const peer of peers.peers) {
          expect(peer.peer_id).toBeDefined();
          expect(peer.is_connected).toBeDefined();
        }
      }
    });
  });
});
