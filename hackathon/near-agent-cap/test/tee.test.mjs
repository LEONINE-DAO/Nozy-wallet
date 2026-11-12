import test from "node:test";
import assert from "node:assert/strict";
import crypto from "node:crypto";
import { expectedMrConfigId, freshNonce, replayRtmr3, verifyNearReport } from "../src/tee.mjs";

function sha384(...parts) {
  const digest = crypto.createHash("sha384");
  for (const part of parts) digest.update(part);
  return digest.digest();
}

function eventDigest(event) {
  const eventType = Buffer.alloc(4);
  eventType.writeUInt32LE(event.event_type);
  return sha384(eventType, ":", Buffer.from(event.event, "utf8"), ":", Buffer.from(event.event_payload ?? "", "hex"));
}

function acceptedQuote(reportData, mrConfigId, rtMr3) {
  return {
    status: "UpToDate",
    advisory_ids: [],
    report: {
      type: "td10",
      data: {
        tdAttributes: Uint8Array.from([0]),
        reportData,
        mrConfigId,
        rtMr3,
      },
    },
  };
}

function sample() {
  const nonce = "ab".repeat(32);
  const address = "12".repeat(20);
  const identity = Buffer.concat([Buffer.from(address, "hex"), Buffer.alloc(12)]);
  const reportData = Buffer.concat([identity, Buffer.from(nonce, "hex")]);
  const appCompose = "compose-v1";
  const event = { imr: 3, event_type: 7, event: "app-id", event_payload: "aa" };
  const rtMr3 = sha384(Buffer.alloc(48), eventDigest(event));
  const report = {
    intel_quote: "11".repeat(64),
    request_nonce: nonce.toUpperCase(),
    report_data: reportData.toString("hex"),
    signing_algo: "ecdsa",
    signing_address: `0x${address}`,
    event_log: JSON.stringify([event, { imr: 0, event_type: 1, event: "ignored", event_payload: "" }]),
    info: { tcb_info: JSON.stringify({ app_compose: appCompose }) },
  };
  return { nonce, report, reportData, rtMr3, mrConfigId: expectedMrConfigId(appCompose) };
}

test("RTMR3 replay extends only imr 3 events", () => {
  const event = { imr: 3, event_type: 7, event: "app-id", event_payload: "aa" };
  const replayed = replayRtmr3([event, { imr: 1, event_type: 1, event: "other", event_payload: "" }]);
  assert.ok(replayed.equals(sha384(Buffer.alloc(48), eventDigest(event))));
});

test("an UpToDate quote with matching bindings is verified", async () => {
  const { nonce, report, reportData, rtMr3, mrConfigId } = sample();
  const result = await verifyNearReport(report, nonce, {
    verifyQuote: async () => acceptedQuote(reportData, mrConfigId, rtMr3),
  });
  assert.equal(result.verified, true);
  assert.equal(result.evidence, true);
  assert.equal(result.checks.find((item) => item.name === "Intel DCAP verification").detail, "UpToDate");
});

test("a quote the verifier rejects is not verified", async () => {
  const { nonce, report, reportData, rtMr3, mrConfigId } = sample();
  const result = await verifyNearReport(report, nonce, {
    verifyQuote: async () => ({ ...acceptedQuote(reportData, mrConfigId, rtMr3), status: "OutOfDate" }),
  });
  assert.equal(result.verified, false);
  assert.equal(result.checks.find((item) => item.name === "Intel DCAP verification").ok, false);
});

test("debug mode keeps the quote unverified", async () => {
  const { nonce, report, reportData, rtMr3, mrConfigId } = sample();
  const quote = acceptedQuote(reportData, mrConfigId, rtMr3);
  quote.report.data.tdAttributes = Uint8Array.from([1]);
  const result = await verifyNearReport(report, nonce, { verifyQuote: async () => quote });
  assert.equal(result.verified, false);
  assert.equal(result.checks.find((item) => item.name === "TDX debug mode").ok, false);
});

test("a mismatched nonce is not evidence", async () => {
  const { report, reportData, rtMr3, mrConfigId } = sample();
  const result = await verifyNearReport(report, "22".repeat(32), {
    verifyQuote: async () => acceptedQuote(reportData, mrConfigId, rtMr3),
  });
  assert.equal(result.evidence, false);
  assert.equal(result.verified, false);
  assert.equal(freshNonce(() => Buffer.from("33".repeat(32), "hex")).length, 64);
});
