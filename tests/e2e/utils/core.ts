// E2E Test Utilities
// Shared utilities for end-to-end testing of Axionvera Network protocols

import { expect, describe, it, beforeAll, afterAll } from "vitest";
import * as grpc from "@grpc/grpc-js";
import * as protoLoader from "@grpc/proto-loader";
import path from "path";
import { fileURLToPath } from "url";

export const TEST_TIMEOUT = 60000; // 60 seconds per test
export const GLOBAL_TIMEOUT = 300000; // 5 minutes for global setup

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const PROTO_PATH = path.resolve(__dirname, "../../proto/network.proto");

const packageDefinition = protoLoader.loadSync(PROTO_PATH, {
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

export class User {
  publicAddress: string;
  privateKey: string;
  nonce: number;
  constructor(publicAddress: string, privateKey: string) {
    this.publicAddress = publicAddress;
    this.privateKey = privateKey;
    this.nonce = 1;
  }

  incrementNonce(): void {
    this.nonce++;
  }

  generateSignature(message: string): Buffer {
    return Buffer.from(`${this.privateKey}:${message}:${this.nonce}`);
  }
}

export class TestUtils {
  static generateRandomAddress(): string {
    return `G${Array.from({ length: 63 }, () => Math.floor(Math.random() * 36)).join("")}`;
  }

  static generateTokenAddress(): string {
    return `C${Array.from({ length: 63 }, () => Math.floor(Math.random() * 36)).join("")}`;
  }

  static async wait(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  static async retry<T>(
    operation: () => Promise<T>,
    maxAttempts: number = 3,
    delay: number = 1000,
  ): Promise<T> {
    let lastError: Error | null = null;

    for (let attempt = 1; attempt <= maxAttempts; attempt++) {
      try {
        return await operation();
      } catch (error) {
        lastError = error as Error;
        if (attempt < maxAttempts) {
          await this.wait(delay);
          delay *= 2;
        }
      }
    }

    throw lastError || new Error("Operation failed after retries");
  }

  static async setupMultiUserEnvironment(count: number = 5): Promise<User[]> {
    const users: User[] = [];
    for (let i = 0; i < count; i++) {
      users.push(new User(this.generateRandomAddress(), `priv_${i}`));
    }
    return users;
  }
}
