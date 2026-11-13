import { describe, expect, it } from "vitest";
import {
  classifyRecipientInput,
  isLikelyZnsName,
  normalizeZnsNameInput
} from "./zns";

describe("normalizeZnsNameInput", () => {
  it("strips optional .zec and .zcash suffixes", () => {
    expect(normalizeZnsNameInput("nozywallet.zec")).toBe("nozywallet");
    expect(normalizeZnsNameInput("NozyWallet.zcash")).toBe("nozywallet");
    expect(normalizeZnsNameInput("alice")).toBe("alice");
  });
});

describe("isLikelyZnsName", () => {
  it("accepts bare names and optional suffixes", () => {
    expect(isLikelyZnsName("nozywallet")).toBe(true);
    expect(isLikelyZnsName("nozywallet.zec")).toBe(true);
    expect(isLikelyZnsName("nozywallet.zcash")).toBe(true);
  });

  it("rejects incomplete unified addresses", () => {
    expect(isLikelyZnsName("u1abc")).toBe(false);
  });
});

describe("classifyRecipientInput", () => {
  const UA =
    "u1" + "a".repeat(76);

  it("marks empty input", () => {
    expect(classifyRecipientInput("").kind).toBe("empty");
    expect(classifyRecipientInput("   ").kind).toBe("empty");
  });

  it("marks a full unified address as valid", () => {
    const r = classifyRecipientInput(UA);
    expect(r).toEqual({ kind: "valid_address", address: UA });
  });

  it("marks incomplete UA as invalid", () => {
    const r = classifyRecipientInput("u1short");
    expect(r.kind).toBe("invalid");
    if (r.kind === "invalid") {
      expect(r.message).toMatch(/Incomplete or invalid/i);
    }
  });

  it("classifies nozywallet.zec as a likely name", () => {
    const r = classifyRecipientInput("nozywallet.zec");
    expect(r).toEqual({
      kind: "likely_name",
      name: "nozywallet",
      display: "nozywallet.zec"
    });
  });

  it("classifies garbage as invalid", () => {
    const r = classifyRecipientInput("not an address!!!");
    expect(r.kind).toBe("invalid");
  });

  it("rejects testnet UA on mainnet", () => {
    const testUa = "utest1" + "b".repeat(72);
    const r = classifyRecipientInput(testUa, "mainnet");
    expect(r.kind).toBe("invalid");
  });
});
