/** Gateway attestation. The API key and the raw quote are never returned. */

import crypto from "node:crypto";
import { getCollateralAndVerify } from "@phala/dcap-qvl";

export const ATTESTATION_URL = "https://cloud-api.near.ai/v1/attestation/report";

/** NEAR's sample policy. A quote with any other TCB status stays unverified. */
export const ACCEPTED_TCB_STATUSES = ["UpToDate"];

const IDENTITY_LENGTH = { ecdsa: 20, ed25519: 32 };

export function freshNonce(randomBytes = () => crypto.randomBytes(32)) {
  return Buffer.from(randomBytes()).toString("hex");
}

function hexBytes(value) {
  const text = String(value ?? "").replace(/^0x/i, "");
  if (text.length % 2 !== 0 || !/^[0-9a-fA-F]*$/.test(text)) return null;
  return Buffer.from(text, "hex");
}

function parseJson(value) {
  return typeof value === "string" ? JSON.parse(value) : value;
}

function hash(algorithm, ...parts) {
  const digest = crypto.createHash(algorithm);
  for (const part of parts) digest.update(part);
  return digest.digest();
}

function check(name, ok, detail = null) {
  return { name, ok, detail };
}

function td10(report) {
  if (report?.type === "td10") return report.data;
  if (report?.type === "td15") return report.data?.base ?? null;
  if (report?.type === "td15ex") return report.data?.base?.base ?? null;
  return null;
}

/** Replay RTMR3 the way NEAR documents it. Events whose imr is not 3 are skipped. */
export function replayRtmr3(eventLog) {
  let rtmr3 = Buffer.alloc(48);
  for (const event of parseJson(eventLog)) {
    if (event?.imr !== 3) continue;
    const eventType = Buffer.alloc(4);
    eventType.writeUInt32LE(event.event_type);
    const payload = hexBytes(event.event_payload ?? "");
    if (!payload) throw new Error("event payload is not hex");
    const digest = hash(
      "sha384",
      eventType,
      ":",
      Buffer.from(String(event.event), "utf8"),
      ":",
      payload,
    );
    if (event.digest && !hexBytes(event.digest)?.equals(digest)) {
      throw new Error("event digest mismatch");
    }
    rtmr3 = hash("sha384", rtmr3, digest);
  }
  return rtmr3;
}

export function expectedMrConfigId(appCompose) {
  return Buffer.concat([
    Buffer.from([0x01]),
    hash("sha256", Buffer.from(String(appCompose), "utf8")),
    Buffer.alloc(15),
  ]);
}

function boundIdentity(report) {
  const algo = report?.signing_algo;
  const identity = hexBytes(report?.signing_address);
  if (!identity || IDENTITY_LENGTH[algo] !== identity.length) return null;
  const fingerprint = report?.tls_cert_fingerprint;
  if (fingerprint) {
    const fingerprintBytes = hexBytes(fingerprint);
    if (!fingerprintBytes) return null;
    return hash("sha256", identity, fingerprintBytes);
  }
  return Buffer.concat([identity, Buffer.alloc(32 - identity.length)]);
}

function appCompose(report) {
  const info = parseJson(report?.info?.tcb_info);
  const compose = info?.app_compose;
  return typeof compose === "string" ? compose : null;
}

/**
 * Verify one NEAR gateway report.
 * `verifyQuote` defaults to Intel DCAP collateral plus quote verification.
 */
export async function verifyNearReport(report, nonceHex, options = {}) {
  const verifyQuote = options.verifyQuote ?? ((quote) => getCollateralAndVerify(quote));
  const accepted = options.acceptedTcbStatuses ?? ACCEPTED_TCB_STATUSES;
  const quoteBytes = hexBytes(report?.intel_quote);
  if (!quoteBytes || quoteBytes.length < 64) {
    return { evidence: false, verified: false, checks: [check("Intel DCAP verification", false, "Quote is missing")] };
  }

  let verifiedQuote;
  try {
    verifiedQuote = await verifyQuote(quoteBytes);
  } catch {
    return { evidence: false, verified: false, checks: [check("Intel DCAP verification", false, "Quote verification failed")] };
  }

  const quote = td10(verifiedQuote?.report);
  const status = String(verifiedQuote?.status ?? "");
  const checks = [
    check("Intel DCAP verification", accepted.includes(status), status || "Unknown"),
    check("TDX debug mode", Boolean(quote) && (quote.tdAttributes?.[0] & 1) === 0, quote ? "Off" : "Not a TDX quote"),
  ];
  if (!quote) {
    return { evidence: false, verified: false, checks };
  }

  const nonce = hexBytes(nonceHex);
  const identity = boundIdentity(report);
  const expectedReport = nonce && identity ? Buffer.concat([identity, nonce]) : null;
  const actualReport = Buffer.from(quote.reportData ?? []);
  const echoedNonce = String(report?.request_nonce ?? "").toLowerCase();
  const echoedReport = hexBytes(report?.report_data);
  checks.push(
    check("Fresh nonce", Boolean(nonce) && nonce.length === 32 && echoedNonce === nonce.toString("hex")),
    check("Quote binds signer and nonce", Boolean(expectedReport) && actualReport.equals(expectedReport)),
    check("Echoed report data", Boolean(echoedReport) && echoedReport.equals(actualReport)),
    check(
      "Signing algorithm",
      report?.signing_algo === "ecdsa" || report?.signing_algo === "ed25519",
      report?.signing_algo || null,
    ),
  );
  const address = String(report?.signing_address ?? "");
  checks.push(check("Signing address", address.length > 8, address ? `${address.slice(0, 10)}…${address.slice(-4)}` : null));

  const compose = (() => {
    try {
      return appCompose(report);
    } catch {
      return null;
    }
  })();
  const configOk = Boolean(compose) && Buffer.from(quote.mrConfigId ?? []).equals(expectedMrConfigId(compose));
  checks.push(check("Configuration measurement", configOk));

  let eventOk = false;
  let eventDetail = null;
  try {
    eventOk = replayRtmr3(report?.event_log).equals(Buffer.from(quote.rtMr3 ?? []));
    if (!eventOk) eventDetail = "Does not match the quote";
  } catch (error) {
    eventDetail = error instanceof Error ? error.message : "Event log failed";
  }
  checks.push(check("Event log matches RTMR3", eventOk, eventDetail));

  const verified = checks.every((item) => item.ok);
  return { evidence: verified, verified, checks };
}

export async function fetchGatewayAttestation({ apiKey, fetchImpl = fetch, nonce = freshNonce(), verifyQuote } = {}) {
  const key = String(apiKey ?? "").trim();
  if (!key) {
    return { ok: false, evidence: false, verified: false, reason: "NEAR AI is not connected on this machine.", checks: [] };
  }
  const url = new URL(ATTESTATION_URL);
  url.searchParams.set("signing_algo", "ecdsa");
  url.searchParams.set("nonce", nonce);
  url.searchParams.set("provider", "near");
  let response;
  try {
    response = await fetchImpl(url, {
      headers: { accept: "application/json", authorization: `Bearer ${key}` },
      signal: AbortSignal.timeout(30000),
    });
  } catch {
    return { ok: false, evidence: false, verified: false, reason: "NEAR AI Cloud did not answer.", checks: [] };
  }
  const json = await response.json().catch(() => ({}));
  if (!response.ok) {
    const message = json?.error?.message || "NEAR AI Cloud refused the attestation request.";
    return { ok: false, evidence: false, verified: false, reason: String(message), checks: [] };
  }
  const assessed = await verifyNearReport(json.gateway_attestation, nonce, { verifyQuote });
  return { ok: true, ...assessed };
}
