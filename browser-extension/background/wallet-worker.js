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

self.onmessage = async (event) => {
  const { id, method, params } = event.data || {};
  try {
    await ensureReady();

    if (method === "scan_notes") {
      const startHeight = Number(params?.startHeight ?? 0);
      const endHeight = Number(params?.endHeight ?? startHeight);
      const scannedBlocks = Math.max(0, endHeight - startHeight + 1);
      self.postMessage({
        id,
        result: {
          scannedBlocks,
          discoveredNotes: [],
          totalBalanceZats: 0
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

