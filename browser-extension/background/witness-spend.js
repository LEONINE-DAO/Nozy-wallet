/**
 * Spend-time witness selection.
 *
 * Scan stores a witness on the note at the moment it is found, which is
 * mid-block. The tracker keeps advancing that witness through later
 * commitments. Spending the note snapshot and then skipping to the next
 * block drops the rest of the discovery block, so the root cannot match
 * the node anchor.
 */

import { selectNotesForSpend } from "./tx-utils.js";

const TIP_CATCHUP_LIMIT = 40;

export function notePool(note) {
  return note?.pool === "ironwood" ? "ironwood" : "orchard";
}

function bytesFromUnknown(value) {
  if (value instanceof Uint8Array) return Array.from(value);
  if (Array.isArray(value)) return value;
  if (value && typeof value === "object") {
    const keys = Object.keys(value);
    if (keys.length > 0 && keys.every((k) => /^\d+$/.test(k))) {
      const out = [];
      for (const k of keys) out[Number(k)] = Number(value[k]) & 255;
      return out;
    }
  }
  return null;
}

export function nullifierHex(note) {
  const nf = note?.nullifier ?? note?.nf;
  if (typeof nf === "string") {
    const clean = nf.replace(/^0x/i, "").toLowerCase();
    return /^[0-9a-f]{64}$/.test(clean) ? clean : "";
  }
  const arr = bytesFromUnknown(nf);
  if (!arr || arr.length !== 32) return "";
  return arr.map((b) => (Number(b) & 255).toString(16).padStart(2, "0")).join("");
}

function parseJson(value) {
  if (value && typeof value === "object") return value;
  if (typeof value !== "string" || !value.trim()) return null;
  try {
    return JSON.parse(value);
  } catch {
    return null;
  }
}

export function witnessHexFromNote(note, pool) {
  if (!note || typeof note !== "object") return { witnessHex: "", tipHeight: NaN };
  const iron = pool === "ironwood";
  const witnessHex =
    (iron &&
      typeof note.ironwood_incremental_witness_hex === "string" &&
      note.ironwood_incremental_witness_hex) ||
    (iron &&
      typeof note.ironwoodIncrementalWitnessHex === "string" &&
      note.ironwoodIncrementalWitnessHex) ||
    (typeof note.orchard_incremental_witness_hex === "string" &&
      note.orchard_incremental_witness_hex) ||
    (typeof note.orchardIncrementalWitnessHex === "string" && note.orchardIncrementalWitnessHex) ||
    "";
  const tipHeight = Number(
    (iron ? note.ironwood_witness_tip_height ?? note.ironwoodWitnessTipHeight : undefined) ??
      note.orchard_witness_tip_height ??
      note.orchardWitnessTipHeight ??
      NaN
  );
  return { witnessHex, tipHeight };
}

function witnessHexFromInner(inner, note) {
  if (!inner?.witnesses || typeof inner.witnesses !== "object") return "";
  const nf = nullifierHex(note);
  if (!nf) return "";
  const direct = inner.witnesses[nf];
  if (typeof direct === "string" && direct.length > 32) return direct;
  const hit = Object.keys(inner.witnesses).find((k) => k.replace(/^0x/i, "").toLowerCase() === nf);
  const hex = hit ? inner.witnesses[hit] : "";
  return typeof hex === "string" && hex.length > 32 ? hex : "";
}

/** Live witness from `{orchard, ironwood}` tracker JSON, keyed by nullifier. */
export function witnessFromTracker(trackerState, note) {
  const outer = parseJson(trackerState);
  if (!outer) return "";
  const pool = notePool(note);
  if (outer.orchard || outer.ironwood) {
    const raw = pool === "ironwood" ? outer.ironwood : outer.orchard;
    return witnessHexFromInner(parseJson(raw), note);
  }
  if (pool === "orchard") return witnessHexFromInner(outer, note);
  return "";
}

export function anchorHexFromTreestate(ts, pool) {
  if (!ts || typeof ts !== "object") return "";
  const node = ts[pool];
  const c = node?.commitments ?? node;
  const fromPool = c?.finalRoot ?? c?.final_root ?? node?.finalRoot ?? node?.final_root ?? "";
  const legacy = pool === "orchard" ? ts.anchor ?? ts.orchardTree?.anchor ?? "" : "";
  let hex = String(fromPool || legacy || "").trim();
  if (hex.startsWith("0x") || hex.startsWith("0X")) hex = hex.slice(2);
  return hex;
}

function finalStateHex(ts, pool) {
  if (!ts || typeof ts !== "object") return "";
  const node = ts[pool];
  const c = node?.commitments ?? node;
  const raw = c?.finalState ?? c?.final_state ?? node?.finalState ?? node?.final_state ?? "";
  return String(raw || "").trim().replace(/^0x/i, "");
}

function trackerTreeHex(trackerState, pool) {
  const outer = parseJson(trackerState);
  if (!outer) return "";
  const raw = outer.orchard || outer.ironwood ? (pool === "ironwood" ? outer.ironwood : outer.orchard) : outer;
  const inner = parseJson(raw);
  return typeof inner?.tree_hex === "string" ? inner.tree_hex.replace(/^0x/i, "") : "";
}

/** Prefer the commitment-tree root. `finalRoot` can disagree with the tree the scan appended to. */
export function anchorCandidates(ts, pool, wasm) {
  const finalState = finalStateHex(ts, pool);
  const finalRoot = anchorHexFromTreestate(ts, pool);
  const list = [];
  if (finalState && typeof wasm?.orchard_anchor_hex_from_final_state === "function") {
    try {
      const derived = String(wasm.orchard_anchor_hex_from_final_state(finalState) || "").replace(/^0x/i, "");
      if (derived.length >= 64) list.push({ source: "finalState", hex: derived });
    } catch {
      /* fall through to finalRoot */
    }
  }
  if (finalRoot.length >= 64 && !list.some((c) => c.hex === finalRoot)) {
    list.push({ source: "finalRoot", hex: finalRoot });
  }
  return { list, finalStateLen: finalState.length, finalRootLen: finalRoot.length };
}

/** `advance_orchard_witness_hex` reads `tx.orchard.actions`. Ironwood uses the same hasher. */
export function blockForPoolAdvance(block, pool) {
  if (pool !== "ironwood" || !block || typeof block !== "object") return block;
  const txs = Array.isArray(block.tx) ? block.tx : [];
  return {
    tx: txs.map((tx) => {
      if (!tx || typeof tx !== "object") return { txid: "00", orchard: { actions: [] } };
      const poolObj = tx.ironwood ?? tx.ironwoodActions ?? tx.ironwood_actions;
      let actions = [];
      if (Array.isArray(poolObj)) actions = poolObj;
      else if (poolObj && Array.isArray(poolObj.actions)) actions = poolObj.actions;
      return {
        txid: String(tx.txid || tx.hash || "00"),
        orchard: { actions }
      };
    })
  };
}

export function witnessMismatchError(info) {
  const source = info.usedTracker ? "scanned tracker" : "note snapshot";
  return (
    `Send check failed in NozyWallet 2.0.4. The ${info.pool} witness from the ${source} ` +
    `does not match the node at height ${info.anchorHeight} ` +
    `(witness tip ${info.witnessTip}, chain tip ${info.chainTip}). ` +
    `Rescan from this note's birthday so the witness is rebuilt.`
  );
}

export function selectNotesSamePool(candidates, requiredValue) {
  let best = [];
  for (const pool of ["orchard", "ironwood"]) {
    const subset = candidates.filter((n) => notePool(n.note) === pool);
    const picked = selectNotesForSpend(subset, requiredValue);
    if (picked.length && (best.length === 0 || picked.length < best.length)) best = picked;
  }
  return best.length ? best : selectNotesForSpend(candidates, requiredValue);
}

export async function advanceWitnessToHeight(wasm, witnessHex, pool, fromTip, chainTip, fetchBlock) {
  let hex = witnessHex;
  for (let h = fromTip + 1; h <= chainTip; h += 1) {
    const block = await fetchBlock(h);
    if (!block) throw new Error(`Block ${h} was empty while updating the ${pool} witness.`);
    const shaped = blockForPoolAdvance(block, pool);
    hex = wasm.advance_orchard_witness_hex(hex, JSON.stringify(shaped));
  }
  return hex;
}

/**
 * Prefer the tracker witness at the scan frontier. That root already includes
 * every commitment the scan applied. Catch up only a short gap to chain tip.
 */
export async function resolveSpendWitness({
  wasm,
  note,
  noteHeight,
  trackerState,
  scanHeight,
  chainTip,
  fetchBlock,
  fetchTreestate
}) {
  const pool = notePool(note);
  const fromNote = witnessHexFromNote(note, pool);
  const tracked = witnessFromTracker(trackerState, note);
  const tip = Math.floor(Number(chainTip));
  if (!Number.isFinite(tip) || tip < 0) throw new Error("Chain tip is unavailable.");

  const witnessHex = tracked || fromNote.witnessHex;
  if (!witnessHex) {
    throw new Error(
      `This note has no ${pool} witness in the scan (NozyWallet 2.0.4). Rescan from the note's birthday.`
    );
  }

  const frontier =
    tracked && Number.isFinite(Number(scanHeight))
      ? Math.floor(Number(scanHeight))
      : Number.isFinite(fromNote.tipHeight)
        ? Math.floor(fromNote.tipHeight)
        : Math.floor(Number(noteHeight) || 0);
  if (!Number.isFinite(frontier) || frontier < 0) {
    throw new Error(`Invalid ${pool} witness tip.`);
  }

  const anchorAt = async (height) => {
    const ts = await fetchTreestate(height);
    const candidates = anchorCandidates(ts, pool, wasm);
    return { ...candidates, treeHexLen: trackerTreeHex(trackerState, pool).length, treeMatchesState: trackerTreeHex(trackerState, pool) === finalStateHex(ts, pool) && finalStateHex(ts, pool).length > 0 };
  };
  const matches = (hex, anchor) =>
    typeof anchor === "string" &&
    anchor.length >= 64 &&
    wasm.orchard_witness_matches_anchor_hex(hex, anchor);
  const pickAnchor = (witnessHex, packed) => {
    for (const candidate of packed.list) {
      if (matches(witnessHex, candidate.hex)) return candidate;
    }
    return null;
  };

  if (tracked && Number.isFinite(Number(scanHeight))) {
    const scannedAt = Math.floor(Number(scanHeight));
    const scanPacked = await anchorAt(scannedAt);
    const scanPick = pickAnchor(tracked, scanPacked);
    const scanAnchor = scanPick?.hex || "";
    if (scanPick) {
      const gap = tip - scannedAt;
      if (gap > 0 && gap <= TIP_CATCHUP_LIMIT) {
        try {
          const advanced = await advanceWitnessToHeight(
            wasm,
            tracked,
            pool,
            scannedAt,
            tip,
            fetchBlock
          );
          const tipPacked = await anchorAt(tip);
          const tipPick = pickAnchor(advanced, tipPacked);
          if (tipPick) {
            return { witnessHex: advanced, anchorHex: tipPick.hex, anchorHeight: tip, pool };
          }
        } catch {
          /* The scanned root is already valid. */
        }
      }
      return { witnessHex: tracked, anchorHex: scanAnchor, anchorHeight: scannedAt, pool };
    }
  }

  let advanced = witnessHex;
  const gap = tip - frontier;
  if (gap > 0 && gap <= TIP_CATCHUP_LIMIT) {
    advanced = await advanceWitnessToHeight(wasm, witnessHex, pool, frontier, tip, fetchBlock);
  }
  const checkHeight = gap > 0 && gap <= TIP_CATCHUP_LIMIT ? tip : frontier;
  const packed = await anchorAt(checkHeight);
  const picked = pickAnchor(advanced, packed);
  if (!picked) {
    if (!packed.list.length) {
      throw new Error(`Node did not return a ${pool} anchor at height ${checkHeight}.`);
    }
    throw new Error(
      witnessMismatchError({
        pool,
        witnessTip: frontier,
        anchorHeight: checkHeight,
        chainTip: tip,
        usedTracker: Boolean(tracked)
      })
    );
  }
  return { witnessHex: advanced, anchorHex: picked.hex, anchorHeight: checkHeight, pool };
}
