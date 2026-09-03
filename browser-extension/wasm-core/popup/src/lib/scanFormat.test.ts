import { describe, expect, it } from "vitest";
import { isScanInProgress } from "./scanFormat";

describe("isScanInProgress", () => {
  it("is false when idle, done, or missing", () => {
    expect(isScanInProgress(null)).toBe(false);
    expect(isScanInProgress({ status: "idle" })).toBe(false);
    expect(isScanInProgress({ status: "done", scannedBlocks: 10, totalBlocks: 10 })).toBe(false);
  });

  it("is true while scanning below 100%", () => {
    expect(
      isScanInProgress({
        status: "scanning",
        scannedBlocks: 50,
        totalBlocks: 100
      })
    ).toBe(true);
  });

  it("is false at 100% even if status is still scanning", () => {
    expect(
      isScanInProgress({
        status: "scanning",
        scannedBlocks: 100,
        totalBlocks: 100
      })
    ).toBe(false);
  });
});
