/* global fetch */

/**
 * Helper to check if the endpoint is a public LWD / Lightwalletd endpoint
 * @param {string} endpoint 
 * @returns {boolean}
 */
export function isPublicLwdEndpoint(endpoint) {
  if (!endpoint) return false;
  return endpoint.includes('lwd.nozywallet.org');
}

/**
 * Get block count compatible with both Zebra JSON-RPC and public LWD endpoints
 * @param {string} rpcEndpoint 
 * @param {Function} rpcCall 
 * @returns {Promise<number>}
 */
export async function getBlockCountCompatible(rpcEndpoint, rpcCall) {
  if (isPublicLwdEndpoint(rpcEndpoint)) {
    // For public LWD, fetch block height using lightwalletd gRPC-web or companion/http endpoint if available,
    // or fallback to fetching chain tip height via lightwalletd getLatestBlock method equivalent.
    // Here we query the lightwalletd endpoint or fallback safely.
    try {
      const response = await fetch(`${rpcEndpoint}/v1/transparent/getlatestblock`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({})
      });
      if (response.ok) {
        const data = await response.json();
        if (data && data.height) {
          return data.height;
        }
      }
    } catch (e) {
      // ignore and fallback
    }

    // Alternative lightweight fallback for lwd public sync block height
    const altResponse = await fetch(`${rpcEndpoint.replace(/\/+$/, '')}/getblockcount`, {
      method: 'GET'
    });
    if (altResponse.ok) {
      const text = await altResponse.text();
      const height = parseInt(text, 10);
      if (!isNaN(height)) return height;
    }

    // If specific LWD endpoints fail, try standard companion or default height estimation
    throw new Error('Public LWD does not support JSON-RPC getblockcount. Please use a local Zebra node for sending transactions.');
  }

  return await rpcCall('getblockcount', []);
}
