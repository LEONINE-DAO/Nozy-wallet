export const MOBILE_SYNC_SCHEMA_VERSION = 2;
export const MOBILE_SYNC_PAIRING_TTL_MS = 10 * 60 * 1000;
export const MOBILE_SYNC_REPLAY_TTL_MS = 7 * 24 * 60 * 60 * 1000;
export const MOBILE_SYNC_MAX_REPLAY_ENTRIES = 128;

function toFiniteNumber(value, fallback = 0) {
  const n = Number(value);
  return Number.isFinite(n) ? n : fallback;
}

function normalizePairedDevices(devices, now) {
  if (!Array.isArray(devices)) return [];
  return devices
    .filter((d) => d && typeof d === "object" && typeof d.id === "string")
    .map((d) => ({
      id: String(d.id),
      name: String(d.name || "Nozy Mobile"),
      platform: String(d.platform || "unknown"),
      sessionId: String(d.sessionId || ""),
      signaturePrefix: String(d.signaturePrefix || ""),
      pairedAt: toFiniteNumber(d.pairedAt, now),
      status: d.status === "revoked" ? "revoked" : "paired",
      renamedAt: d.renamedAt ? toFiniteNumber(d.renamedAt, now) : null,
      revokedAt: d.revokedAt ? toFiniteNumber(d.revokedAt, now) : null,
      lastSeenAt: toFiniteNumber(d.lastSeenAt, toFiniteNumber(d.pairedAt, now)),
      trustLevel: String(d.trustLevel || "signed-challenge-v1")
    }));
}

function normalizeConsumedSessions(consumedSessions, now) {
  if (!Array.isArray(consumedSessions)) return [];
  return consumedSessions
    .filter((entry) => entry && typeof entry === "object" && typeof entry.sessionId === "string")
    .map((entry) => ({
      sessionId: String(entry.sessionId),
      consumedAt: toFiniteNumber(entry.consumedAt, now)
    }))
    .sort((a, b) => b.consumedAt - a.consumedAt);
}

export function sanitizeDeviceName(input) {
  const trimmed = String(input || "").trim().replace(/\s+/g, " ");
  if (!trimmed) return "Nozy Mobile";
  return trimmed.slice(0, 40);
}

export function migrateMobileSyncState(rawState, now = Date.now()) {
  const raw = rawState && typeof rawState === "object" ? rawState : {};
  const activePairing = raw.activePairing && typeof raw.activePairing === "object"
    ? {
        sessionId: String(raw.activePairing.sessionId || ""),
        walletAddress: String(raw.activePairing.walletAddress || ""),
        verifyCode: String(raw.activePairing.verifyCode || ""),
        challenge: String(raw.activePairing.challenge || ""),
        createdAt: toFiniteNumber(raw.activePairing.createdAt, now),
        expiresAt: toFiniteNumber(raw.activePairing.expiresAt, now + MOBILE_SYNC_PAIRING_TTL_MS)
      }
    : null;

  return {
    schemaVersion: MOBILE_SYNC_SCHEMA_VERSION,
    pairedDevices: normalizePairedDevices(raw.pairedDevices, now),
    activePairing,
    pairingPayload: typeof raw.pairingPayload === "string" ? raw.pairingPayload : null,
    consumedSessions: normalizeConsumedSessions(raw.consumedSessions, now),
    updatedAt: toFiniteNumber(raw.updatedAt, now)
  };
}

export function cleanupMobileSyncState(state, now = Date.now()) {
  const next = migrateMobileSyncState(state, now);
  let changed = false;

  if (next.activePairing && next.activePairing.expiresAt <= now) {
    next.activePairing = null;
    next.pairingPayload = null;
    changed = true;
  }

  const minConsumedAt = now - MOBILE_SYNC_REPLAY_TTL_MS;
  const keptSessions = next.consumedSessions
    .filter((entry) => entry.consumedAt >= minConsumedAt)
    .slice(0, MOBILE_SYNC_MAX_REPLAY_ENTRIES);
  if (keptSessions.length !== next.consumedSessions.length) {
    next.consumedSessions = keptSessions;
    changed = true;
  }

  if (changed) {
    next.updatedAt = now;
  }
  return { state: next, changed };
}

export function isSessionConsumed(state, sessionId, now = Date.now()) {
  const { state: cleaned } = cleanupMobileSyncState(state, now);
  return cleaned.consumedSessions.some((entry) => entry.sessionId === sessionId);
}

export function consumeSession(state, sessionId, now = Date.now()) {
  const session = String(sessionId || "");
  if (!session) return cleanupMobileSyncState(state, now).state;
  const { state: cleaned } = cleanupMobileSyncState(state, now);
  const withoutDup = cleaned.consumedSessions.filter((entry) => entry.sessionId !== session);
  const consumedSessions = [{ sessionId: session, consumedAt: now }, ...withoutDup].slice(
    0,
    MOBILE_SYNC_MAX_REPLAY_ENTRIES
  );
  return {
    ...cleaned,
    consumedSessions,
    updatedAt: now
  };
}
