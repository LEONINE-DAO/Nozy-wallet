import test from "node:test";
import assert from "node:assert/strict";
import { assessGatewayReport, freshNonce } from "../src/tee.mjs";

test("a matching nonce is evidence and is not called DCAP-verified", () => {
  const nonce = "ab".repeat(32);
  const report = {
    gateway_attestation: {
      request_nonce: nonce.toUpperCase(),
      report_data: `00${nonce}ff`,
      signing_algo: "ecdsa",
      signing_address: "0x1234567890abcdef",
      intel_quote: "aa".repeat(80),
    },
  };
  const result = assessGatewayReport(report, nonce);
  assert.equal(result.evidence, true);
  assert.equal(result.verified, false);
  assert.equal(result.checks.find((check) => check.name === "Intel DCAP verification").ok, false);
});

test("a mismatched nonce is not evidence", () => {
  const result = assessGatewayReport({
    gateway_attestation: {
      request_nonce: "11".repeat(32),
      report_data: "11".repeat(32),
      signing_algo: "ecdsa",
      signing_address: "0x1234567890abcdef",
      intel_quote: "aa".repeat(80),
    },
  }, "22".repeat(32));
  assert.equal(result.evidence, false);
  assert.equal(freshNonce(() => Buffer.from("33".repeat(32), "hex")).length, 64);
});
