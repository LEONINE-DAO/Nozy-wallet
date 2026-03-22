import initWasm, * as wasm from "../wasm/pkg/nozy_wasm.js";
import { rpcFallbackWithRequester, selectNotesForSpend } from "./tx-utils.js";
import { normalizeRpcEndpoint, rpcNetworkErrorMessage } from "./rpc-utils.js";

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
      const resp = await fetch(endpoint, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          jsonrpc: "2.0",
          id: 1,
          method,
          params
        })
      });
      if (resp.status === 401 || resp.status === 403) {
        throw new Error(
          `RPC ${method} returned HTTP ${resp.status} (authentication required). ` +
            `For Zebra local dev, disable cookie auth or use a credentialed proxy.`
        );
      }
      if (!resp.ok) throw new Error(`RPC ${method} failed: ${resp.status}`);
      const body = await resp.json();
      if (body?.error) {
        const e = new Error(`RPC ${method} error: ${body.error.message ?? JSON.stringify(body.error)}`);
        if (typeof body.error.code === "number") e.jsonRpcCode = body.error.code;
        e.jsonRpcMethod = method;
        throw e;
      }
      return body?.result ?? null;
    } catch (err) {
      const msg = String(err?.message ?? err ?? "");
      if (msg === "Failed to fetch" || /NetworkError|network failed|Load failed/i.test(msg)) {
        lastErr = new Error(rpcNetworkErrorMessage(endpoint, err));
      } else {
        lastErr = err instanceof Error ? err : new Error(String(err));
      }
      if (attempt < retries) {
        await new Promise((r) => setTimeout(r, baseDelayMs * 2 ** attempt));
      }
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

function orchardAnchorHexFromRpc(tr) {
  if (!tr || typeof tr !== "object") return "";
  const o = tr.orchard;
  const c = o?.commitments ?? o;
  const fromZebra = c?.finalRoot ?? c?.final_root ?? o?.finalRoot ?? o?.final_root ?? "";
  let hex = String(tr.anchor ?? tr.orchardTree?.anchor ?? fromZebra ?? "").trim();
  if (hex.startsWith("0x") || hex.startsWith("0X")) hex = hex.slice(2);
  return hex;
}

async function fetchOrchardAnchor(rpcEndpoint, targetHeight) {
  const h = String(targetHeight);
  const orchardTree = await rpcFallback(rpcEndpoint, [
    { method: "z_getorchardtree", params: [h] },
    { method: "z_gettreestate", params: [h] }
  ]);
  const anchorHex = orchardAnchorHexFromRpc(orchardTree);
  if (!anchorHex || anchorHex.length < 64) {
    throw new Error("RPC did not return a valid Orchard anchor.");
  }
  return anchorHex;
}

function rewriteMissingWitnessRpcError(err) {
  const m = String(err?.message || err || "");
  const code = typeof err?.jsonRpcCode === "number" ? err.jsonRpcCode : null;
  const looksLikeMissingMethod =
    code === -32601 ||
    /method not found/i.test(m) ||
    /\bmethod\b.*\bnot found\b/i.test(m) ||
    /does not exist|is not available/i.test(m);
  if (!looksLikeMissingMethod) return err instanceof Error ? err : new Error(String(err));
  return new Error(
    "Zebrad (Zebra) does not implement z_findnoteposition or z_getauthpath, which Nozy needs to build Orchard spends. " +
      "Scanning and receiving work with Zebrad; for shielded sends, use a zcashd JSON-RPC (or another node that exposes those methods)."
  );
}

async function fetchWitnessForNote(rpcEndpoint, selectedNote, anchorHex, targetHeight) {
  const cmxHex = bytesToHex(selectedNote?.note?.cmx ?? []);
  if (!cmxHex || cmxHex.length !== 64) {
    throw new Error(`Selected note cmx hex invalid (len=${cmxHex.length}).`);
  }

  try {
    const posResp = await rpcFallback(rpcEndpoint, [
      { method: "z_findnoteposition", params: [cmxHex] },
      { method: "z_findnoteposition", params: [cmxHex, String(selectedNote.height)] }
    ]);
    const position = Number(posResp?.position ?? posResp?.pos ?? posResp ?? 0);
    if (!Number.isFinite(position) || position < 0) {
      throw new Error("z_findnoteposition returned invalid position.");
    }

    const authResp = await rpcFallback(rpcEndpoint, [
      { method: "z_getauthpath", params: [position, anchorHex] },
      { method: "z_getauthpath", params: [position] }
    ]);
    const authPathHexes = authResp?.auth_path ?? authResp?.authPath ?? authResp?.path ?? [];
    if (!Array.isArray(authPathHexes) || authPathHexes.length !== 32) {
      throw new Error(`z_getauthpath returned unexpected auth_path length: ${authPathHexes.length}`);
    }

    return {
      anchor: anchorHex,
      position: position >>> 0,
      auth_path: authPathHexes,
      target_height: targetHeight
    };
  } catch (e) {
    throw rewriteMissingWitnessRpcError(e);
  }
}

self.onmessage = async (event) => {
  const { id, method, params } = event.data || {};
  try {
    await ensureReady();

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

      for (let h = startHeight; h <= endHeight; h += 1) {
        scannedBlocks += 1;
        try {
          const resp = await fetch(endpoint, {
            method: "POST",
            headers: { "content-type": "application/json" },
            body: JSON.stringify({
              jsonrpc: "2.0",
              id: 1,
              method: "getblock",
              params: [String(h), 2]
            })
          });
          if (!resp.ok) continue;
          const body = await resp.json();
          if (body?.error || !body?.result) continue;
          const block = body.result;
          const actions = extractActionsFromBlockJson(block);
          if (actions.length === 0) continue;

          const txid = block?.hash || `h${h}`;
          const scan = wasm.scan_orchard_actions(
            mnemonic,
            address,
            JSON.stringify(actions),
            h,
            txid
          );
          if (scan?.notes?.length) {
            discoveredNotes.push(...scan.notes);
            totalBalanceZats += Number(scan.total_value_zats || 0);
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
      const requestedFee = Number(params?.fee ?? 10000);
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

      // Approach:
      // 1) Scan a recent window for decryptable Orchard notes.
      // 2) Fetch witness data with RPC method fallbacks across node variants.
      // 3) Build Orchard v5 transaction in WASM.
      const endHeight = Number(await rpcRequest(endpoint, "getblockcount", []));
      const scanWindow = Number(params?.scanWindow ?? 200);
      const startHeight = Math.max(0, endHeight - Math.max(10, scanWindow));
      const requiredValue = requestedAmount + requestedFee;
      if (!Number.isFinite(requiredValue) || requiredValue <= 0) {
        throw new Error(`Invalid amount/fee (amount=${requestedAmount}, fee=${requestedFee}).`);
      }

      const candidates = [];
      let scannedValue = 0;

      for (let h = startHeight; h <= endHeight; h += 1) {
        try {
          const resp = await fetch(endpoint, {
            method: "POST",
            headers: { "content-type": "application/json" },
            body: JSON.stringify({
              jsonrpc: "2.0",
              id: 1,
              method: "getblock",
              params: [String(h), 2]
            })
          });
          if (!resp.ok) continue;
          const body = await resp.json();
          if (body?.error || !body?.result) continue;
          const block = body.result;

          const actions = extractActionsFromBlockJson(block);
          if (!actions.length) continue;

          const txid = block?.hash || `h${h}`;
          const scan = wasm.scan_orchard_actions(
            mnemonic,
            walletAddress,
            JSON.stringify(actions),
            h,
            txid
          );

          if (scan?.notes?.length) {
            for (const n of scan.notes) {
              const v = Number(n?.value ?? 0);
              if (Number.isFinite(v) && v > 0) {
                scannedValue += v;
                candidates.push({
                  note: n,
                  height: h,
                  txid,
                  value: v
                });
              }
            }
          }
        } catch (_) {
          // Continue scanning even if one block fails.
        }
      }

      if (candidates.length === 0) {
        throw new Error(
          `No spendable Orchard notes in blocks ${startHeight}..${endHeight}.`
        );
      }

      const selected = selectNotesForSpend(candidates, requiredValue);
      if (selected.length === 0) {
        throw new Error(
          `Insufficient funds for amount+fee (${requiredValue}). Scanned spendable value: ${scannedValue}.`
        );
      }
      const selectedNote = selected[0].note;
      const selectedHeight = selected[0].height;
      const selectedTxid = selected[0].txid;
      const spendValue = selected.reduce((acc, n) => acc + n.value, 0);
      const spendAmount = requestedAmount;
      const targetHeight = endHeight;
      const sharedAnchor = await fetchOrchardAnchor(endpoint, targetHeight);
      const selectedWitnesses = [];
      for (const noteSel of selected) {
        const witness = await fetchWitnessForNote(
          endpoint,
          noteSel,
          sharedAnchor,
          targetHeight
        );
        selectedWitnesses.push(witness);
      }

      const provingResult = wasm.build_orchard_v5_tx_from_note(
        mnemonic,
        recipientAddress,
        spendAmount,
        requestedFee,
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

