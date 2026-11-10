import assert from "node:assert/strict";
import test from "node:test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { companionHeaders, invoiceView, loadCompanionKey, unlockView } from "../src/companion.mjs";

test("unlock hides the seed preview the companion returns", () => {
  const view = unlockView(200, "word one two three ... last");
  assert.deepEqual(view, { ok: true, unlocked: true });
  assert.equal(JSON.stringify(view).includes("word"), false);
});

test("an open invoice does not release the order", () => {
  const view = invoiceView(200, {
    invoice_id: "inv_1",
    status: "open",
    amount_zec: 0.01,
    payment_address: "u1example",
    zcash_uri: "zcash:u1example",
  });
  assert.equal(view.ok, true);
  assert.equal(view.invoice.release, false);
  assert.equal(view.invoice.status, "open");
});

test("a confirmed invoice is the only release", () => {
  const view = invoiceView(200, {
    invoice_id: "inv_1",
    status: "confirmed",
    confirmed_txid: "abc",
  });
  assert.equal(view.invoice.release, true);
  assert.equal(view.invoice.confirmed_txid, "abc");
});

test("a missing invoice is not marked paid", () => {
  const view = invoiceView(404, { error: "Invoice inv_9 not found" });
  assert.equal(view.ok, false);
  assert.equal(view.invoice, undefined);
});

test("the companion key is sent as a header and read from the active profile", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "nozy-companion-"));
  fs.mkdirSync(path.join(root, "profiles", "wallet1"), { recursive: true });
  fs.writeFileSync(path.join(root, "profiles", "manifest.json"), JSON.stringify({ active_id: "wallet1" }));
  fs.writeFileSync(path.join(root, "profiles", "wallet1", "companion_api_key"), "local-key\n");
  const key = loadCompanionKey({ envKey: "", root });
  assert.equal(key, "local-key");
  const headers = companionHeaders(key, true);
  assert.equal(headers["X-API-Key"], "local-key");
  assert.equal(headers["content-type"], "application/json");
  fs.rmSync(root, { recursive: true, force: true });
});

test("a bad password is an error without echoing the password", () => {
  const view = unlockView(401, { error: "Invalid password" });
  assert.equal(view.ok, false);
  assert.match(view.reason, /password/i);
});
