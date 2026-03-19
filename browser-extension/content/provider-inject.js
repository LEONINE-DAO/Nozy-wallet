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
  const provider = {
    isNozyWallet: true,
    selectedAddress: null,
    chainId: "0x5ba3",
    request: ({ method, params } = {}) => makeRequest(method, params),
    on: (event, handler) => {
      if (!listeners.has(event)) listeners.set(event, []);
      listeners.get(event).push(handler);
    },
    removeListener: (event, handler) => {
      const arr = listeners.get(event) || [];
      listeners.set(
        event,
        arr.filter((h) => h !== handler)
      );
    },
    isConnected: () => true
  };

  window.zcash = provider;
  window.ethereum = provider;
  window.nozy = provider;

  window.dispatchEvent(new Event("ethereum#initialized"));
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

