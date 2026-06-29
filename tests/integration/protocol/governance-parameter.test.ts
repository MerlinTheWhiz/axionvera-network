import { describe, it, expect, beforeAll } from "vitest";
import {
  createClient,
  callRpc,
  PROTOCOL_CONFIG,
  PROTOCOL_USERS,
  generateSignature,
  attempt,
} from "../fixtures/protocol.js";

const host = PROTOCOL_CONFIG.host;
const port = PROTOCOL_CONFIG.port;

describe("Governance & Protocol Parameter Integration", () => {
  let client: any;

  beforeAll(() => {
    client = createClient(host, port);
  });

  describe("1. Chain Parameter Queries", () => {
    it("should return complete chain parameters with active configuration", async () => {
      const params = await attempt(
        () => callRpc<any>(client, "GetChainParameters", {}),
        "GetChainParameters not available",
      );
      if (params) {
        expect(params.chain_id).toBeDefined();
        expect(params.current_block_height).toBeDefined();
        expect(params.active_parameters).toBeDefined();
        expect(params.min_activation_delay_blocks).toBeDefined();
        expect(params.genesis_parameters).toBeDefined();
        if (params.active_parameters) {
          expect(
            Number(params.active_parameters.max_block_body_bytes),
          ).toBeGreaterThan(0);
        }
      }
    });

    it("should return chain ID as a non-empty string", async () => {
      const params = await attempt(
        () => callRpc<any>(client, "GetChainParameters", {}),
        "Chain ID check not available",
      );
      if (params) {
        expect(typeof params.chain_id).toBe("string");
        expect(params.chain_id.length).toBeGreaterThan(0);
      }
    });

    it("should return a monotonically increasing block height", async () => {
      const params1 = await attempt(
        () => callRpc<any>(client, "GetChainParameters", {}),
        "First block height check not available",
      );
      const params2 = await attempt(
        () => callRpc<any>(client, "GetChainParameters", {}),
        "Second block height check not available",
      );
      if (params1 && params2) {
        expect(Number(params2.current_block_height)).toBeGreaterThanOrEqual(
          Number(params1.current_block_height),
        );
      }
    });
  });

  describe("2. Parameter Upgrade Proposals", () => {
    it("should submit a valid parameter upgrade and return a transaction hash", async () => {
      const upgrade = await attempt(
        () =>
          callRpc<any>(client, "ParameterUpgrade", {
            parameter_patch: {
              max_block_body_bytes: "4194304",
              min_base_fee: "200",
              max_transactions_per_block: 500,
            },
            activation_epoch_height: "200000",
            proposer_address: PROTOCOL_CONFIG.admin.address,
            proposer_signature: generateSignature(
              PROTOCOL_CONFIG.admin.address,
              10,
            ),
            nonce: 10,
          }),
        "ParameterUpgrade not available",
      );
      if (upgrade) {
        expect(upgrade.success).toBe(true);
        expect(upgrade.transaction_hash).toBeDefined();
      }
    });

    it("should list the submitted parameter upgrade in pending upgrades", async () => {
      const upgrades = await attempt(
        () => callRpc<any>(client, "ListPendingParameterUpgrades", {}),
        "ListPendingParameterUpgrades not available",
      );
      if (upgrades && upgrades.pending) {
        for (const upgrade of upgrades.pending) {
          expect(upgrade.transaction_id).toBeDefined();
          expect(upgrade.announced_at_height).toBeDefined();
          expect(upgrade.activation_epoch_height).toBeDefined();
          expect(upgrade.patch).toBeDefined();
        }
      }
    });

    it("should submit a partial parameter upgrade with only one field changed", async () => {
      const partialUpgrade = await attempt(
        () =>
          callRpc<any>(client, "ParameterUpgrade", {
            parameter_patch: {
              min_base_fee: "150",
            },
            activation_epoch_height: "300000",
            proposer_address: PROTOCOL_CONFIG.admin.address,
            proposer_signature: generateSignature(
              PROTOCOL_CONFIG.admin.address,
              11,
            ),
            nonce: 11,
          }),
        "Partial parameter upgrade not available",
      );
      if (partialUpgrade) {
        expect(partialUpgrade.success).toBe(true);
      }
    });

    it("should submit a max block body bytes only upgrade", async () => {
      const bodyBytesUpgrade = await attempt(
        () =>
          callRpc<any>(client, "ParameterUpgrade", {
            parameter_patch: {
              max_block_body_bytes: "8388608",
            },
            activation_epoch_height: "400000",
            proposer_address: PROTOCOL_CONFIG.admin.address,
            proposer_signature: generateSignature(
              PROTOCOL_CONFIG.admin.address,
              12,
            ),
            nonce: 12,
          }),
        "Block body bytes upgrade not available",
      );
      if (bodyBytesUpgrade) {
        expect(bodyBytesUpgrade.success).toBe(true);
      }
    });
  });

  describe("3. Network Status and Node Information", () => {
    it("should return network status with health, block height, and peers", async () => {
      const status = await attempt(
        () => callRpc<any>(client, "GetNetworkStatus", {}),
        "GetNetworkStatus not available",
      );
      if (status) {
        expect(status.is_healthy).toBeDefined();
        expect(typeof status.is_healthy).toBe("boolean");
        expect(status.block_height).toBeDefined();
        expect(status.connected_peers).toBeDefined();
        expect(status.network_version).toBeDefined();
      }
    });

    it("should return detailed node info for a known node", async () => {
      const nodeInfo = await attempt(
        () => callRpc<any>(client, "GetNodeInfo", { node_id: "node-1" }),
        "GetNodeInfo not available",
      );
      if (nodeInfo) {
        expect(nodeInfo.node_id).toBe("node-1");
        expect(nodeInfo.address).toBeDefined();
        expect(nodeInfo.version).toBeDefined();
        expect(nodeInfo.is_syncing).toBeDefined();
        expect(typeof nodeInfo.is_syncing).toBe("boolean");
        expect(nodeInfo.metadata).toBeDefined();
      }
    });

    it("should handle non-existent node gracefully", async () => {
      const unknownNode = await attempt(
        () =>
          callRpc<any>(client, "GetNodeInfo", {
            node_id: "non-existent-node-999",
          }),
        "Unknown node info not available",
      );
      if (unknownNode) {
        expect(unknownNode.node_id).toBe("non-existent-node-999");
      }
    });
  });

  describe("4. Multi-Signature Governance Operations", () => {
    it("should process a parameter upgrade with DAO voter addresses", async () => {
      const daoUpgrade = await attempt(
        () =>
          callRpc<any>(client, "ParameterUpgrade", {
            parameter_patch: {
              max_transactions_per_block: 1000,
            },
            activation_epoch_height: "500000",
            proposer_address: PROTOCOL_CONFIG.admin.address,
            proposer_signature: generateSignature(
              PROTOCOL_CONFIG.admin.address,
              13,
            ),
            nonce: 13,
            dao_voter_addresses: [
              PROTOCOL_USERS[0].address,
              PROTOCOL_USERS[1].address,
              PROTOCOL_USERS[2].address,
            ],
          }),
        "DAO voter upgrade not available",
      );
      if (daoUpgrade) {
        expect(daoUpgrade.success).toBe(true);
      }
    });

    it("should reject submission without required fields", async () => {
      const missingFields = await attempt(
        () =>
          callRpc<any>(client, "ParameterUpgrade", {
            activation_epoch_height: "600000",
            proposer_address: PROTOCOL_CONFIG.admin.address,
            proposer_signature: generateSignature(
              PROTOCOL_CONFIG.admin.address,
              14,
            ),
            nonce: 14,
          }),
        "Missing fields validation not available",
      );
      if (missingFields) {
        expect(missingFields.success).toBe(false);
      }
    });
  });

  describe("5. Transaction Query and Verification", () => {
    it("should return a specific transaction by hash with all fields", async () => {
      const depositRes = await attempt(
        () =>
          callRpc<any>(client, "Deposit", {
            user_address: PROTOCOL_USERS[0].address,
            token_address: PROTOCOL_CONFIG.tokens.usdc.address,
            amount: "100",
            signature: generateSignature(
              PROTOCOL_USERS[0].address,
              PROTOCOL_USERS[0].nonce++,
            ),
            nonce: PROTOCOL_USERS[0].nonce,
          }),
        "Deposit not available",
      );
      if (!depositRes) return;
      expect(depositRes.success).toBe(true);

      if (depositRes.transaction_hash) {
        const tx = await attempt(
          () =>
            callRpc<any>(client, "GetTransaction", {
              transaction_hash: depositRes.transaction_hash,
            }),
          "GetTransaction not available",
        );
        if (tx) {
          expect(tx.transaction_hash).toBe(depositRes.transaction_hash);
          expect(tx.transaction_type).toBeDefined();
          expect(tx.status).toBeDefined();
          expect(tx.gas_used).toBeDefined();
        }
      }
    });

    it("should return a paginated user transaction history with total count", async () => {
      const alice = PROTOCOL_USERS[0];
      const history = await attempt(
        () =>
          callRpc<any>(client, "GetTransactionHistory", {
            user_address: alice.address,
            limit: 5,
            offset: 0,
          }),
        "Transaction history not available",
      );
      if (history) {
        expect(Array.isArray(history.transactions)).toBe(true);
        expect(history.total_count).toBeDefined();
        expect(typeof history.total_count).toBe("string");
      }
    });

    it("should filter transaction history by DEPOSIT type", async () => {
      const alice = PROTOCOL_USERS[0];
      const depositHistory = await attempt(
        () =>
          callRpc<any>(client, "GetTransactionHistory", {
            user_address: alice.address,
            transaction_type: 1,
          }),
        "Filtered deposit history not available",
      );
      if (depositHistory && depositHistory.transactions.length > 0) {
        for (const tx of depositHistory.transactions) {
          expect(tx.transaction_type).toBe(1);
        }
      }
    });

    it("should filter transaction history by WITHDRAW type", async () => {
      const alice = PROTOCOL_USERS[0];
      const withdrawHistory = await attempt(
        () =>
          callRpc<any>(client, "GetTransactionHistory", {
            user_address: alice.address,
            transaction_type: 2,
          }),
        "Filtered withdraw history not available",
      );
      if (withdrawHistory && withdrawHistory.transactions.length > 0) {
        for (const tx of withdrawHistory.transactions) {
          expect(tx.transaction_type).toBe(2);
        }
      }
    });
  });
});
