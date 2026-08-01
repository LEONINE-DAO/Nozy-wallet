/**
 * Zcash Name Service (ZNS) client — resolve human-readable names to unified addresses.
 *
 * Docs: https://www.zcashnames.com/docs/zns-developer-guide
 *
 * In a Zcash address field, unsuffixed names are Zcash names. Optional `.zcash` / `.zec`
 * may be accepted but must not be required (and must not be the resolution trigger).
 */

export type ZnsNetwork = "mainnet" | "testnet";

export type ZnsRegistration = {
  name: string;
  address: string;
  txid?: string;
  height?: number;
  nonce?: number;
  last_action?: string;
};

const MAINNET_URL = "https://light.zcash.me/zns-mainnet-test";
const TESTNET_URL = "https://light.zcash.me/zns-testnet";

const UA_MIN_LEN = 78;
const UA_MAX_LEN = 256;

/** Optional display suffixes — strip if present; never require them. */
const OPTIONAL_SUFFIX_RE = /\.(zcash|zec)$/i;

/** Lowercase letters, digits, hyphens; 1–63 chars (ZNS name rules). */
const NAME_RE = /^[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?$/;

export function znsIndexerUrl(network: ZnsNetwork = "mainnet"): string {
  return network === "testnet" ? TESTNET_URL : MAINNET_URL;
}

export function normalizeUnifiedAddress(value: string): string {
  return value.replace(/\s+/g, "");
}

export function isUnifiedZcashAddress(value: string): boolean {
  const normalized = normalizeUnifiedAddress(value);
  if (!normalized.startsWith("u1") && !normalized.startsWith("utest1")) return false;
  return normalized.length >= UA_MIN_LEN && normalized.length <= UA_MAX_LEN;
}

/**
 * Normalize a typed recipient that may be a ZNS name.
 * Strips optional `.zcash` / `.zec` so `alice`, `alice.zcash`, and `alice.zec` are equivalent.
 */
export function normalizeZnsNameInput(raw: string): string {
  let s = raw.trim().toLowerCase();
  if (OPTIONAL_SUFFIX_RE.test(s)) {
    s = s.replace(OPTIONAL_SUFFIX_RE, "");
  }
  return s;
}

export function isLikelyZnsName(raw: string): boolean {
  const trimmed = raw.trim();
  if (!trimmed) return false;
  if (isUnifiedZcashAddress(trimmed)) return false;
  // Partial / full UA typing should not be treated as a name.
  const compact = normalizeUnifiedAddress(trimmed);
  if (compact.startsWith("u1") || compact.startsWith("utest1")) return false;
  const name = normalizeZnsNameInput(trimmed);
  return NAME_RE.test(name);
}

export type ResolveRecipientResult =
  | { kind: "address"; address: string }
  | { kind: "name"; name: string; registration: ZnsRegistration }
  | { kind: "unresolved"; name: string }
  | { kind: "invalid"; message: string };

type JsonRpcResponse = {
  jsonrpc?: string;
  id?: number | string;
  result?: ZnsRegistration | null;
  error?: { code?: number; message?: string };
};

export async function resolveZnsName(
  rawName: string,
  network: ZnsNetwork = "mainnet",
  options?: { signal?: AbortSignal; url?: string },
): Promise<ZnsRegistration | null> {
  const name = normalizeZnsNameInput(rawName);
  if (!NAME_RE.test(name)) {
    throw new Error("Invalid Zcash name.");
  }
  const url = options?.url ?? znsIndexerUrl(network);
  const res = await fetch(url, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "resolve",
      params: [name],
    }),
    signal: options?.signal,
  });
  if (!res.ok) {
    throw new Error(`ZNS indexer HTTP ${res.status}`);
  }
  const data = (await res.json()) as JsonRpcResponse;
  if (data.error?.message) {
    throw new Error(data.error.message);
  }
  const reg = data.result;
  if (!reg || typeof reg !== "object" || typeof reg.address !== "string") {
    return null;
  }
  return {
    name: typeof reg.name === "string" ? reg.name : name,
    address: reg.address,
    txid: typeof reg.txid === "string" ? reg.txid : undefined,
    height: typeof reg.height === "number" ? reg.height : undefined,
    nonce: typeof reg.nonce === "number" ? reg.nonce : undefined,
    last_action: typeof reg.last_action === "string" ? reg.last_action : undefined,
  };
}

/**
 * Resolve a send-form recipient on commit (amount focus, review, or submit).
 * Unified addresses pass through; ZNS names are looked up. Does not run on keystroke.
 */
export async function resolveSendRecipient(
  raw: string,
  network: ZnsNetwork = "mainnet",
  options?: { signal?: AbortSignal; url?: string },
): Promise<ResolveRecipientResult> {
  const trimmed = raw.trim();
  if (!trimmed) {
    return { kind: "invalid", message: "Enter a recipient address or Zcash name." };
  }
  if (isUnifiedZcashAddress(trimmed)) {
    return { kind: "address", address: normalizeUnifiedAddress(trimmed) };
  }
  const compact = normalizeUnifiedAddress(trimmed);
  if (compact.startsWith("u1") || compact.startsWith("utest1")) {
    return {
      kind: "invalid",
      message:
        "Address must be an Orchard unified address (u1… or utest1…) at least 78 characters.",
    };
  }
  if (!isLikelyZnsName(trimmed)) {
    return {
      kind: "invalid",
      message: "Enter an Orchard unified address (u1…) or a Zcash name (e.g. alice).",
    };
  }
  const name = normalizeZnsNameInput(trimmed);
  const registration = await resolveZnsName(name, network, options);
  if (!registration) {
    return { kind: "unresolved", name };
  }
  return { kind: "name", name: registration.name || name, registration };
}
