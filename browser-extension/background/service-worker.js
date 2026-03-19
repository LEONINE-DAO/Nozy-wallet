import initWasm, * as wasmExports from "../wasm/pkg/nozy_wasm.js";

let wasmReady = null;

async function ensureWasm() {
  if (!wasmReady) {
    wasmReady = (async () => {
      await initWasm();
      // wasmExports contains direct function bindings into the initialized wasm module.
      return wasmExports;
    })();
  }
  return wasmReady;
}

function errorResult(message) {
  return { result: null, error: { message } };
}

function okResult(result) {
  return { result, error: null };
}

chrome.runtime.onMessage.addListener((msg, _sender, sendResponse) => {
  if (!msg || msg.type !== "NOZY_REQUEST") return;

  (async () => {
    try {
      const wasm = await ensureWasm();
      const method = msg.method;
      const params = msg.params ?? {};

      // dApp compatibility: support a minimal EIP-1193 surface for now.
      // We will hook up full approval + signing in a later phase.
      switch (method) {
        case "eth_chainId":
        case "zcash_chainId": {
          const chainId = wasm.get_zcash_chain_id();
          sendResponse(okResult(chainId));
          return;
        }

        case "eth_accounts":
        case "zcash_accounts": {
          // No wallet-connection UI wired yet.
          sendResponse(okResult([]));
          return;
        }

        case "eth_requestAccounts":
        case "zcash_requestAccounts": {
          sendResponse(errorResult("Wallet connection UI not implemented yet."));
          return;
        }

        case "eth_sendTransaction":
        case "zcash_sendTransaction": {
          sendResponse(errorResult("Transaction signing/proving not implemented yet."));
          return;
        }

        case "personal_sign":
        case "zcash_signMessage": {
          sendResponse(errorResult("Message signing not implemented yet."));
          return;
        }

        default: {
          sendResponse(errorResult(`Unsupported method: ${method}`));
          return;
        }
      }
    } catch (e) {
      sendResponse(errorResult(e?.message ?? String(e)));
    }
  })();

  // Tell Chrome we will respond asynchronously.
  return true;
});

