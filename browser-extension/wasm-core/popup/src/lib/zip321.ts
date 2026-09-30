/**
 * ZIP-321 payment URIs (payer-side).
 * https://zips.z.cash/zip-0321
 *
 * Paste or deep-link a `zcash:` URI. Do not call ZGo, NozyPay, or any merchant HTTP API
 * from this module — the URI is the whole payment request.
 *
 * Nozy overlay: Unified Addresses only (u1 / utest1 / uregtest1). Rejects t-addrs,
 * Sprout, Sapling-only, unknown `req-` parameters, and multi-recipient requests.
 */

const MAX_ZEC = 21_000_000;
const MAX_MEMO_BYTES = 512;
const B64URL = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

export type Zip321Payment = {
  address: string;
  amountZec?: string;
  memo?: string;
  message?: string;
  label?: string;
};

export class Zip321Error extends Error {
  constructor(message: string) {
    super(message);
    this.name = "Zip321Error";
  }
}

export function looksLikeZgoPage(raw: string): boolean {
  const s = raw.trim().toLowerCase();
  return (s.startsWith("http://") || s.startsWith("https://")) && s.includes("zgo.cash");
}

export function extractZip321Uri(raw: string): string | null {
  const s = raw.trim();
  if (looksLikeZgoPage(s) && !/zcash:/i.test(s)) return null;
  const lower = s.toLowerCase();
  const idx = lower.indexOf("zcash:");
  if (idx < 0) return null;
  const rest = s.slice(idx);
  const end = rest.search(/[\s"<>]/);
  const cut = end < 0 ? rest : rest.slice(0, end);
  return cut.replace(/[.,;)]+$/, "");
}

/** Parse pasted Send input. Returns null when the text is a normal address/name. */
export function tryParsePaymentInput(raw: string): Zip321Payment | null {
  const s = raw.trim();
  if (!s) return null;
  if (looksLikeZgoPage(s) && !/zcash:/i.test(s)) {
    throw new Zip321Error(
      "Paste the zcash: payment URI from the QR, not the ZGo page. Nozy does not call ZGo’s API.",
    );
  }
  if (!/zcash:/i.test(s)) return null;
  return parseZip321Uri(s);
}

export function buildZip321Uri(
  address: string,
  amountZec?: number,
  memo?: string,
): string {
  const addr = address.trim();
  if (!addr) throw new Zip321Error("ZIP-321 URI requires a payment address");
  assertShieldedUa(addr);
  if (!isAddressToken(addr)) {
    throw new Zip321Error("ZIP-321 address may only contain letters and digits");
  }
  const params: string[] = [];
  if (amountZec != null) params.push(`amount=${formatAmount(amountZec)}`);
  const m = memo?.trim();
  if (m) {
    const bytes = utf8Bytes(m);
    if (bytes.length > MAX_MEMO_BYTES) {
      throw new Zip321Error("ZIP-321 memo exceeds 512 bytes");
    }
    params.push(`memo=${base64urlEncode(bytes)}`);
  }
  return params.length ? `zcash:${addr}?${params.join("&")}` : `zcash:${addr}`;
}

export function parseZip321Uri(raw: string): Zip321Payment {
  const extracted = extractZip321Uri(raw);
  if (!extracted) {
    throw new Zip321Error(
      "Not a zcash: payment URI. Paste the ZIP-321 URI from the QR, not a zgo.cash page URL.",
    );
  }
  const rest = stripScheme(extracted);
  if (rest == null) throw new Zip321Error("Not a zcash: payment URI");
  if (rest.startsWith("//")) {
    throw new Zip321Error("ZIP-321 URIs must not use // (not a hierarchical URI)");
  }

  const qIdx = rest.indexOf("?");
  const hier = (qIdx < 0 ? rest : rest.slice(0, qIdx)).trim();
  const query = qIdx < 0 ? undefined : rest.slice(qIdx + 1);
  if (hier.includes("%")) {
    throw new Zip321Error("ZIP-321 address must not be percent-encoded");
  }
  if (hier && !isAddressToken(hier)) {
    throw new Zip321Error("ZIP-321 address may only contain letters and digits");
  }

  let queryAddress: string | undefined;
  let amountZec: string | undefined;
  let memo: string | undefined;
  let message: string | undefined;
  let label: string | undefined;
  const seen = new Set<string>();

  if (query !== undefined) {
    if (!query) throw new Zip321Error("ZIP-321 URI has an empty query");
    for (const pair of query.split("&")) {
      if (!pair) continue;
      const eq = pair.indexOf("=");
      const rawKey = eq < 0 ? pair : pair.slice(0, eq);
      const rawVal = eq < 0 ? undefined : pair.slice(eq + 1);
      if (rawKey.includes("%")) {
        throw new Zip321Error("ZIP-321 parameter names must not be percent-encoded");
      }
      const { name, index } = splitNameIndex(rawKey);
      if (index) {
        throw new Zip321Error("Multi-recipient ZIP-321 requests are not supported");
      }
      const seenKey = `${name}.${index}`;
      if (seen.has(seenKey)) {
        throw new Zip321Error(`Duplicate ZIP-321 parameter \`${rawKey}\``);
      }
      seen.add(seenKey);
      if (name.startsWith("req-")) {
        throw new Zip321Error(`Unsupported required ZIP-321 parameter \`${name}\``);
      }
      switch (name) {
        case "address": {
          if (rawVal == null) throw new Zip321Error("ZIP-321 address= is missing a value");
          if (rawVal.includes("%")) {
            throw new Zip321Error("ZIP-321 address must not be percent-encoded");
          }
          if (!isAddressToken(rawVal)) {
            throw new Zip321Error("ZIP-321 address may only contain letters and digits");
          }
          queryAddress = rawVal;
          break;
        }
        case "amount": {
          if (rawVal == null) throw new Zip321Error("ZIP-321 amount= is missing a value");
          if (rawVal.includes("%")) {
            throw new Zip321Error("ZIP-321 amount must not be percent-encoded");
          }
          amountZec = formatParsedAmount(parseAmount(rawVal));
          break;
        }
        case "memo":
          memo = decodeMemo(rawVal ?? "");
          break;
        case "message":
          message = percentDecode(rawVal ?? "");
          break;
        case "label":
          label = percentDecode(rawVal ?? "");
          break;
        default:
          break;
      }
    }
  }

  if (hier && queryAddress) {
    throw new Zip321Error("ZIP-321 address specified twice (path and address=)");
  }
  const address = hier || queryAddress || "";
  if (!address) throw new Zip321Error("zcash: URI missing address");
  assertShieldedUa(address);
  return { address, amountZec, memo, message, label };
}

function stripScheme(s: string): string | null {
  return s.toLowerCase().startsWith("zcash:") ? s.slice(6) : null;
}

function splitNameIndex(rawKey: string): { name: string; index: string } {
  const dot = rawKey.indexOf(".");
  if (dot < 0) {
    if (!isParamName(rawKey)) throw new Zip321Error(`Invalid ZIP-321 parameter \`${rawKey}\``);
    return { name: rawKey.toLowerCase(), index: "" };
  }
  const name = rawKey.slice(0, dot);
  const index = rawKey.slice(dot + 1);
  if (!name || !isParamName(name)) throw new Zip321Error(`Invalid ZIP-321 parameter \`${rawKey}\``);
  if (!isValidParamIndex(index)) {
    throw new Zip321Error(`Invalid ZIP-321 parameter index in \`${rawKey}\``);
  }
  return { name: name.toLowerCase(), index };
}

function isParamName(s: string): boolean {
  return /^[A-Za-z][A-Za-z0-9+-]*$/.test(s);
}

function isValidParamIndex(s: string): boolean {
  return /^[1-9][0-9]{0,3}$/.test(s);
}

function isAddressToken(s: string): boolean {
  return s.length > 0 && /^[A-Za-z0-9]+$/.test(s);
}

function assertShieldedUa(addr: string): void {
  const a = addr.toLowerCase();
  if (/^t[123m]/.test(a)) {
    throw new Zip321Error("Transparent ZEC payment URIs are not supported");
  }
  if (a.startsWith("zc")) {
    throw new Zip321Error("Sprout addresses are not supported in payment requests");
  }
  if (a.startsWith("zs1") || a.startsWith("ztestsapling") || a.startsWith("zregtestsapling")) {
    throw new Zip321Error(
      "Nozy pays Unified Addresses only (u1…). Sapling-only payment requests are not supported.",
    );
  }
  if (!(a.startsWith("u1") || a.startsWith("utest1") || a.startsWith("uregtest1"))) {
    throw new Zip321Error("Payment URI must use a shielded Unified Address (u1 / utest1)");
  }
}

function formatAmount(amt: number): string {
  if (!Number.isFinite(amt) || amt <= 0 || amt > MAX_ZEC) {
    throw new Zip321Error("ZIP-321 amount must be a positive ZEC value of at most 21000000");
  }
  return amt.toFixed(8).replace(/\.?0+$/, "");
}

function formatParsedAmount(amt: number): string {
  return String(amt);
}

function parseAmount(val: string): number {
  if (!val || val.startsWith(".") || val.endsWith(".") || val.includes(",") || (val.match(/\./g) || []).length > 1) {
    throw new Zip321Error(`Invalid ZIP-321 amount: ${val}`);
  }
  const [whole, frac] = val.split(".");
  if (!whole || !/^[0-9]+$/.test(whole)) throw new Zip321Error(`Invalid ZIP-321 amount: ${val}`);
  if (frac != null && (!frac || frac.length > 8 || !/^[0-9]+$/.test(frac))) {
    throw new Zip321Error(`Invalid ZIP-321 amount: ${val}`);
  }
  const amt = Number(val);
  if (!Number.isFinite(amt) || amt < 0 || amt > MAX_ZEC) {
    throw new Zip321Error(`ZIP-321 amount out of range: ${val}`);
  }
  return amt;
}

function decodeMemo(val: string): string {
  if (/[^A-Za-z0-9_-]/.test(val)) {
    throw new Zip321Error("ZIP-321 memo must be unpadded base64url");
  }
  const bytes = base64urlDecode(val);
  if (bytes.length > MAX_MEMO_BYTES) {
    throw new Zip321Error("ZIP-321 memo exceeds 512 bytes");
  }
  let end = bytes.length;
  while (end > 0 && bytes[end - 1] === 0) end -= 1;
  return utf8Decode(bytes.slice(0, end));
}

function utf8Bytes(s: string): Uint8Array {
  return new TextEncoder().encode(s);
}

function utf8Decode(bytes: Uint8Array): string {
  return new TextDecoder("utf-8", { fatal: false }).decode(bytes);
}

function base64urlEncode(input: Uint8Array): string {
  let out = "";
  for (let i = 0; i < input.length; i += 3) {
    const b0 = input[i];
    const b1 = i + 1 < input.length ? input[i + 1] : 0;
    const b2 = i + 2 < input.length ? input[i + 2] : 0;
    const n = (b0 << 16) | (b1 << 8) | b2;
    out += B64URL[(n >> 18) & 63];
    out += B64URL[(n >> 12) & 63];
    if (i + 1 < input.length) out += B64URL[(n >> 6) & 63];
    if (i + 2 < input.length) out += B64URL[n & 63];
  }
  return out;
}

function base64urlDecode(s: string): Uint8Array {
  if (/[+/=]/.test(s)) {
    throw new Zip321Error("ZIP-321 memo must be unpadded base64url (no + / =)");
  }
  const vals: number[] = [];
  for (const c of s) {
    const v = B64URL.indexOf(c);
    if (v < 0) throw new Zip321Error("ZIP-321 memo must be unpadded base64url");
    vals.push(v);
  }
  const out: number[] = [];
  for (let i = 0; i < vals.length; i += 4) {
    const v0 = vals[i];
    const v1 = i + 1 < vals.length ? vals[i + 1] : 0;
    const v2 = i + 2 < vals.length ? vals[i + 2] : 0;
    const v3 = i + 3 < vals.length ? vals[i + 3] : 0;
    const n = (v0 << 18) | (v1 << 12) | (v2 << 6) | v3;
    if (i + 1 < vals.length) out.push((n >> 16) & 0xff);
    if (i + 2 < vals.length) out.push((n >> 8) & 0xff);
    if (i + 3 < vals.length) out.push(n & 0xff);
  }
  return Uint8Array.from(out);
}

function percentDecode(s: string): string {
  const out: number[] = [];
  for (let i = 0; i < s.length; i++) {
    if (s[i] === "%" && i + 2 < s.length) {
      const h = parseInt(s.slice(i + 1, i + 3), 16);
      if (!Number.isNaN(h)) {
        out.push(h);
        i += 2;
        continue;
      }
    }
    out.push(s[i] === "+" ? 0x20 : s.charCodeAt(i));
  }
  return utf8Decode(Uint8Array.from(out));
}

export function amountZecToZats(amountZec: string | undefined): string | undefined {
  if (amountZec == null || amountZec === "") return undefined;
  const n = Number(amountZec);
  if (!Number.isFinite(n) || n < 0) return undefined;
  return String(Math.round(n * 1e8));
}
