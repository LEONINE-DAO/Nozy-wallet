import test from "node:test";
import assert from "node:assert/strict";
import { isLightwalletdEndpoint, rpcStatusShouldFailover } from "./rpc-utils.js";

test("lightwalletd host is not treated as Zebrad JSON-RPC", () => {
  assert.equal(isLightwalletdEndpoint("https://lwd.nozywallet.org:443"), true);
  assert.equal(isLightwalletdEndpoint("lwd.nozywallet.org"), true);
  assert.equal(isLightwalletdEndpoint("http://127.0.0.1:8232"), false);
  assert.equal(isLightwalletdEndpoint("http://127.0.0.1:18232"), false);
});

test("HTTP 415 tries another RPC endpoint", () => {
  assert.equal(rpcStatusShouldFailover(415), true);
  assert.equal(rpcStatusShouldFailover(404), true);
  assert.equal(rpcStatusShouldFailover(200), false);
  assert.equal(rpcStatusShouldFailover(401), false);
  assert.equal(rpcStatusShouldFailover(500), false);
});
