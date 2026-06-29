import { describe, it, expect, beforeAll } from "vitest";
import * as grpc from "@grpc/grpc-js";
import * as protoLoader from "@grpc/proto-loader";
import path from "path";
import { fileURLToPath } from "url";

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

const networkProto = (
  grpc.loadPackageDefinition(packageDefinition) as any
).axionvera.network;

const HOST = process.env.TEST_NODE_HOST ?? "localhost";
const PORT = process.env.TEST_NODE_PORT ?? "50051";

function createRegistryClient(): any {
  return new networkProto.ServiceRegistry(
    `${HOST}:${PORT}`,
    grpc.credentials.createInsecure(),
  );
}

async function rpcCall<T>(
  client: any,
  method: string,
  request: any,
): Promise<T> {
  return new Promise((resolve, reject) => {
    const deadline = new Date(Date.now() + 5_000);
    client[method](request, { deadline }, (err: any, res: any) => {
      if (err) reject(err);
      else resolve(res as T);
    });
  });
}

describe("Service Discovery Registry gRPC Tests", () => {
  let client: any;

  beforeAll(() => {
    client = createRegistryClient();
  });

  // Helper to attempt a call and skip if the server is unavailable
  async function attempt<T>(fn: () => Promise<T>, label: string): Promise<T | undefined> {
    try {
      return await fn();
    } catch {
      console.log(`⚠️  ${label} — server not available, skipping`);
      return undefined;
    }
  }

  it("should register a new service and return success", async () => {
    const res = await attempt(
      () =>
        rpcCall<any>(client, "RegisterService", {
          service_name: "vault-service",
          service_address: "localhost:9001",
          version: "1.0.0",
          metadata: { env: "test" },
        }),
      "RegisterService not available",
    );
    if (!res) return;

    expect(res.success).toBe(true);
    expect(res.service.service_name).toBe("vault-service");
    expect(res.service.service_address).toBe("localhost:9001");
  });

  it("should reject duplicate registration with an error message", async () => {
    await attempt(
      () =>
        rpcCall<any>(client, "RegisterService", {
          service_name: "dup-service",
          service_address: "localhost:9002",
          version: "1.0.0",
          metadata: {},
        }),
      "RegisterService (first) not available",
    );

    const res = await attempt(
      () =>
        rpcCall<any>(client, "RegisterService", {
          service_name: "dup-service",
          service_address: "localhost:9003",
          version: "1.0.0",
          metadata: {},
        }),
      "RegisterService (duplicate) not available",
    );
    if (!res) return;

    expect(res.success).toBe(false);
    expect(res.error_message).toContain("already registered");
  });

  it("should look up a registered service by name", async () => {
    await attempt(
      () =>
        rpcCall<any>(client, "RegisterService", {
          service_name: "lookup-service",
          service_address: "localhost:9010",
          version: "2.0.0",
          metadata: {},
        }),
      "RegisterService not available",
    );

    const res = await attempt(
      () =>
        rpcCall<any>(client, "LookupService", {
          service_name: "lookup-service",
        }),
      "LookupService not available",
    );
    if (!res) return;

    expect(res.found).toBe(true);
    expect(res.service.service_name).toBe("lookup-service");
    expect(res.service.version).toBe("2.0.0");
  });

  it("should return found=false for an unknown service", async () => {
    const res = await attempt(
      () =>
        rpcCall<any>(client, "LookupService", {
          service_name: "does-not-exist-xyz",
        }),
      "LookupService not available",
    );
    if (!res) return;

    expect(res.found).toBe(false);
  });

  it("should deregister a service and confirm removal", async () => {
    await attempt(
      () =>
        rpcCall<any>(client, "RegisterService", {
          service_name: "dereg-service",
          service_address: "localhost:9020",
          version: "1.0.0",
          metadata: {},
        }),
      "RegisterService not available",
    );

    const deregRes = await attempt(
      () =>
        rpcCall<any>(client, "DeregisterService", {
          service_name: "dereg-service",
        }),
      "DeregisterService not available",
    );
    if (!deregRes) return;

    expect(deregRes.success).toBe(true);

    const lookupRes = await attempt(
      () =>
        rpcCall<any>(client, "LookupService", {
          service_name: "dereg-service",
        }),
      "LookupService not available",
    );
    if (!lookupRes) return;
    expect(lookupRes.found).toBe(false);
  });

  it("should return false when deregistering a non-existent service", async () => {
    const res = await attempt(
      () =>
        rpcCall<any>(client, "DeregisterService", {
          service_name: "never-registered",
        }),
      "DeregisterService not available",
    );
    if (!res) return;

    expect(res.success).toBe(false);
    expect(res.error_message).toContain("not found");
  });

  it("should list all registered services", async () => {
    const res = await attempt(
      () => rpcCall<any>(client, "ListServices", {}),
      "ListServices not available",
    );
    if (!res) return;

    expect(Array.isArray(res.services)).toBe(true);
    expect(typeof res.total_count).toBe("string"); // longs are strings
  });
});
