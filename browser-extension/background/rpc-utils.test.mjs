import test from "node:test";
import assert from "node:assert/strict";
import {
  isLightwalletdEndpoint,
  postZebradRpc,
  rpcHttpFailureMessage,
  rpcStatusShouldFailover
} from "./rpc-utils.js";

const realFetch = globalThis.fetch;

test.afterEach(() => {
  globalThis.fetch = realFetch;
});

test("lightwalletd host is not treated as Zebrad JSON-RPC", () => {
  assert.equal(isLightwalletdEndpoint("https://lwd.nozywallet.org:443"), true);
  assert.equal(isLightwalletdEndpoint("lwd.nozywallet.org"), true);
  assert.equal(isLightwalletdEndpoint("https://zec.rocks"), true);
  assert.equal(isLightwalletdEndpoint("http://127.0.0.1:18232"), false);
  assert.equal(isLightwalletdEndpoint("http://127.0.0.1:8232"), false);
});

test("HTTP 415 tries another RPC endpoint", () => {
  assert.equal(rpcStatusShouldFailover(415), true);
  assert.equal(rpcStatusShouldFailover(404), true);
  assert.equal(rpcStatusShouldFailover(405), true);
  assert.equal(rpcStatusShouldFailover(501), true);
  assert.equal(rpcStatusShouldFailover(200), false);
  assert.equal(rpcStatusShouldFailover(401), false);
  assert.equal(rpcStatusShouldFailover(500), false);
});

test("HTTP 415 on getblockcount tells the sender to use a local node", () => {
  const message = rpcHttpFailureMessage(
    "getblockcount",
    415,
    "https://lwd.nozywallet.org:443"
  );
  assert.match(message, /Sending needs a local Zebrad or Zakura node/);
  assert.match(message, /18232/);
  assert.match(message, /HTTP 415/);
  assert.equal(message.includes("RPC getblockcount HTTP 415"), false);
});

function jsonResponse(status, result, contentType = "application/json") {
  return new Response(JSON.stringify({ jsonrpc: "2.0", id: 1, result }), {
    status,
    headers: { "content-type": contentType }
  });
}

test("JSON-RPC body stays bytes so Chrome does not append a charset", async () => {
  /** @type {Array<{ contentType: string | undefined, body: unknown }>} */
  const calls = [];
  globalThis.fetch = async (_url, init) => {
    calls.push({
      contentType: init.headers["Content-Type"],
      body: init.body
    });
    return jsonResponse(200, 148128);
  };
  const resp = await postZebradRpc("http://127.0.0.1:18232", "getblockcount", []);
  assert.equal(resp.status, 200);
  assert.equal(calls.length, 1);
  assert.equal(calls[0].contentType, "application/json");
  assert.notEqual(typeof calls[0].body, "string");
  assert.ok(calls[0].body instanceof Uint8Array);
});

test("HTTP 415 on application/json is retried as text/plain", async () => {
  /** @type {string[]} */
  const types = [];
  globalThis.fetch = async (_url, init) => {
    const contentType = init.headers["Content-Type"] || "";
    types.push(contentType);
    if (contentType === "application/json") return jsonResponse(415, null);
    return jsonResponse(200, 148128);
  };
  const resp = await postZebradRpc("http://127.0.0.1:18232", "getblockcount", []);
  assert.equal(resp.status, 200);
  assert.deepEqual(types, ["application/json", "text/plain"]);
});

test("gRPC HTTP 415 is not retried with another content type", async () => {
  let calls = 0;
  globalThis.fetch = async () => {
    calls += 1;
    return new Response("", {
      status: 415,
      headers: { "content-type": "application/grpc" }
    });
  };
  const resp = await postZebradRpc("https://lwd.nozywallet.org:443", "getblockcount", []);
  assert.equal(resp.status, 415);
  assert.equal(calls, 1);
});
