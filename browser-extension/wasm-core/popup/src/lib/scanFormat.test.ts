import { describe, expect, it } from "vitest";
import {
  isScanInProgress,
  scanLooksLikeEmptyNearTipWindow,
  shouldJumpToOwnBirthday
} from "./scanFormat";

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

describe("scanLooksLikeEmptyNearTipWindow", () => {
  it("is true for a short empty near-tip range", () => {
    expect(
      scanLooksLikeEmptyNearTipWindow({
        status: "scanning",
        startHeight: 3_471_077,
        endHeight: 3_471_866,
        discoveredNotes: 0
      })
    ).toBe(true);
  });

  it("is false once notes exist or the range starts at the restore floor", () => {
    expect(
      scanLooksLikeEmptyNearTipWindow({
        status: "done",
        startHeight: 3_471_077,
        endHeight: 3_471_866,
        discoveredNotes: 2
      })
    ).toBe(false);
    expect(
      scanLooksLikeEmptyNearTipWindow({
        status: "scanning",
        startHeight: 3_050_000,
        endHeight: 3_471_866,
        discoveredNotes: 0
      })
    ).toBe(false);
  });
});

describe("shouldJumpToOwnBirthday", () => {
  it("is true when any wallet is scanning before its own birthday", () => {
    expect(
      shouldJumpToOwnBirthday(
        { status: "scanning", startHeight: 3_050_000, endHeight: 3_471_918, discoveredNotes: 0 },
        3_471_077
      )
    ).toBe(true);
  });

  it("is false when already at or after this wallet's birthday", () => {
    expect(
      shouldJumpToOwnBirthday(
        { status: "scanning", startHeight: 3_471_077, endHeight: 3_471_918, discoveredNotes: 0 },
        3_471_077
      )
    ).toBe(false);
    expect(
      shouldJumpToOwnBirthday(
        { status: "scanning", startHeight: 3_050_000, endHeight: 3_471_918, discoveredNotes: 0 },
        3_050_000
      )
    ).toBe(false);
  });
});
