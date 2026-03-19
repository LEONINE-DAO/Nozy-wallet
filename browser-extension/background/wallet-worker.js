import initWasm, * as wasm from "../wasm/pkg/nozy_wasm.js";

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

function normalizeAction(raw) {
  const nullifier = toByteArray(raw?.nullifier ?? raw?.nf ?? []);
  const cmx = toByteArray(raw?.cmx ?? raw?.note_commitment ?? []);
  const ephemeral_key = toByteArray(raw?.ephemeral_key ?? raw?.epk ?? []);
  const encrypted_note = toByteArray(raw?.encrypted_note ?? raw?.enc_ciphertext ?? []);
  if (nullifier.length !== 32 || cmx.length !== 32 || ephemeral_key.length !== 32) return null;
  return { nullifier, cmx, ephemeral_key, encrypted_note };
}

function extractActionsFromBlockJson(block) {
  const actions = [];
  const txs = block?.tx ?? block?.transactions ?? [];
  for (const tx of txs) {
    const orchard = tx?.orchard || tx?.orchard_bundle || tx?.vShieldedOutput || {};
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
      let scannedBlocks = 0;
      let totalBalanceZats = 0;
      const discoveredNotes = [];

      for (let h = startHeight; h <= endHeight; h += 1) {
        scannedBlocks += 1;
        try {
          const resp = await fetch(rpcEndpoint, {
            method: "POST",
            headers: { "content-type": "application/json" },
            body: JSON.stringify({
              jsonrpc: "2.0",
              id: 1,
              method: "getblock",
              params: [h]
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
      // TODO(phase3c): wire full Orchard proving from nozy-wasm core.
      // This preserves the async worker contract now so UI + background can be stable.
      const txid = randomTxidLike();
      const chainId = wasm.get_zcash_chain_id();
      self.postMessage({
        id,
        result: {
          txid,
          chainId,
          rawTxHex: "",
          proving: "placeholder"
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

