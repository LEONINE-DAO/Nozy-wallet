import test from "node:test";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
  advanceWitnessOverRange,
  anchorHexFromCompactTreestate,
  compactBlocksByHeight,
  explicitAnchorHex,
  feeBlockFromChainTip,
  isNozyPublicSyncUrl,
  mainnetFromNetworkName,
  zcashTxidFromRawHex
} from "./public-sync-send.js";

test("isNozyPublicSyncUrl matches the public lightwalletd host only", () => {
  assert.equal(isNozyPublicSyncUrl("https://lwd.nozywallet.org:443"), true);
  assert.equal(isNozyPublicSyncUrl("http://127.0.0.1:8232"), false);
  assert.equal(isNozyPublicSyncUrl("https://zec.rocks:443"), false);
  assert.equal(isNozyPublicSyncUrl(""), false);
});

test("feeBlockFromChainTip mines in the next block", () => {
  assert.deepEqual(feeBlockFromChainTip(3_600_000, true), {
    mainnet: true,
    minedBlockHeight: 3_600_001
  });
  assert.equal(feeBlockFromChainTip(Number.NaN).minedBlockHeight, 1);
});

test("mainnetFromNetworkName treats a missing name as mainnet", () => {
  assert.equal(mainnetFromNetworkName(""), true);
  assert.equal(mainnetFromNetworkName("main"), true);
  assert.equal(mainnetFromNetworkName("test"), false);
  assert.equal(mainnetFromNetworkName("regtest"), false);
});

test("explicitAnchorHex ignores a frontier blob that is not a 32-byte root", () => {
  const frontier = "01" + "ab".repeat(40);
  assert.equal(explicitAnchorHex({ orchard_tree: frontier, orchard: { commitments: { finalState: frontier } } }), "");
  const root = "cd".repeat(32);
  assert.equal(explicitAnchorHex({ anchor: root }), root);
  assert.equal(
    explicitAnchorHex({ orchard: { commitments: { finalRoot: `0x${root}` } } }),
    root
  );
});

test("anchorHexFromCompactTreestate derives the root from finalState", () => {
  const root = "11".repeat(32);
  const hex = anchorHexFromCompactTreestate(
    { orchard_tree: "01ff", orchard: { commitments: { finalState: "01ff" } } },
    () => root
  );
  assert.equal(hex, root);
  assert.equal(
    anchorHexFromCompactTreestate({ orchard_tree: "01ff" }, () => {
      throw new Error("bad tree");
    }),
    ""
  );
});

test("compactBlocksByHeight chunks ranges the compact server accepts", async () => {
  const calls = [];
  const fetchRange = async (start, end) => {
    calls.push([start, end]);
    assert.ok(end - start <= 500);
    return [{ height: start, tx: [] }, { height: end, tx: [{ orchard: { actions: [] } }] }];
  };
  const map = await compactBlocksByHeight(fetchRange, 1, 700);
  assert.equal(calls.length, 2);
  assert.deepEqual(calls[0], [1, 500]);
  assert.deepEqual(calls[1], [501, 700]);
  assert.equal(map.get(700).height, 700);
  assert.equal(map.has(250), false);
});

test("advanceWitnessOverRange fills missing heights with empty blocks", async () => {
  const seen = [];
  const hex = await advanceWitnessOverRange(
    (witness, blockJson) => {
      seen.push(JSON.parse(blockJson));
      return witness + "x";
    },
    "w",
    4,
    6,
    async () => [{ height: 5, tx: [{ txid: "aa" }] }]
  );
  assert.equal(hex, "wxx");
  assert.deepEqual(
    seen.map((b) => b.height),
    [5, 6]
  );
  assert.deepEqual(seen[0].tx, [{ txid: "aa" }]);
  assert.deepEqual(seen[1].tx, []);
});

test("zcashTxidFromRawHex is reversed double-sha256", async () => {
  const raw = "010203";
  const once = createHash("sha256").update(Buffer.from(raw, "hex")).digest();
  const twice = createHash("sha256").update(once).digest();
  const expected = Buffer.from(twice).reverse().toString("hex");
  assert.equal(await zcashTxidFromRawHex(raw), expected);
  await assert.rejects(() => zcashTxidFromRawHex("zz"), /invalid/);
});
