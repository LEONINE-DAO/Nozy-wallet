import test from "node:test";
import assert from "node:assert/strict";
import { scanStatusDefersAutoLock } from "./session-lock.js";

test("catch-up scan defers idle auto-lock", () => {
  assert.equal(scanStatusDefersAutoLock("scanning"), true);
});

test("tip-follow and finished scans still auto-lock", () => {
  assert.equal(scanStatusDefersAutoLock("watching"), false);
  assert.equal(scanStatusDefersAutoLock("done"), false);
  assert.equal(scanStatusDefersAutoLock("stopped"), false);
  assert.equal(scanStatusDefersAutoLock("failed"), false);
  assert.equal(scanStatusDefersAutoLock("idle"), false);
  assert.equal(scanStatusDefersAutoLock(null), false);
  assert.equal(scanStatusDefersAutoLock(undefined), false);
});
