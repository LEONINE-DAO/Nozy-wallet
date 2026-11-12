import test from "node:test";
import assert from "node:assert/strict";
import { applyShieldText, finishShield, formatUnits, idleShield, publicZcashKey } from "../src/wallets.mjs";

test("public zcash key drops the mnemonic", () => {
  const view = publicZcashKey({
    mnemonic: "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
    transparent_address: "t1YVKP6WGcvdJYb1XHehkfLYWXvBCt3BcZG",
    derivation_path: "m/44'/133'/0'/0/0",
  });
  assert.equal(view.address, "t1YVKP6WGcvdJYb1XHehkfLYWXvBCt3BcZG");
  assert.equal(JSON.stringify(view).includes("abandon"), false);
});

test("RHEA token units use 18 decimals", () => {
  assert.equal(formatUnits(10n ** 18n, 18), "1.0000");
  assert.equal(formatUnits(1500000000000000000n, 18), "1.5000");
});

test("shield output moves the live line and keeps the txid", () => {
  let state = idleShield();
  state = applyShieldText(state, "Reading the transparent coin.\nProving the Ironwood output.\n");
  assert.equal(state.phase, "proving");
  state = applyShieldText(state, "   Shielded value: 0.00147418 ZEC\n   Fee: 0.00150000 ZEC\n   TXID: abc\nBroadcast\n   TXID: abc\n");
  state = finishShield(state, 0, "");
  assert.equal(state.phase, "done");
  assert.equal(state.txid, "abc");
  assert.equal(state.shieldedZec, "0.00147418");
  assert.match(state.line, /abc/);
});
