import test from "node:test";
import assert from "node:assert/strict";
import {
  anchorHexFromTreestate,
  blockForPoolAdvance,
  notePool,
  nullifierHex,
  witnessFromTracker,
  witnessMismatchError
} from "./witness-spend.js";

test("nullifier hex accepts a 32-byte array", () => {
  const nf = Array.from({ length: 32 }, (_, i) => i);
  assert.equal(
    nullifierHex({ nullifier: nf }),
    nf.map((b) => b.toString(16).padStart(2, "0")).join("")
  );
});

test("tracker witness is pool-specific", () => {
  const nf = "ab".repeat(32);
  const orchardWitness = "11".repeat(40);
  const ironwoodWitness = "22".repeat(40);
  const tracker = JSON.stringify({
    orchard: JSON.stringify({ witnesses: { [nf]: orchardWitness } }),
    ironwood: JSON.stringify({ witnesses: { [nf]: ironwoodWitness } })
  });
  const note = { pool: "ironwood", nullifier: nf };
  assert.equal(notePool(note), "ironwood");
  assert.equal(witnessFromTracker(tracker, note), ironwoodWitness);
  assert.equal(witnessFromTracker(tracker, { pool: "orchard", nullifier: nf }), orchardWitness);
});

test("ironwood catch-up reads ironwood actions", () => {
  const shaped = blockForPoolAdvance(
    {
      tx: [
        { txid: "aa", orchard: { actions: [{ cmx: "11" }] }, ironwood: { actions: [{ cmx: "22" }] } }
      ]
    },
    "ironwood"
  );
  assert.deepEqual(shaped.tx[0].orchard.actions, [{ cmx: "22" }]);
});

test("ironwood anchor does not fall back to the orchard root", () => {
  const ts = {
    orchard: { commitments: { finalRoot: "aa".repeat(32) } },
    ironwood: { commitments: { finalRoot: "bb".repeat(32) } }
  };
  assert.equal(anchorHexFromTreestate(ts, "ironwood"), "bb".repeat(32));
  assert.equal(anchorHexFromTreestate(ts, "orchard"), "aa".repeat(32));
});

test("mismatch text identifies this build", () => {
  const msg = witnessMismatchError({
    pool: "orchard",
    witnessTip: 10,
    anchorHeight: 12,
    chainTip: 12,
    usedTracker: true
  });
  assert.match(msg, /NozyWallet 2\.0\.4/);
  assert.equal(msg.includes("does not match anchor at tip (rescan or wait for sync)"), false);
});
