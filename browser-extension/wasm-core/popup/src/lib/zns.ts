/**
 * Zcash Name Service (ZNS) — name normalization + companion-proxied resolve.
 *
 * Suffixes `.zcash` / `.zec` are optional and must not be required.
 * Resolve on commit (amount focus / submit), not on every keystroke.
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

const UA_MIN_LEN = 78;
const UA_MAX_LEN = 256;
const OPTIONAL_SUFFIX_RE = /\.(zcash|zec)$/i;
const NAME_RE = /^[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?$/;

export function normalizeUnifiedAddress(value: string): string {
  return value.replace(/\s+/g, "");
}

export function isUnifiedZcashAddress(value: string): boolean {
  const normalized = normalizeUnifiedAddress(value);
  if (!normalized.startsWith("u1") && !normalized.startsWith("utest1")) return false;
  return normalized.length >= UA_MIN_LEN && normalized.length <= UA_MAX_LEN;
}

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
  const compact = normalizeUnifiedAddress(trimmed);
  if (compact.startsWith("u1") || compact.startsWith("utest1")) return false;
  return NAME_RE.test(normalizeZnsNameInput(trimmed));
}

export type ResolveRecipientResult =
  | { kind: "address"; address: string }
  | { kind: "name"; name: string; registration: ZnsRegistration }
  | { kind: "unresolved"; name: string }
  | { kind: "invalid"; message: string };

type ResolveViaCompanion = (
  name: string,
  network?: ZnsNetwork
) => Promise<ZnsRegistration | null>;

/**
 * Resolve a send recipient on commit. Prefer companion `/api/zns/resolve`
 * so the extension stays on localhost host permissions.
 */
export async function resolveSendRecipient(
  raw: string,
  resolveName: ResolveViaCompanion,
  network: ZnsNetwork = "mainnet"
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
      message: "Use a shielded unified address (u1…) or a Zcash name.",
    };
  }
  if (!isLikelyZnsName(trimmed)) {
    return {
      kind: "invalid",
      message: "Enter a unified address (u1…) or a Zcash name (e.g. alice).",
    };
  }
  const name = normalizeZnsNameInput(trimmed);
  const registration = await resolveName(name, network);
  if (!registration) {
    return { kind: "unresolved", name };
  }
  return { kind: "name", name: registration.name || name, registration };
}
