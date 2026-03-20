import test from "node:test";
import assert from "node:assert/strict";
import {
  cleanupMobileSyncState,
  consumeSession,
  isSessionConsumed,
  migrateMobileSyncState,
  MOBILE_SYNC_SCHEMA_VERSION,
  sanitizeDeviceName
} from "./mobile-sync.js";

test("migrateMobileSyncState upgrades v1-ish payloads to schema v2", () => {
  const now = 1_700_000_000_000;
  const migrated = migrateMobileSyncState(
    {
      schemaVersion: 1,
      pairedDevices: [{ id: "d1", name: "Phone", platform: "ios", sessionId: "s1", pairedAt: now - 50 }],
      activePairing: { sessionId: "s2", challenge: "abc", verifyCode: "AAA", expiresAt: now + 1000 }
    },
    now
  );

  assert.equal(migrated.schemaVersion, MOBILE_SYNC_SCHEMA_VERSION);
  assert.equal(migrated.pairedDevices.length, 1);
  assert.equal(migrated.pairedDevices[0].status, "paired");
  assert.equal(migrated.consumedSessions.length, 0);
  assert.equal(migrated.activePairing?.sessionId, "s2");
});

test("cleanupMobileSyncState clears expired active pairing", () => {
  const now = 1_700_000_100_000;
  const { state, changed } = cleanupMobileSyncState(
    {
      schemaVersion: 2,
      pairedDevices: [],
      activePairing: {
        sessionId: "expired",
        walletAddress: "u1abc",
        verifyCode: "ABC123",
        challenge: "deadbeef",
        createdAt: now - 10_000,
        expiresAt: now - 1
      },
      pairingPayload: "{\"sessionId\":\"expired\"}",
      consumedSessions: [],
      updatedAt: now - 10_000
    },
    now
  );

  assert.equal(changed, true);
  assert.equal(state.activePairing, null);
  assert.equal(state.pairingPayload, null);
});

test("consumeSession adds replay guard entries and isSessionConsumed detects them", () => {
  const now = 1_700_000_200_000;
  const state = consumeSession({ schemaVersion: 2 }, "ms_test", now);
  assert.equal(isSessionConsumed(state, "ms_test", now + 1), true);
  assert.equal(isSessionConsumed(state, "ms_other", now + 1), false);
});

test("sanitizeDeviceName trims and limits length", () => {
  assert.equal(sanitizeDeviceName("   My   Device   "), "My Device");
  assert.equal(sanitizeDeviceName(""), "Nozy Mobile");
  const shortened = sanitizeDeviceName("A very long mobile device name that should be cut down");
  assert.equal(shortened.length, 40);
});
