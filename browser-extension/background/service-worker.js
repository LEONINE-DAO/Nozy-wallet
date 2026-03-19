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
          return;
        }
        case "wallet_reject_request":
          pendingApprovals.delete(params.id);
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
          sendResponse(
            ok(
              await callWorker("scan_notes", {
                startHeight: params.startHeight ?? 0,
                endHeight: params.endHeight ?? params.startHeight ?? 0
              })
            )
          );
          return;
        case "wallet_prove_transaction":
          sendResponse(ok(await callWorker("prove_transaction", params)));
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
          sendResponse(ok({ pendingApproval: approval.id }));
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
          sendResponse(ok({ pendingApproval: approval.id }));
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

