(function () {
  const makeRequest = (method, params) => {
    return new Promise((resolve, reject) => {
      chrome.runtime.sendMessage(
        {
          type: "NOZY_REQUEST",
          method,
          params: params ?? {},
          origin: location.origin
        },
        (response) => {
          if (chrome.runtime.lastError) {
            reject(new Error(chrome.runtime.lastError.message));
            return;
          }
          if (!response) {
            reject(new Error("No response from NozyWallet"));
            return;
          }
          if (response.error) {
            reject(new Error(response.error.message));
            return;
          }
          resolve(response.result);
        }
      );
    });
  };

  const listeners = new Map();
  const emit = (event, ...args) => {
    const handlers = listeners.get(event) || [];
    for (const h of handlers) {
      try {
        h(...args);
      } catch (_) {}
    }
  };

  const normalizeRequestArgs = (args) => {
    if (!args || typeof args !== "object") {
      throw new Error("Provider request expects an object argument.");
    }
    return {
      method: args.method,
      params: args.params
    };
  };

  const provider = {
    isNozyWallet: true,
    selectedAddress: null,
    chainId: "0x5ba3",
    request: (args = {}) => {
      const { method, params } = normalizeRequestArgs(args);
      return makeRequest(method, params).then((result) => {
        if (method === "eth_requestAccounts" || method === "zcash_requestAccounts") {
          provider.selectedAddress = Array.isArray(result) && result.length > 0 ? result[0] : null;
          emit("accountsChanged", Array.isArray(result) ? result : []);
          emit("connect", { chainId: provider.chainId });
        }
        return result;
      });
    },
    on: (event, handler) => {
      if (!listeners.has(event)) listeners.set(event, []);
      listeners.get(event).push(handler);
    },
    removeAllListeners: (event) => {
      if (event) listeners.set(event, []);
      else listeners.clear();
    },
    removeListener: (event, handler) => {
      const arr = listeners.get(event) || [];
      listeners.set(
        event,
        arr.filter((h) => h !== handler)
      );
    },
    isConnected: () => true,
    enable: () => provider.request({ method: "eth_requestAccounts" }),
    sendAsync: (payload, cb) => {
      const method = payload?.method;
      const params = payload?.params;
      provider
        .request({ method, params })
        .then((result) => cb?.(null, { id: payload?.id, jsonrpc: "2.0", result }))
        .catch((err) =>
          cb?.(err, {
            id: payload?.id,
            jsonrpc: "2.0",
            error: { message: err?.message || "Provider error" }
          })
        );
    }
  };

  window.zcash = provider;
  window.ethereum = provider;
  window.nozy = provider;

  window.dispatchEvent(new Event("ethereum#initialized"));
  // Ask wallets to announce as per EIP-6963.
  window.addEventListener("eip6963:requestProvider", () => {
    try {
      window.dispatchEvent(
        new CustomEvent("eip6963:announceProvider", {
          detail: {
            uuid: "nozy-wallet",
            name: "NozyWallet",
            rdns: "com.nozywallet",
            provider
          }
        })
      );
    } catch (_) {}
  });
  try {
    window.dispatchEvent(
      new CustomEvent("eip6963:announceProvider", {
        detail: {
          uuid: "nozy-wallet",
          name: "NozyWallet",
          rdns: "com.nozywallet",
          provider
        }
      })
    );
  } catch (_) {}
})();

