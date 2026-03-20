import { describe, expect, it } from "vitest";
import { isLikelyUnifiedOrchardAddress, memoByteLength, validateMemoLimit } from "./security";

describe("security helpers", () => {
  it("validates orchard-like unified addresses", () => {
    expect(isLikelyUnifiedOrchardAddress("u1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq")).toBe(true);
    expect(isLikelyUnifiedOrchardAddress("t1legacyaddress")).toBe(false);
    expect(isLikelyUnifiedOrchardAddress("")).toBe(false);
  });

  it("enforces memo byte limit", () => {
    const shortMemo = "hello";
    const longMemo = "a".repeat(513);
    expect(memoByteLength(shortMemo)).toBe(5);
    expect(validateMemoLimit(shortMemo)).toBe(true);
    expect(validateMemoLimit(longMemo)).toBe(false);
  });
});

