(function () {
  // Minimal EIP-1193-like provider injected into dApp pages.
  // Full approval + signing UI will be wired in later phases.

  const makeRequest = (method, params) => {
    return new Promise((resolve, reject) => {
      chrome.runtime.sendMessage(
        {
          type: "NOZY_REQUEST",
          method,
          params: params ?? {}
        },
        (response) => {
          if (chrome.runtime.lastError) {
            reject(new Error(chrome.runtime.lastError.message));
            return;
          }
          if (!response) {
            reject(new Error("No response from background"));
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

  const provider = {
    isNozyWallet: true,
    request: ({ method, params } = {}) => {
      return makeRequest(method, params);
    },
    // Some dApps check selectedAddress/isConnected.
    selectedAddress: null,
    isConnected: () => false
  };

  // Inject into the page window for dApps that look for these.
  window.zcash = provider;
  window.ethereum = provider;
  window.nozy = provider;

  // Signal initialization (EVM wallets commonly dispatch this).
  window.dispatchEvent(new Event("ethereum#initialized"));

  // EIP-6963 provider discovery (best-effort).
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
  } catch (_) {
    // Ignore if CustomEvent or EIP-6963 is not supported.
  }
})();

