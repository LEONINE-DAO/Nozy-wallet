export function isPublicLwdEndpoint(endpoint) {
  if (!endpoint) return false;
  return endpoint.includes('lwd.nozywallet.org');
}

export async function rpcCall(session, method, params) {
  const response = await fetch(session.rpcEndpoint, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ jsonrpc: '2.0', method, params, id: 1 })
  });
  if (!response.ok) {
    throw new Error(`RPC ${method} HTTP ${response.status}`);
  }
  const data = await response.json();
  return data.result;
}
