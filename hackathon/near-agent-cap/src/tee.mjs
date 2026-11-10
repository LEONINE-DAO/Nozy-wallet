/** Gateway attestation checks. The API key is never returned. Intel DCAP is not run here. */

import crypto from "node:crypto";

export const ATTESTATION_URL = "https://cloud-api.near.ai/v1/attestation/report";

export function freshNonce(randomBytes = () => crypto.randomBytes(32)) {
  return Buffer.from(randomBytes()).toString("hex");
}

export function assessGatewayReport(report, nonce) {
  const gateway = report?.gateway_attestation ?? {};
  const expected = String(nonce ?? "").toLowerCase();
  const echoed = String(gateway.request_nonce ?? "").toLowerCase();
  const reportData = String(gateway.report_data ?? "").toLowerCase();
  const quote = String(gateway.intel_quote ?? "");
  const algo = String(gateway.signing_algo ?? "");
  const address = String(gateway.signing_address ?? "");
  const checks = [
    { name: "Fresh nonce", ok: expected.length === 64 && echoed === expected },
    { name: "Nonce bound in report data", ok: expected.length === 64 && reportData.includes(expected) },
    { name: "Signing algorithm", ok: algo === "ecdsa" || algo === "ed25519", detail: algo || null },
    { name: "Signing address", ok: address.length > 8, detail: address ? `${address.slice(0, 10)}…${address.slice(-4)}` : null },
    { name: "Intel quote present", ok: /^[0-9a-fA-F]+$/.test(quote) && quote.length >= 128 },
    { name: "Intel DCAP verification", ok: false, detail: "Not run on this desk" },
  ];
  const evidence = checks.filter((check) => check.name !== "Intel DCAP verification").every((check) => check.ok);
  return { evidence, verified: false, checks };
}

export async function fetchGatewayAttestation({ apiKey, fetchImpl = fetch, nonce = freshNonce() } = {}) {
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
      signal: AbortSignal.timeout(20000),
    });
  } catch {
    return { ok: false, evidence: false, verified: false, reason: "NEAR AI Cloud did not answer.", checks: [] };
  }
  const json = await response.json().catch(() => ({}));
  if (!response.ok) {
    const message = json?.error?.message || "NEAR AI Cloud refused the attestation request.";
    return { ok: false, evidence: false, verified: false, reason: String(message), checks: [] };
  }
  const assessed = assessGatewayReport(json, nonce);
  return { ok: true, ...assessed };
}
