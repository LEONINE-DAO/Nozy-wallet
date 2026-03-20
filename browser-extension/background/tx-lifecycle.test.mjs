import test from "node:test";
import assert from "node:assert/strict";
import {
  buildBuiltTxStateEntry,
  buildFailedTxStateEntry,
  findRecentBuiltTxId,
  inferInputMode,
  nextLifecycleStateFromConfirmation,
  resolveTxidFromBroadcast
} from "./tx-lifecycle.js";

test("resolveTxidFromBroadcast resolves common RPC shapes", () => {
  assert.equal(resolveTxidFromBroadcast({ txid: "a" }, "f"), "a");
  assert.equal(resolveTxidFromBroadcast({ result: { txid: "b" } }, "f"), "b");
  assert.equal(resolveTxidFromBroadcast({ result: "c" }, "f"), "c");
  assert.equal(resolveTxidFromBroadcast("d", "f"), "d");
  assert.equal(resolveTxidFromBroadcast(null, "f"), "f");
});

test("inferInputMode returns explicit mode or infers from count", () => {
  assert.equal(inferInputMode(3, "single"), "single");
  assert.equal(inferInputMode(2, ""), "multi");
  assert.equal(inferInputMode(1, undefined), "single");
});

test("buildBuiltTxStateEntry maps proving payload", () => {
  const entry = buildBuiltTxStateEntry({
    id: "tx_1",
    origin: "https://app.test",
    createdAt: 1000,
    proving: {
      txid: "abc",
      recipientAddress: "u1recipient",
      amount: 123,
      fee: 10,
      memo: "m",
      inputs_used: 2,
      rawTxHex: "deadbeef"
    }
  });
  assert.equal(entry.state, "built");
  assert.equal(entry.inputMode, "multi");
  assert.equal(entry.txid, "abc");
});

test("buildFailedTxStateEntry uses parseAmount and preflight info", () => {
  const entry = buildFailedTxStateEntry({
    id: "tx_2",
    origin: "https://app.test",
    tx: { to: "u1to", amount: "42", memo: "err" },
    preflight: { inputs_used: 1 },
    error: "boom",
    createdAt: 2000,
    parseAmount: (v) => Number(v)
  });
  assert.equal(entry.state, "failed");
  assert.equal(entry.amount, 42);
  assert.equal(entry.inputMode, "single");
  assert.equal(entry.error, "boom");
});

test("findRecentBuiltTxId returns most recent matching built item", () => {
  const txs = [
    { id: "a", state: "built", origin: "x", updatedAt: 1000 },
    { id: "b", state: "pending", origin: "x", updatedAt: 1200 },
    { id: "c", state: "built", origin: "x", updatedAt: 1500 }
  ];
  assert.equal(findRecentBuiltTxId(txs, "x", 2000, 2000), "c");
  assert.equal(findRecentBuiltTxId(txs, "y", 2000, 2000), null);
});

test("nextLifecycleStateFromConfirmation maps confirm status", () => {
  assert.equal(nextLifecycleStateFromConfirmation({ confirmed: true }), "confirmed");
  assert.equal(nextLifecycleStateFromConfirmation({ confirmed: false }), "pending");
});
