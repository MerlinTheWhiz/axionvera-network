import * as grpc from "@grpc/grpc-js";
import * as protoLoader from "@grpc/proto-loader";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

export const PROTO_PATH = path.resolve(__dirname, "../../../proto/network.proto");

export const packageDefinition = protoLoader.loadSync(PROTO_PATH, {
  keepCase: true,
  longs: String,
  enums: String,
  defaults: true,
  oneofs: true,
});

export const networkProto = (
  grpc.loadPackageDefinition(packageDefinition) as any
).axionvera.network;

export function createClient(host: string, port: string): any {
  return new networkProto.NetworkService(
    `${host}:${port}`,
    grpc.credentials.createInsecure(),
  );
}

export async function callRpc<T>(
  client: any,
  method: string,
  request: any,
): Promise<T> {
  return new Promise((resolve, reject) => {
    const deadline = new Date();
    deadline.setSeconds(deadline.getSeconds() + 10);
    client[method](request, { deadline }, (error: any, response: any) => {
      if (error) {
        reject(error);
        return;
      }
      resolve(response as T);
    });
  });
}

export interface ProtocolUser {
  address: string;
  privateKey: string;
  nonce: number;
}

export interface ProtocolToken {
  address: string;
  name: string;
  decimals: number;
}

export interface CrossContractConfig {
  vaultContractAddress: string;
  rewardContractAddress: string;
  governanceContractAddress: string;
  stakingContractAddress: string;
}

export const PROTOCOL_CONFIG = {
  host: process.env.TEST_NODE_HOST || "localhost",
  port: process.env.TEST_NODE_PORT || "50051",
  contracts: {
    vault: "CAXIONVERA001",
    reward: "CREWARD_TOKEN_ADDRESS",
    governance: "CGOVERNANCE001",
    staking: "CSTAKING_TOKEN_ADDRESS",
  },
  tokens: {
    usdc: { address: "CUSDC_TOKEN_ADDRESS", name: "USD Coin", decimals: 7 },
    axn: { address: "CAXN_TOKEN_ADDRESS", name: "Axion Token", decimals: 7 },
    eth: { address: "CETH_TOKEN_ADDRESS", name: "Wrapped Ether", decimals: 7 },
    reward: {
      address: "CREWARD_REWARD_TOKEN_ADDRESS",
      name: "Reward Token",
      decimals: 7,
    },
  },
  admin: {
    address: "GBA_ADMIN0000000000000000000000000000000000000000000000",
    privateKey: "admin_private_key_123",
  },
  amounts: {
    minDeposit: "1",
    maxDeposit: "999999999999999999",
    defaultDeposit: "1000",
    defaultReward: "500000",
    minStake: "100",
    lockDuration: 86400 * 7,
  },
};

export const PROTOCOL_USERS: ProtocolUser[] = [
  {
    address: "GALICE1234567890ABCDEFGHIJ1234567890ABCDEFGHIJKL",
    privateKey: "alice_private_key",
    nonce: 1,
  },
  {
    address: "GBOB9876543210ABCDEFGHIJKLMNOPQRSTUVWXYZ0987654321",
    privateKey: "bob_private_key",
    nonce: 1,
  },
  {
    address: "GCHARLIE111222333444555666777888999000111222333444",
    privateKey: "charlie_private_key",
    nonce: 1,
  },
  {
    address: "GDAVE444555666777888999000111222333444555666777888",
    privateKey: "dave_private_key",
    nonce: 1,
  },
  {
    address: "GEVE999000111222333444555666777888999000111222333",
    privateKey: "eve_private_key",
    nonce: 1,
  },
];

export function generateSignature(userAddress: string, nonce: number): Buffer {
  return Buffer.from(`${userAddress}:${nonce}`);
}

export async function attempt<T>(
  operation: () => Promise<T>,
  fallbackMessage: string,
): Promise<T | undefined> {
  try {
    return await operation();
  } catch {
    console.log(`⚠️  ${fallbackMessage}`);
    return undefined;
  }
}

export async function expectRpcSuccess<T extends { success: boolean }>(
  client: any,
  method: string,
  request: any,
): Promise<T> {
  const res = await callRpc<T>(client, method, request);
  expect(res.success).toBe(true);
  return res;
}
