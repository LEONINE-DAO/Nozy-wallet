import { describe, expect, it } from "vitest";
import {
  atRiskDelegations,
  finalizerCrowdLabel,
  finalizerDisplayName,
  finalizerNeedsRetarget,
  hybridPosStanding,
  pickSaferFinalizer,
  saferSuggestionPool,
  type HybridPosFinalizer
} from "./hybridPos";

const AA = "aa".repeat(32);
const BB = "bb".repeat(32);
const CC = "cc".repeat(32);
const DD = "dd".repeat(32);
const EE = "ee".repeat(32);

function row(partial: Partial<HybridPosFinalizer> & { pubkey: string }): HybridPosFinalizer {
  return {
    rank: 1,
    stake_ctaz: 100,
    share_pct: 1,
    cumulative_pct: 1,
    in_threshold_set: false,
    name: null,
    website: null,
    score: 90,
    grade: "B+",
    provisional: false,
    unobserved: false,
    score_note: null,
    voted: 10,
    of: 10,
    pct: 100,
    first_seen: 1,
    live: true,
    ...partial
  };
}

describe("finalizerNeedsRetarget", () => {
  it("does not nag when observer data is missing", () => {
    expect(finalizerNeedsRetarget(null)).toBeNull();
    expect(finalizerNeedsRetarget(undefined)).toBeNull();
  });

  it("flags offline before grade", () => {
    expect(
      finalizerNeedsRetarget(row({ pubkey: AA, live: false, score: 95, grade: "A" }))
    ).toBe("offline");
  });

  it("flags below the B− bar", () => {
    expect(
      finalizerNeedsRetarget(row({ pubkey: AA, score: 40, grade: "D" }))
    ).toBe("uneven");
  });

  it("leaves reliable live finalizers alone", () => {
    expect(
      finalizerNeedsRetarget(row({ pubkey: AA, score: 88, grade: "B" }))
    ).toBeNull();
  });

  it("does not treat provisional or unobserved as a retarget reason", () => {
    expect(
      finalizerNeedsRetarget(row({ pubkey: AA, provisional: true, score: 40, grade: "D" }))
    ).toBeNull();
    expect(hybridPosStanding(row({ pubkey: AA, provisional: true }))).toBe("provisional");
    expect(
      finalizerNeedsRetarget(row({ pubkey: AA, unobserved: true, grade: null, score: null }))
    ).toBeNull();
  });
});

describe("saferSuggestionPool / pickSaferFinalizer", () => {
  const reliableSmall = row({ pubkey: BB, score: 85, grade: "B", in_threshold_set: false });
  const reliableLarge = row({
    pubkey: CC,
    score: 99,
    grade: "A",
    in_threshold_set: true
  });
  const dead = row({ pubkey: DD, live: false, score: 90, grade: "B+" });
  const weak = row({ pubkey: EE, score: 20, grade: "F" });

  it("keeps live B−+ finalizers outside the largest third", () => {
    const pool = saferSuggestionPool(
      [reliableSmall, reliableLarge, dead, weak],
      [AA]
    );
    expect(pool.map((r) => r.pubkey)).toEqual([BB]);
  });

  it("excludes the current target", () => {
    const pool = saferSuggestionPool([reliableSmall, row({ pubkey: AA, score: 91 })], [AA]);
    expect(pool.map((r) => r.pubkey)).toEqual([BB]);
  });

  it("returns a stable pick for the same seed", () => {
    const pool = [row({ pubkey: BB }), row({ pubkey: DD, score: 82, grade: "B−" })];
    const a = pickSaferFinalizer(pool, AA);
    const b = pickSaferFinalizer(pool, AA);
    expect(a?.pubkey).toBe(b?.pubkey);
    expect(a?.pubkey).toBeTruthy();
  });

  it("returns null on an empty pool", () => {
    expect(pickSaferFinalizer([], AA)).toBeNull();
  });
});

describe("atRiskDelegations", () => {
  it("lists offline and weak bonds with a safer suggestion", () => {
    const safer = row({ pubkey: BB, score: 86, grade: "B" });
    const offline = row({ pubkey: AA, live: false, score: 90, grade: "A" });
    const weak = row({ pubkey: CC, score: 30, grade: "D" });
    const largeOk = row({
      pubkey: DD,
      score: 95,
      grade: "A",
      in_threshold_set: true
    });

    const risks = atRiskDelegations(
      {
        [AA]: [{ pk: "bond-offline", latest_val: 50 }],
        [CC]: [{ pk: "bond-weak", latest_val: 10 }],
        [DD]: [{ pk: "bond-ok", latest_val: 999 }]
      },
      [offline, weak, safer, largeOk]
    );

    expect(risks.map((r) => r.reason)).toEqual(["offline", "uneven"]);
    expect(risks[0]?.suggestion?.pubkey).toBe(BB);
    expect(risks[1]?.suggestion?.pubkey).toBe(BB);
    expect(risks.some((r) => r.finalizer === DD)).toBe(false);
  });
});

describe("finalizerCrowdLabel", () => {
  it("tags largest third vs spread", () => {
    expect(finalizerCrowdLabel(row({ pubkey: AA, in_threshold_set: true }))).toBe(
      "largest_third"
    );
    expect(finalizerCrowdLabel(row({ pubkey: BB, in_threshold_set: false }))).toBe(
      "spread"
    );
    expect(finalizerDisplayName(row({ pubkey: AA, name: "  Labs  " }))).toBe("Labs");
    expect(finalizerDisplayName(row({ pubkey: AA, name: null }))).toBeNull();
  });
});
