import test from "node:test";
import assert from "node:assert/strict";
import {
  insufficientFundsMessage,
  mandatoryOrchardFeeZats,
  MANDATORY_ORCHARD_FEE_FALLBACK_ZATS,
  payForInclusionRequested,
  rpcFallbackWithRequester,
  selectNotesForSpend
} from "./tx-utils.js";

test("selectNotesForSpend prefers smallest single sufficient note", () => {
  const notes = [
    { value: 5000, height: 10 },
    { value: 11000, height: 11 },
    { value: 9000, height: 12 }
  ];
  const selected = selectNotesForSpend(notes, 8000);
  assert.equal(selected.length, 1);
  assert.equal(selected[0].value, 9000);
});

test("selectNotesForSpend accumulates multiple notes when needed", () => {
  const notes = [
    { value: 3000, height: 10 },
    { value: 4000, height: 12 },
    { value: 5000, height: 11 }
  ];
  const selected = selectNotesForSpend(notes, 9000);
  assert.ok(selected.length >= 2);
  const total = selected.reduce((acc, n) => acc + n.value, 0);
  assert.ok(total >= 9000);
});

test("selectNotesForSpend returns empty on insufficient funds", () => {
  const notes = [
    { value: 1000, height: 1 },
    { value: 2000, height: 2 }
  ];
  const selected = selectNotesForSpend(notes, 99999);
  assert.deepEqual(selected, []);
});

test("rpcFallbackWithRequester returns first successful attempt", async () => {
  let calls = 0;
  const result = await rpcFallbackWithRequester(async (attempt) => {
    calls += 1;
    if (attempt.method === "fail") throw new Error("nope");
    return "ok";
  }, [{ method: "fail" }, { method: "ok" }]);

  assert.equal(result, "ok");
  assert.equal(calls, 2);
});

test("mandatoryOrchardFeeZats falls back when wasm is missing", () => {
  assert.equal(mandatoryOrchardFeeZats(null, ""), MANDATORY_ORCHARD_FEE_FALLBACK_ZATS);
  assert.equal(mandatoryOrchardFeeZats({}, "hello"), MANDATORY_ORCHARD_FEE_FALLBACK_ZATS);
});

test("mandatoryOrchardFeeZats uses the inclusion fee from wasm", () => {
  const wasm = {
    estimate_orchard_send_fee_zats: (memo, priority) => {
      assert.equal(priority, true);
      return memo ? 100_000 : 100_000;
    }
  };
  assert.equal(mandatoryOrchardFeeZats(wasm, ""), 100_000);
});

test("mandatoryOrchardFeeZats uses getstandardfee from JS when WASM has no quote export", () => {
  const wasm = {
    estimate_orchard_send_fee_zats_at: () => 40_000
  };
  assert.equal(
    mandatoryOrchardFeeZats(wasm, "", { mainnet: true, minedBlockHeight: 10, marginalFee: 1000 }, true),
    20_000
  );
  assert.equal(
    mandatoryOrchardFeeZats(wasm, "", { mainnet: true, minedBlockHeight: 10, marginalFee: 1000 }, false),
    2_000
  );
});

test("payForInclusionRequested is off unless priority is explicitly true", () => {
  assert.equal(payForInclusionRequested(undefined), false);
  assert.equal(payForInclusionRequested({}), false);
  assert.equal(payForInclusionRequested({ priority: false }), false);
  assert.equal(payForInclusionRequested({ priority: true }), true);
});

test("insufficientFundsMessage shows amount plus fee in ZEC", () => {
  assert.equal(
    insufficientFundsMessage(8_000_000, 100_000, 8_034_191),
    "Insufficient funds. This send needs 0.08100000 ZEC (0.08000000 + 0.00100000 fee) but this wallet has 0.08034191 ZEC spendable. Lower the amount so the network fee still fits."
  );
});

test("mandatoryOrchardFeeZats prefers the getstandardfee quote", () => {
  const wasm = {
    estimate_orchard_send_fee_zats_quoted: (memo, priority, mainnet, height, marginal) => {
      assert.equal(priority, true);
      assert.equal(marginal, 1000);
      return 20_000;
    }
  };
  assert.equal(
    mandatoryOrchardFeeZats(wasm, "", { mainnet: true, minedBlockHeight: 10, marginalFee: 1000 }),
    20_000
  );
});

