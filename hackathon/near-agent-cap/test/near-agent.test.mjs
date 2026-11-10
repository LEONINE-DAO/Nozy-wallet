import assert from "node:assert/strict";
import test from "node:test";
import { ask, propose, requestModelProposal } from "../src/near-agent.mjs";

test("a stake stays on the NEAR agent", () => {
  const result = propose({ kind: "stake", amount: "1" });
  assert.equal(result.ok, true);
  assert.equal(result.agent, "near");
  assert.equal(result.account, "nozy-agent.testnet");
  assert.equal(result.tx.receiverId, "lst.ref-dev.testnet");
});

test("ethereum has no agent", () => {
  const result = propose({ text: "stake 2 on ethereum" });
  assert.equal(result.ok, false);
  assert.equal(result.agent, null);
  assert.match(result.reason, /ethereum/);
});

test("an invoice is handed to the Zcash agent", () => {
  const result = propose({ text: "draft an invoice for 0.01" });
  assert.equal(result.agent, "zcash");
  assert.equal(result.handoff, "invoice");
  assert.equal(result.amount, "0.01");
});

test("a NEAR swap is a dry Intents quote for USDC", () => {
  const result = propose({ text: "swap 1 NEAR to USDC" });
  assert.equal(result.ok, true);
  assert.equal(result.agent, "near");
  assert.equal(result.swap.originAsset, "nep141:wrap.near");
  assert.equal(result.swap.destinationAsset, "nep141:17208628f84f5d6ad33f0da3bbbeb27ffcb398eac501a31bd6ad2011e36133a1");
  assert.equal(result.swap.dry, true);
  assert.equal(result.swap.recipient, "nozy-agent.near");
});

test("shielded ZEC does not go through NEAR Intents", () => {
  const result = propose({ text: "swap shielded zec through NEAR intents" });
  assert.equal(result.ok, false);
  assert.equal(result.agent, "near");
  assert.match(result.reason, /Shielded ZEC/);
});

test("a mainnet stake addresses Rhea on mainnet", () => {
  const result = propose({ text: "stake 1 on mainnet" });
  assert.equal(result.ok, true);
  assert.equal(result.tx.signerId, "nozy-agent.near");
  assert.equal(result.tx.receiverId, "lst.rhealab.near");
  assert.equal(result.tx.actions[0].params.methodName, "deposit_and_stake");
});

test("a live swap quote waits until the mainnet account exists", async () => {
  let quoted = false;
  const result = await ask(
    { text: "swap 1 NEAR to USDC", confirmed: true, live: true },
    {
      fetchImpl: async (url) => {
        if (String(url).includes("1click")) quoted = true;
        return { ok: true, json: async () => ({ error: { cause: { name: "UNKNOWN_ACCOUNT" } } }) };
      },
    },
  );
  assert.equal(quoted, false);
  assert.equal(result.ok, false);
  assert.match(result.reason, /nozy-agent\.near/);
});

test("a private RHEA quote does not call 1Click without a user session", async () => {
  let called = false;
  const result = await ask(
    { text: "private swap 1 NEAR to RHEA" },
    {
      fetchImpl: async () => {
        called = true;
        return { ok: true, json: async () => ({}) };
      },
    },
  );
  assert.equal(called, false);
  assert.equal(result.ok, false);
  assert.equal(result.confidentiality, "basic");
  assert.match(result.reason, /user session/);
});

test("a dry swap quote uses the 1Click response", async () => {
  const result = await ask(
    { text: "swap 1 NEAR to USDC" },
    {
      fetchImpl: async () => ({
        ok: true,
        json: async () => ({ quote: { amountInFormatted: "1", amountOutFormatted: "4.02", depositAddress: null } }),
      }),
    },
  );
  assert.equal(result.ok, true);
  assert.equal(result.quote.amountOut, "4.02");
  assert.equal(result.quote.depositAddress, null);
  assert.equal(result.egress, "clearnet");
});

test("a model stake proposal still has to pass the cap", async () => {
  const seen = [];
  const fetchImpl = async (url, options) => {
    seen.push(url);
    assert.equal(options.headers.authorization, "Bearer test-key");
    return {
      ok: true,
      json: async () => ({
        choices: [
          {
            message: {
              tool_calls: [
                { function: { name: "stake", arguments: JSON.stringify({ amount: "10" }) } },
              ],
            },
          },
        ],
      }),
    };
  };
  const result = await ask({ text: "stake 10" }, { apiKey: "test-key", fetchImpl });
  assert.equal(seen[0], "https://cloud-api.near.ai/v1/chat/completions");
  assert.equal(result.model, true);
  assert.equal(result.ok, false);
  assert.equal(result.needsConfirm, true);
  assert.equal(JSON.stringify(result).includes("test-key"), false);
});

test("the model host is allowlisted to NEAR AI Cloud", async () => {
  let called = false;
  const result = await requestModelProposal("stake 1", {
    apiKey: "test-key",
    url: "https://example.com/v1/chat/completions",
    fetchImpl: async () => {
      called = true;
      return { ok: true, json: async () => ({}) };
    },
  });
  assert.equal(called, false);
  assert.equal(result.ok, false);
  assert.match(result.reason, /cloud-api\.near\.ai/);
});

test("a missing NEAR AI key does not invent a model answer", async () => {
  const result = await requestModelProposal("stake 1", { apiKey: "" });
  assert.equal(result.model, false);
  assert.match(result.reason, /not connected/);
});
