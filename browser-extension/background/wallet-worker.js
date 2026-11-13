import initWasm, * as wasm from "../wasm/pkg/nozy_wasm.js";
import {
  feeBlockFromRpc,
  insufficientFundsMessage,
  mandatoryOrchardFeeZats,
  payForInclusionRequested,
  rpcFallbackWithRequester
} from "./tx-utils.js";
import { notePool, resolveSpendWitness, selectNotesSamePool } from "./witness-spend.js";
import {
  normalizeRpcEndpoint,
  rpcCallJson,
  rpcGetBlockVerboseByHeight,
  rpcNetworkErrorMessage
} from "./rpc-utils.js";

let ready;
async function ensureReady() {
  if (!ready) {
    ready = initWasm();
  }
  await ready;
}

function randomTxidLike() {
  const bytes = crypto.getRandomValues(new Uint8Array(32));
  return Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");
}

function toByteArray(value) {
  if (Array.isArray(value)) return value.map((v) => Number(v) & 0xff);
  if (typeof value === "string") {
    const clean = value.startsWith("0x") ? value.slice(2) : value;
    if (clean.length % 2 !== 0) return [];
    const bytes = [];
    for (let i = 0; i < clean.length; i += 2) {
      bytes.push(parseInt(clean.slice(i, i + 2), 16));
    }
    return bytes;
  }
  return [];
}

function bytesToHex(bytes) {
  const arr = Array.isArray(bytes) || bytes instanceof Uint8Array ? bytes : [];
  return Array.from(arr, (b) => Number(b) & 0xff).map((b) => b.toString(16).padStart(2, "0")).join("");
}

function poolFinalStateFromTreestate(ts, pool) {
  const c = ts?.[pool]?.commitments ?? ts?.[pool];
  return (
    (typeof c?.finalState === "string" && c.finalState) ||
    (typeof c?.final_state === "string" && c.final_state) ||
    ""
  );
}

function serializeShieldedTrackerState(trackers) {
  return JSON.stringify({
    orchard: trackers.orchard || "",
    ironwood: trackers.ironwood || ""
  });
}

async function initShieldedTrackerState(endpoint, startHeight) {
  let orchardFinal = "";
  let ironwoodFinal = "";
  if (startHeight > 0) {
    const ts = await rpcFallback(endpoint, [
      { method: "z_gettreestate", params: [String(startHeight - 1)] }
    ]);
    orchardFinal = poolFinalStateFromTreestate(ts, "orchard");
    ironwoodFinal = poolFinalStateFromTreestate(ts, "ironwood");
  }
  if (typeof wasm.shielded_scan_tracker_new === "function") {
    return wasm.shielded_scan_tracker_new(orchardFinal, ironwoodFinal);
  }
  return serializeShieldedTrackerState({
    orchard: wasm.orchard_scan_tracker_new(orchardFinal),
    ironwood: wasm.orchard_scan_tracker_new(ironwoodFinal)
  });
}

function wasmJsonValue(v) {
  if (v instanceof Map) {
    const o = {};
    for (const [k, val] of v) o[String(k)] = wasmJsonValue(val);
    return o;
  }
  if (Array.isArray(v)) return v.map(wasmJsonValue);
  return v;
}

function applyShieldedScanBlock(
  trackerJson,
  mnemonic,
  address,
  height,
  blockJson,
  accountIndicesJson
) {
  const out = wasmJsonValue(
    wasm.orchard_scan_tracker_apply_block(
      trackerJson,
      mnemonic,
      address,
      height,
      blockJson,
      accountIndicesJson || "[0]"
    )
  );
  const nextTracker =
    out?.tracker_state ??
    out?.trackerState ??
    (out?.orchard_tracker_state && out?.ironwood_tracker_state
      ? serializeShieldedTrackerState({
          orchard: out.orchard_tracker_state,
          ironwood: out.ironwood_tracker_state
        })
      : null);
  return { out, nextTracker };
}

async function rpcRequest(rpcEndpoint, method, params = [], opts = {}) {
  let endpoint;
  try {
    endpoint = normalizeRpcEndpoint(rpcEndpoint);
  } catch (e) {
    throw e instanceof Error ? e : new Error(String(e));
  }
  const retries = Number.isFinite(opts.retries) ? opts.retries : 2;
  const baseDelayMs = Number.isFinite(opts.baseDelayMs) ? opts.baseDelayMs : 200;
  let lastErr;
  for (let attempt = 0; attempt <= retries; attempt += 1) {
    try {
      const called = await rpcCallJson(endpoint, method, params);
      if (called.endpoint) endpoint = called.endpoint;
      return called.result;
    } catch (err) {
      const msg = String(err?.message ?? err ?? "");
      if (msg === "Failed to fetch" || /NetworkError|network failed|Load failed/i.test(msg)) {
        lastErr = new Error(rpcNetworkErrorMessage(endpoint, err));
      } else {
        lastErr = err instanceof Error ? err : new Error(String(err));
      }
      if (attempt < retries && !/HTTP 415|cannot use /.test(String(lastErr?.message ?? ""))) {
        await new Promise((r) => setTimeout(r, baseDelayMs * 2 ** attempt));
        continue;
      }
      break;
    }
  }
  throw lastErr || new Error(`RPC ${method} failed`);
}

async function rpcFallback(rpcEndpoint, attempts) {
  return rpcFallbackWithRequester(
    (at) => rpcRequest(rpcEndpoint, at.method, at.params ?? [], at.opts ?? {}),
    attempts
  );
}

function normalizeAction(raw) {
  const nullifier = toByteArray(raw?.nullifier ?? raw?.nf ?? []);
  const cmx = toByteArray(raw?.cmx ?? raw?.note_commitment ?? []);
  const ephemeral_key = toByteArray(raw?.ephemeralKey ?? raw?.ephemeral_key ?? raw?.epk ?? []);
  const encrypted_note = toByteArray(raw?.encCiphertext ?? raw?.encrypted_note ?? raw?.enc_ciphertext ?? []);
  if (nullifier.length !== 32 || cmx.length !== 32 || ephemeral_key.length !== 32) return null;
  return { nullifier, cmx, ephemeral_key, encrypted_note };
}

function extractActionsFromBlockJson(block) {
  const actions = [];
  const txs = block?.tx ?? block?.transactions ?? [];
  for (const tx of txs) {
    if (typeof tx === "string") continue;
    const orchard = tx?.orchard || tx?.orchard_bundle || {};
    const candidates = orchard?.actions || orchard?.action || tx?.orchard_actions || [];
    if (Array.isArray(candidates)) {
      for (const c of candidates) {
        const normalized = normalizeAction(c);
        if (normalized) actions.push(normalized);
      }
    }
  }
  return actions;
}

function candidateFromDiscovered(raw) {
  if (!raw || typeof raw !== "object" || raw.spent) return null;
  const note = raw.note && typeof raw.note === "object" ? raw.note : raw;
  const value = Number(raw.value ?? note?.value ?? 0);
  if (!Number.isFinite(value) || value <= 0) return null;
  const height = Number(raw.height ?? note?.block_height ?? 0);
  return {
    note,
    height,
    txid: String(raw.txid ?? note?.txid ?? ""),
    value,
    pool: notePool(note)
  };
}

self.onmessage = async (event) => {
  const { id, method, params } = event.data || {};
  try {
    await ensureReady();

    if (method === "apply_scan_block") {
      const trackerState = String(params?.trackerState ?? "");
      const mnemonic = String(params?.mnemonic ?? "");
      const address = String(params?.address ?? "");
      const height = Number(params?.height ?? 0);
      const blockJson = String(params?.blockJson ?? "");
      if (!trackerState) throw new Error("Missing trackerState");
      if (!mnemonic) throw new Error("Missing mnemonic");
      if (!address) throw new Error("Missing address");
      if (!Number.isFinite(height)) throw new Error("Invalid height");
      if (!blockJson) throw new Error("Missing blockJson");
      const { out, nextTracker } = applyShieldedScanBlock(
        trackerState,
        mnemonic,
        address,
        Math.floor(height),
        blockJson,
        String(params?.accountIndicesJson ?? "[0]")
      );
      self.postMessage({
        id,
        result: { out, nextTracker: nextTracker || null }
      });
      return;
    }

    if (method === "scan_notes") {
      const startHeight = Number(params?.startHeight ?? 0);
      const endHeight = Number(params?.endHeight ?? startHeight);
      const rpcEndpoint = String(params?.rpcEndpoint ?? "");
      const mnemonic = String(params?.mnemonic ?? "");
      const address = String(params?.address ?? "");
      let endpoint;
      try {
        endpoint = normalizeRpcEndpoint(rpcEndpoint);
      } catch (e) {
        throw e instanceof Error ? e : new Error(String(e));
      }
      let scannedBlocks = 0;
      let totalBalanceZats = 0;
      const discoveredNotes = [];

      let trackerState = await initShieldedTrackerState(endpoint, startHeight);

      for (let h = startHeight; h <= endHeight; h += 1) {
        scannedBlocks += 1;
        try {
          const block = await rpcGetBlockVerboseByHeight(endpoint, h);
          if (!block) continue;
          const blockJson = JSON.stringify(block);
          const { out, nextTracker } = applyShieldedScanBlock(
            trackerState,
            mnemonic,
            address,
            h,
            blockJson,
            String(params?.accountIndicesJson ?? "[0]")
          );
          if (nextTracker) trackerState = nextTracker;
          if (out.notes?.length) {
            for (const n of out.notes) {
              discoveredNotes.push(n);
              totalBalanceZats += Number(n?.value ?? 0);
            }
          }
        } catch (_) {
          // Continue scanning even if one block fails.
        }
      }

      self.postMessage({
        id,
        result: {
          scannedBlocks,
          discoveredNotes,
          totalBalanceZats
        }
      });
      return;
    }

    if (method === "prove_transaction") {
      const recipientAddress = String(
        params?.recipientAddress ?? params?.to ?? ""
      );
      const walletAddress = String(params?.walletAddress ?? "");
      const mnemonic = String(params?.mnemonic ?? "");
      const rpcEndpoint = String(params?.rpcEndpoint ?? "");
      const requestedAmount = Number(params?.amount ?? 0);
      const memo = String(params?.memo ?? "nozy-poc");

      if (!rpcEndpoint) throw new Error("Missing rpcEndpoint for proving scan.");
      if (!mnemonic) throw new Error("Missing wallet mnemonic for proving.");
      if (!walletAddress) throw new Error("Missing wallet address for proving scan.");
      if (!recipientAddress) throw new Error("Missing recipientAddress.");

      let endpoint;
      try {
        endpoint = normalizeRpcEndpoint(rpcEndpoint);
      } catch (e) {
        throw e instanceof Error ? e : new Error(String(e));
      }

      // Spend notes the background scan already witnessed. A short RPC rescan is
      // only the fallback when that scan has not recorded any notes yet.
      const feeBlock = await feeBlockFromRpc((method, rpcParams) =>
        rpcRequest(endpoint, method, rpcParams)
      );
      const payForInclusion = payForInclusionRequested(params);
      const requestedFee = mandatoryOrchardFeeZats(wasm, memo, feeBlock, payForInclusion);
      const endHeight = Number(await rpcRequest(endpoint, "getblockcount", []));
      const requiredValue = requestedAmount + requestedFee;
      if (!Number.isFinite(requiredValue) || requiredValue <= 0) {
        throw new Error(`Invalid amount/fee (amount=${requestedAmount}, fee=${requestedFee}).`);
      }

      const passedNotes = Array.isArray(params?.discoveredNotes) ? params.discoveredNotes : [];
      let candidates = passedNotes.map(candidateFromDiscovered).filter(Boolean);
      let scannedValue = candidates.reduce((acc, n) => acc + n.value, 0);
      let trackerState = params?.trackerState ?? "";
      let scanHeight = Number(params?.scanHeight);

      if (candidates.length === 0) {
        const scanWindow = Number(params?.scanWindow ?? 200);
        const startHeight = Math.max(0, endHeight - Math.max(10, scanWindow));
        trackerState = await initShieldedTrackerState(endpoint, startHeight);
        scanHeight = endHeight;
        candidates = [];
        scannedValue = 0;
        for (let h = startHeight; h <= endHeight; h += 1) {
          try {
            const block = await rpcGetBlockVerboseByHeight(endpoint, h);
            if (!block) continue;
            const blockJson = JSON.stringify(block);
            const { out, nextTracker } = applyShieldedScanBlock(
              trackerState,
              mnemonic,
              walletAddress,
              h,
              blockJson,
              String(params?.accountIndicesJson ?? "[0]")
            );
            if (nextTracker) trackerState = nextTracker;
            if (out.notes?.length) {
              const txid = block?.hash || `h${h}`;
              for (const n of out.notes) {
                const v = Number(n?.value ?? 0);
                if (Number.isFinite(v) && v > 0) {
                  scannedValue += v;
                  candidates.push({
                    note: n,
                    height: h,
                    txid,
                    value: v,
                    pool: notePool(n)
                  });
                }
              }
            }
          } catch (_) {
            // Continue scanning even if one block fails.
          }
        }
        if (candidates.length === 0) {
          throw new Error(`No spendable shielded notes in blocks ${startHeight}..${endHeight}.`);
        }
      }

      const selected = selectNotesSamePool(candidates, requiredValue);
      if (selected.length === 0) {
        throw new Error(
          insufficientFundsMessage(requestedAmount, requestedFee, scannedValue)
        );
      }
      const selectedHeight = selected[0].height;
      const selectedTxid = selected[0].txid;
      const spendValue = selected.reduce((acc, n) => acc + n.value, 0);
      const spendAmount = requestedAmount;
      const txBuildHeight = endHeight + 1;
      const tsCache = new Map();
      const fetchTreestate = async (h) => {
        const key = Math.floor(Number(h));
        if (tsCache.has(key)) return tsCache.get(key);
        let ts;
        try {
          ts = await rpcRequest(endpoint, "z_gettreestate", [String(key)]);
        } catch (err) {
          const msg = String(err?.message ?? err);
          const missing = err?.jsonRpcCode === -32601 || /method not found/i.test(msg);
          if (!missing) throw err;
          ts = await rpcRequest(endpoint, "z_getorchardtree", [String(key)]);
        }
        tsCache.set(key, ts);
        return ts;
      };
      const selectedWitnesses = [];
      for (const noteSel of selected) {
        const resolved = await resolveSpendWitness({
          wasm,
          note: noteSel.note,
          noteHeight: noteSel.height,
          trackerState,
          scanHeight,
          chainTip: endHeight,
          fetchBlock: (h) => rpcGetBlockVerboseByHeight(endpoint, h),
          fetchTreestate
        });
        selectedWitnesses.push({
          incremental_witness_hex: resolved.witnessHex,
          anchor_hex: resolved.anchorHex,
          target_height: txBuildHeight,
          fee_block_height: feeBlock.minedBlockHeight,
          fee_mainnet: feeBlock.mainnet,
          marginal_fee: feeBlock.marginalFee || 0,
          pay_for_inclusion: payForInclusion
        });
      }

      const provingResult = wasm.build_orchard_v5_tx_from_note(
        mnemonic,
        recipientAddress,
        BigInt(spendAmount),
        BigInt(requestedFee),
        memo,
        JSON.stringify(selected.map((s) => s.note)),
        JSON.stringify(selectedWitnesses)
      );

      self.postMessage({
        id,
        result: {
          txid: provingResult?.txid ?? "",
          chainId: wasm.get_zcash_chain_id(),
          rawTxHex: provingResult?.rawTxHex ?? "",
          proving: "orchestrated_orchard_v5_tx_build_wasm",
          bundle_actions: provingResult?.bundle_actions ?? 0,
          proof_generated: provingResult?.proof_generated ?? true,
          selected_note: {
            value: spendValue,
            block_height: selectedHeight,
            txid: selectedTxid
          },
          selected_notes_count: selected.length,
          selected_notes_total_value: spendValue,
          selected_notes: selected.map((s) => ({
            value: Number(s?.note?.value ?? 0),
            cmx: bytesToHex(s?.note?.cmx ?? []).slice(0, 16),
            block_height: s.height
          })),
          selected_witnesses_count: selectedWitnesses.length,
          inputs_used: selected.length,
          input_mode: selected.length <= 1 ? "single" : "multi",
          fee: requestedFee
        }
      });
      return;
    }

    throw new Error(`Unsupported worker method: ${method}`);
  } catch (error) {
    self.postMessage({
      id,
      error: error?.message ?? String(error)
    });
  }
};

