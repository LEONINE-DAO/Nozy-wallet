import assert from "node:assert/strict";
import test from "node:test";
import { evaluateAction, evaluateSwap, evaluateTransaction } from "../src/policy.mjs";

const human = ["human.near"];

test("stake under the cap is allowed", () => {
  const result = evaluateAction({
    signerId: "agent.near",
    humanWalletIds: human,
    receiverId: "lst.ref-dev.testnet",
    methodName: "deposit_and_stake",
    amountNear: "1",
    capNear: "5",
  });
  assert.equal(result.ok, true);
});

test("stake over the cap is refused until confirmed", () => {
  const over = evaluateAction({
    signerId: "agent.near",
    humanWalletIds: human,
    receiverId: "lst.ref-dev.testnet",
    methodName: "deposit_and_stake",
    amountNear: "10",
    capNear: "5",
  });
  assert.equal(over.ok, false);
  assert.match(over.reason, /cap/);

  const confirmed = evaluateAction({
    signerId: "agent.near",
    humanWalletIds: human,
    receiverId: "lst.ref-dev.testnet",
    methodName: "deposit_and_stake",
    amountNear: "10",
    capNear: "5",
    confirmed: true,
  });
  assert.equal(confirmed.ok, true);
});

test("a different contract is refused", () => {
  const result = evaluateAction({
    signerId: "agent.near",
    humanWalletIds: human,
    receiverId: "evil.near",
    methodName: "deposit_and_stake",
    amountNear: "1",
  });
  assert.equal(result.ok, false);
  assert.match(result.reason, /receiver/);
});

test("a payment to a stranger is refused", () => {
  const result = evaluateTransaction(
    {
      signerId: "agent.near",
      receiverId: "stranger.near",
      actions: [{ type: "Transfer", params: { deposit: "1000000000000000000000000" } }],
    },
    { humanWalletIds: human },
  );
  assert.equal(result.ok, false);
  assert.match(result.reason, /stranger/);
});

test("a transfer method is refused", () => {
  const result = evaluateAction({
    signerId: "agent.near",
    humanWalletIds: human,
    receiverId: "lst.ref-dev.testnet",
    methodName: "transfer",
    amountNear: "1",
  });
  assert.equal(result.ok, false);
  assert.match(result.reason, /method/);
});

test("the human wallet cannot sign", () => {
  const result = evaluateAction({
    signerId: "human.near",
    humanWalletIds: human,
    receiverId: "lst.ref-dev.testnet",
    methodName: "deposit_and_stake",
    amountNear: "1",
  });
  assert.equal(result.ok, false);
  assert.match(result.reason, /human wallet/);
});

test("a mainnet Rhea receiver is the Rhea contract", () => {
  const result = evaluateAction({
    network: "mainnet",
    signerId: "nozy-agent.near",
    humanWalletIds: human,
    receiverId: "lst.rhealab.near",
    methodName: "withdraw_all",
    confirmed: true,
  });
  assert.equal(result.ok, true);
});

test("a private RHEA quote asks for basic confidentiality and stays dry", () => {
  const result = evaluateSwap({
    from: "NEAR",
    to: "RHEA",
    amount: "1",
    accountId: "nozy-agent.near",
    confidentiality: "basic",
  });
  assert.equal(result.ok, true);
  assert.equal(result.request.dry, true);
  assert.equal(result.request.confidentiality, "basic");
  assert.equal(result.request.destinationAsset, "nep141:token.rhealab.near");
  assert.equal(result.request.depositType, "ORIGIN_CHAIN");
});

test("a confidential ZEC quote is still refused", () => {
  const result = evaluateSwap({
    from: "ZEC",
    to: "USDC",
    amount: "1",
    accountId: "nozy-agent.near",
    confidentiality: "basic",
  });
  assert.equal(result.ok, false);
  assert.match(result.reason, /ZEC/);
});

test("a ZEC asset cannot be quoted", () => {
  const result = evaluateSwap({
    from: "NEAR",
    to: "ZEC",
    amount: "1",
    accountId: "nozy-agent.near",
  });
  assert.equal(result.ok, false);
  assert.match(result.reason, /ZEC|NEAR, USDC/);
});

test("unstake of the full position needs confirmation", () => {
  const result = evaluateAction({
    signerId: "agent.near",
    humanWalletIds: human,
    receiverId: "lst.ref-dev.testnet",
    methodName: "unstake_all",
  });
  assert.equal(result.ok, false);
  assert.match(result.reason, /confirmation/);
});

test("a built transaction is checked action by action", () => {
  const tx = {
    signerId: "agent.near",
    receiverId: "lst.ref-dev.testnet",
    actions: [
      {
        type: "FunctionCall",
        params: {
          methodName: "storage_deposit",
          deposit: "1250000000000000000000",
          args: {},
        },
      },
      {
        type: "FunctionCall",
        params: {
          methodName: "deposit_and_stake",
          deposit: "1000000000000000000000000",
          args: {},
        },
      },
    ],
  };
  assert.equal(
    evaluateTransaction(tx, { humanWalletIds: human, capNear: "5" }).ok,
    true,
  );

  tx.actions[1].params.deposit = "10000000000000000000000000";
  const over = evaluateTransaction(tx, { humanWalletIds: human, capNear: "5" });
  assert.equal(over.ok, false);
});
