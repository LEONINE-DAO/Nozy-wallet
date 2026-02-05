/**
 * NozyWallet Web3 Provider Injection Script
 * This script should be included by dApps to enable Web3 connectivity
 * Usage: <script src="nozy://provider"></script> or include this file
 */

(function() {
  'use strict';

  // Check if we're in an iframe
  const isInIframe = window.self !== window.top;

  // Create the provider object
  function createProvider() {
    let address = null;
    let isConnected = false;
    const listeners = new Map();

    // Listen for provider injection from parent window
    window.addEventListener('message', (event) => {
      // Only accept messages from trusted parent
      if (event.data?.type === 'NOZY_WALLET_PROVIDER_INJECT') {
        address = event.data.provider?.address || null;
        isConnected = !!address;
        
        // Emit accountsChanged event if we have an address
        if (address && listeners.has('accountsChanged')) {
          listeners.get('accountsChanged').forEach(cb => cb([address]));
        }
      }

      // Handle responses to our requests
      if (event.data?.type === 'NOZY_WALLET_RESPONSE') {
        const { requestId, result, error } = event.data;
        const pendingRequest = window.__nozyPendingRequests?.get(requestId);
        if (pendingRequest) {
          if (error) {
            pendingRequest.reject(new Error(error));
          } else {
            pendingRequest.resolve(result);
          }
          window.__nozyPendingRequests.delete(requestId);
        }
      }
    });

    // Request provider from parent
    if (isInIframe && window.parent) {
      window.parent.postMessage({ type: 'NOZY_WALLET_REQUEST_PROVIDER' }, '*');
    }

    const provider = {
      isNozyWallet: true,
      isMetaMask: false,
      
      // EIP-1193 standard methods
      request: async (args) => {
        const { method, params } = args;
        
        // Generate unique request ID
        const requestId = `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
        
        // Initialize pending requests map if needed
        if (!window.__nozyPendingRequests) {
          window.__nozyPendingRequests = new Map();
        }

        return new Promise((resolve, reject) => {
          // Store the promise handlers
          window.__nozyPendingRequests.set(requestId, { resolve, reject });

          // Send request to parent window
          if (isInIframe && window.parent) {
            window.parent.postMessage(
              {
                type: 'NOZY_WALLET_REQUEST',
                method,
                params,
                requestId,
              },
              '*'
            );
          } else {
            // Direct access (same origin)
            reject(new Error('NozyWallet provider not available'));
          }

          // Timeout after 30 seconds
          setTimeout(() => {
            if (window.__nozyPendingRequests?.has(requestId)) {
              window.__nozyPendingRequests.delete(requestId);
              reject(new Error('Request timeout'));
            }
          }, 30000);
        });
      },

      // Event listeners
      on: (event, callback) => {
        if (!listeners.has(event)) {
          listeners.set(event, []);
        }
        listeners.get(event).push(callback);
      },

      removeListener: (event, callback) => {
        if (listeners.has(event)) {
          const callbacks = listeners.get(event);
          const index = callbacks.indexOf(callback);
          if (index > -1) {
            callbacks.splice(index, 1);
          }
        }
      },

      // Legacy methods for compatibility
      send: function(method, params) {
        return this.request({ method, params });
      },

      sendAsync: function(payload, callback) {
        this.request({ method: payload.method, params: payload.params })
          .then(result => callback(null, { result }))
          .catch(error => callback(error, null));
      },

      // Properties
      get selectedAddress() {
        return address;
      },

      get isConnected() {
        return isConnected;
      },

      // Chain ID
      get chainId() {
        return '0x5ba3'; // Zcash mainnet
      },
    };

    return provider;
  }

  // Inject provider into window
  const provider = createProvider();
  
  // Set on window object for compatibility
  if (typeof window !== 'undefined') {
    window.ethereum = provider;
    window.zcash = provider;
    window.nozy = provider;
    
    // Dispatch event for dApps that listen for provider
    window.dispatchEvent(new Event('ethereum#initialized'));
    
    // Also support EIP-6963 provider discovery
    if (window.dispatchEvent) {
      window.dispatchEvent(new CustomEvent('eip6963:announceProvider', {
        detail: {
          info: {
            uuid: 'nozy-wallet',
            name: 'NozyWallet',
            icon: 'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMzIiIGhlaWdodD0iMzIiIHZpZXdCb3g9IjAgMCAzMiAzMiIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPHJlY3Qgd2lkdGg9IjMyIiBoZWlnaHQ9IjMyIiByeD0iOCIgZmlsbD0iI0YwQTExMyIvPgo8L3N2Zz4K',
            rdns: 'com.nozywallet',
          },
          provider: provider,
        },
      }));
    }
  }
})();
