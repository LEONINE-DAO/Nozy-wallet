import initWasm, * as wasm from "../wasm/pkg/nozy_wasm.js";

const STORAGE_KEY = "nozy_wallet_state_v1";

let wasmReady;
let session = {
  unlocked: false,
  mnemonic: null,
  address: null,
  rpcEndpoint: "http://127.0.0.1:8232"
};

const pendingApprovals = new Map();
const providerRequestResolvers = new Map();
let worker;
let workerSeq = 0;
const workerPending = new Map();

async function ensureWasm() {
  if (!wasmReady) {
    wasmReady = initWasm();
  }
  await wasmReady;
  return wasm;
}

function ensureWorker() {
  if (!worker) {
    worker = new Worker(chrome.runtime.getURL("background/wallet-worker.js"), {
      type: "module"
    });
    worker.onmessage = (event) => {
      const { id, result, error } = event.data || {};
      const pending = workerPending.get(id);
      if (!pending) return;
      workerPending.delete(id);
      if (error) pending.reject(new Error(error));
      else pending.resolve(result);
    };
  }
}

function callWorker(method, params) {
  ensureWorker();
  return new Promise((resolve, reject) => {
    const id = `w_${++workerSeq}`;
    workerPending.set(id, { resolve, reject });
    worker.postMessage({ id, method, params });
  });
}

function utf8Encode(str) {
  return new TextEncoder().encode(str);
}

function utf8Decode(bytes) {
  return new TextDecoder().decode(bytes);
}

function ok(result) {
  return { result, error: null };
}

function fail(message) {
  return { result: null, error: { message } };
}

function parseNumberMaybeHex(v) {
  if (v === null || v === undefined) return null;
  if (typeof v === "number" && Number.isFinite(v)) return v;
  if (typeof v === "bigint") return Number(v);
  if (typeof v === "string") {
    const s = v.trim();
    if (s.startsWith("0x")) return Number(BigInt(s));
    if (!s) return null;
    return Number(s);
  }
  return null;
}

function parseTxForOrchardV5(tx) {
  // Minimal adapter: accept common EIP-1193 tx shapes.
  // We only need recipient `to`, `value` (zats) and an optional `memo`.
  const to = tx?.to ?? tx?.recipient ?? tx?.receiver ?? tx?.destination ?? null;
  const value = tx?.value ?? tx?.amount ?? tx?.zatoshis ?? tx?.zats ?? null;
  const memo = tx?.memo ?? tx?.data ?? tx?.comment ?? "";

  const recipientAddress = typeof to === "string" ? to : "";
  const amount = parseNumberMaybeHex(value);
  const memoStr = typeof memo === "string" ? memo : "";

  if (!recipientAddress) {
    throw new Error("Missing transaction recipient (expected tx.to)");
  }
  if (!Number.isFinite(amount) || amount <= 0) {
    throw new Error("Missing/invalid transaction amount (expected tx.value in zats)");
  }

  return { recipientAddress, amount, memo: memoStr };
}

async function waitForTxConfirmation({ rpcEndpoint, txid, timeoutMs = 60_000, pollMs = 2_500 }) {
  const startedAt = Date.now();
  while (true) {
    try {
      const resp = await rpcCall("getrawtransaction", [txid, true]);
      const height = resp?.blockheight ?? resp?.blockHeight ?? resp?.block_height ?? null;
      const bh = typeof height === "number" ? height : parseNumberMaybeHex(height);
      if (Number.isFinite(bh) && bh > 0) return { confirmed: true, blockHeight: bh };
    } catch (_) {
    }

    if (Date.now() - startedAt > timeoutMs) break;
    await new Promise((r) => setTimeout(r, pollMs));
  }
  return { confirmed: false, blockHeight: null };
}

function storageGet(key) {
  return new Promise((resolve) => {
    chrome.storage.local.get(key, (items) => resolve(items[key]));
  });
}

function storageSet(data) {
  return new Promise((resolve) => {
    chrome.storage.local.set(data, () => resolve());
  });
}

async function loadWalletState() {
  const state = await storageGet(STORAGE_KEY);
  return state || null;
}

async function saveWalletState(state) {
  await storageSet({ [STORAGE_KEY]: state });
}

async function walletCreate(password) {
  await ensureWasm();
  const created = wasm.create_wallet(password);
  const mnemonic = created.mnemonic;
  const address = created.address;
  const encryptedMnemonic = Array.from(
    wasm.encrypt_for_storage(utf8Encode(mnemonic), password)
  );

  await saveWalletState({
    encryptedMnemonic,
    address,
    createdAt: Date.now(),
    rpcEndpoint: session.rpcEndpoint
  });

  session.unlocked = true;
  session.mnemonic = mnemonic;
  session.address = address;

  return { address };
}

async function walletRestore(mnemonic, password) {
  await ensureWasm();
  const restored = wasm.restore_wallet(mnemonic, password);
  const address = restored.address;
  const encryptedMnemonic = Array.from(
    wasm.encrypt_for_storage(utf8Encode(mnemonic), password)
  );

  await saveWalletState({
    encryptedMnemonic,
    address,
    createdAt: Date.now(),
    rpcEndpoint: session.rpcEndpoint
  });

  session.unlocked = true;
  session.mnemonic = mnemonic;
  session.address = address;

  return { address };
}

async function walletUnlock(password) {
  await ensureWasm();
  const state = await loadWalletState();
  if (!state?.encryptedMnemonic) {
    throw new Error("No wallet found. Create or restore first.");
  }

  const decrypted = wasm.decrypt_from_storage(
    new Uint8Array(state.encryptedMnemonic),
    password
  );
  const mnemonic = utf8Decode(decrypted);
  const address = wasm.generate_address(mnemonic, 0, 0);

  session.unlocked = true;
  session.mnemonic = mnemonic;
  session.address = address;
  session.rpcEndpoint = state.rpcEndpoint || session.rpcEndpoint;

  return { address };
}

function walletLock() {
  session.unlocked = false;
  session.mnemonic = null;
  return true;
}

async function getWalletStatus() {
  const state = await loadWalletState();
  return {
    exists: !!state,
    unlocked: session.unlocked,
    address: session.address || state?.address || null,
    rpcEndpoint: session.rpcEndpoint
  };
}

async function getAccounts() {
  if (!session.unlocked || !session.address) return [];
  return [session.address];
}

async function requestApproval(kind, payload) {
  const id = crypto.randomUUID();
  const approval = { id, kind, payload, createdAt: Date.now() };
  pendingApprovals.set(id, approval);
  return approval;
}

async function rpcCall(method, params = []) {
  const endpoint = session.rpcEndpoint;
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
  if (!resp.ok) throw new Error(`RPC HTTP ${resp.status}`);
  const body = await resp.json();
  if (body.error) throw new Error(body.error.message || "RPC error");
  return body.result;
}

chrome.runtime.onMessage.addListener((msg, _sender, sendResponse) => {
  if (!msg || msg.type !== "NOZY_REQUEST") return;

  (async () => {
    try {
      await ensureWasm();
      const method = msg.method;
      const params = msg.params ?? {};

      // Popup/UI control methods.
      switch (method) {
        case "wallet_create":
          sendResponse(ok(await walletCreate(params.password)));
          return;
        case "wallet_restore":
          sendResponse(ok(await walletRestore(params.mnemonic, params.password)));
          return;
        case "wallet_unlock":
          sendResponse(ok(await walletUnlock(params.password)));
          return;
        case "wallet_lock":
          sendResponse(ok(walletLock()));
          return;
        case "wallet_status":
          sendResponse(ok(await getWalletStatus()));
          return;
        case "wallet_generate_address":
          if (!session.unlocked || !session.mnemonic) throw new Error("Wallet is locked");
          sendResponse(ok(wasm.generate_address(session.mnemonic, params.account ?? 0, params.index ?? 0)));
          return;
        case "wallet_sign_message":
          if (!session.unlocked || !session.mnemonic) throw new Error("Wallet is locked");
          sendResponse(ok(wasm.sign_message(session.mnemonic, params.message || "")));
          return;
        case "wallet_get_pending_approvals":
          sendResponse(ok(Array.from(pendingApprovals.values())));
          return;
        case "wallet_approve_request": {
          const approval = pendingApprovals.get(params.id);
          if (!approval) throw new Error("Approval request not found");
          pendingApprovals.delete(params.id);
          sendResponse(ok({ approved: true, id: params.id }));

          const resolver = providerRequestResolvers.get(params.id);
          if (resolver) {
            providerRequestResolvers.delete(params.id);
            (async () => {
              try {
                if (approval.kind === "sign") {
                  const message = String(approval.payload?.message ?? "");
                  if (!message) throw new Error("Missing message for signing");
                  const signature = wasm.sign_message(session.mnemonic, message);
                  resolver.sendResponse(ok(signature));
                  return;
                }

                if (approval.kind === "transaction") {
                  const tx = approval.payload?.tx ?? {};
                  const { recipientAddress, amount, memo } = parseTxForOrchardV5(tx);

                  const proving = await callWorker("prove_transaction", {
                    recipientAddress,
                    walletAddress: session.address,
                    mnemonic: session.mnemonic,
                    rpcEndpoint: session.rpcEndpoint,
                    amount,
                    memo
                  });

                  if (!proving?.rawTxHex) {
                    throw new Error("Transaction proving did not return rawTxHex");
                  }

                  const broadcastResult = await rpcCall("sendrawtransaction", [
                    proving.rawTxHex
                  ]);

                  const txid =
                    broadcastResult?.txid ??
                    broadcastResult?.result?.txid ??
                    broadcastResult?.result ??
                    broadcastResult ??
                    proving.txid;

                  const confirmation = await waitForTxConfirmation({
                    rpcEndpoint: session.rpcEndpoint,
                    txid: String(txid),
                    timeoutMs: 120_000,
                    pollMs: 2_000
                  });

                  resolver.sendResponse(ok(String(txid)));
                  return;
                }

                resolver.sendResponse(fail(`Unsupported approval kind: ${approval.kind}`));
              } catch (e) {
                resolver.sendResponse(fail(e?.message ?? "Failed to fulfill approved request"));
              }
            })();
          }
          return;
        }
        case "wallet_reject_request":
          pendingApprovals.delete(params.id);
          if (providerRequestResolvers.has(params.id)) {
            const resolver = providerRequestResolvers.get(params.id);
            providerRequestResolvers.delete(params.id);
            resolver.sendResponse(fail("Request rejected by user"));
          }
          sendResponse(ok({ approved: false, id: params.id }));
          return;
        case "rpc_set_endpoint":
          session.rpcEndpoint = params.url || session.rpcEndpoint;
          {
            const existing = (await loadWalletState()) || {};
            await saveWalletState({ ...existing, rpcEndpoint: session.rpcEndpoint });
          }
          sendResponse(ok({ rpcEndpoint: session.rpcEndpoint }));
          return;
        case "rpc_get_status":
          sendResponse(
            ok({
              endpoint: session.rpcEndpoint,
              connected: !!(await rpcCall("getblockcount", []))
            })
          );
          return;
        case "rpc_get_block_count":
          sendResponse(ok(await rpcCall("getblockcount", [])));
          return;
        case "rpc_get_block":
          sendResponse(ok(await rpcCall("getblock", [params.height])));
          return;
        case "wallet_scan_notes":
          if (!session.unlocked || !session.mnemonic || !session.address) {
            throw new Error("Unlock wallet first.");
          }
          sendResponse(
            ok(
              await callWorker("scan_notes", {
                startHeight: params.startHeight ?? 0,
                endHeight: params.endHeight ?? params.startHeight ?? 0,
                rpcEndpoint: session.rpcEndpoint,
                mnemonic: session.mnemonic,
                address: session.address
              })
            )
          );
          return;
        case "wallet_prove_transaction":
          if (!session.unlocked || !session.address) {
            throw new Error("Unlock wallet first.");
          }
          sendResponse(
            ok(
              await callWorker("prove_transaction", {
                ...params,
                recipientAddress: params?.recipientAddress ?? params?.to ?? session.address,
                walletAddress: session.address,
                mnemonic: session.mnemonic,
                rpcEndpoint: session.rpcEndpoint
              })
            )
          );
          return;
      }

      // dApp provider methods.
      switch (method) {
        case "eth_chainId":
        case "zcash_chainId":
          sendResponse(ok(wasm.get_zcash_chain_id()));
          return;
        case "eth_accounts":
        case "zcash_accounts":
          sendResponse(ok(await getAccounts()));
          return;
        case "eth_requestAccounts":
        case "zcash_requestAccounts": {
          const accounts = await getAccounts();
          if (accounts.length === 0) throw new Error("Unlock wallet in popup first.");
          sendResponse(ok(accounts));
          return;
        }
        case "personal_sign":
        case "zcash_signMessage": {
          if (!session.unlocked) throw new Error("Unlock wallet first.");
          const approval = await requestApproval("sign", {
            method,
            origin: msg.origin || "",
            message: params?.message || params?.[0] || ""
          });
          providerRequestResolvers.set(approval.id, { sendResponse });
          return;
        }
        case "eth_sendTransaction":
        case "zcash_sendTransaction": {
          if (!session.unlocked) throw new Error("Unlock wallet first.");
          const approval = await requestApproval("transaction", {
            method,
            origin: msg.origin || "",
            tx: params?.tx || params?.[0] || params
          });
          providerRequestResolvers.set(approval.id, { sendResponse });
          return;
        }
      }

      sendResponse(fail(`Unsupported method: ${method}`));
    } catch (e) {
      sendResponse(fail(e?.message ?? String(e)));
    }
  })();

  return true;
});

