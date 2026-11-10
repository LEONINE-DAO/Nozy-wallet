import assert from "node:assert/strict";
import test from "node:test";
import { act } from "../src/act.mjs";

test("a 1 NEAR stake is allowed and unsigned", () => {
  const result = act({ kind: "stake", amount: "1" });
  assert.equal(result.ok, true);
  assert.equal(result.unsigned, true);
  assert.equal(result.tx.receiverId, "lst.ref-dev.testnet");
  assert.equal(result.tx.actions[0].params.methodName, "deposit_and_stake");
});

test("a 10 NEAR stake waits for the confirmation card", () => {
  const refused = act({ kind: "stake", amount: "10" });
  assert.equal(refused.ok, false);
  assert.equal(refused.needsConfirm, true);

  const confirmed = act({ kind: "stake", amount: "10", confirmed: true });
  assert.equal(confirmed.ok, true);
});

test("paying a stranger is refused", () => {
  const result = act({ kind: "pay", to: "stranger.near", amount: "1" });
  assert.equal(result.ok, false);
  assert.match(result.reason, /stranger/);
});

test("the human wallet cannot stake", () => {
  const result = act({ kind: "stake", amount: "1", useHuman: true });
  assert.equal(result.ok, false);
  assert.match(result.reason, /human wallet/);
});

test("the agent cannot disable Nym", () => {
  const result = act({ kind: "disable-nym" });
  assert.equal(result.ok, false);
  assert.match(result.reason, /Nym/);
});

test("an invoice draft is not payable by the agent", () => {
  const result = act({ kind: "invoice", amount: "0.01" });
  assert.equal(result.ok, true);
  assert.equal(result.invoice.agent_may_sign, false);
  assert.equal(result.invoice.payment_address, null);
});
