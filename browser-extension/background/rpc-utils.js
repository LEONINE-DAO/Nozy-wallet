/**
 * Shared helpers for JSON-RPC calls to a local Zebra / zcashd node from the extension.
 */

/**
 * @param {unknown} raw
 * @returns {string}
 */
export function normalizeRpcEndpoint(raw) {
  const s = String(raw ?? "").trim();
  if (!s) {
    throw new Error("RPC URL is empty. Set it in Settings (e.g. http://127.0.0.1:8232).");
  }
  let u;
  try {
    u = new URL(s);
  } catch {
    throw new Error(`Invalid RPC URL: ${s}`);
  }
  if (u.protocol !== "http:" && u.protocol !== "https:") {
    throw new Error("RPC URL must use http:// or https://");
  }
  if (!u.hostname) {
    throw new Error("RPC URL must include a hostname (e.g. 127.0.0.1 or your WSL IP).");
  }
  const out = u.toString();
  return out.endsWith("/") ? out.slice(0, -1) : out;
}

/**
 * Turn low-level fetch failures into actionable messages for wallet users.
 * @param {string} endpoint
 * @param {unknown} err
 * @returns {string}
 */
export function rpcNetworkErrorMessage(endpoint, err) {
  const base = String(err?.message ?? err ?? "unknown");
  if (
    base === "Failed to fetch" ||
    /NetworkError|network failed|Load failed|ECONNREFUSED|ERR_CONNECTION|aborted|timed out/i.test(
      base
    )
  ) {
    return (
      `Cannot reach RPC at ${endpoint}. ` +
      `Confirm the node is running and the port matches your config. ` +
      `If the node runs in WSL or Docker, try your VM/container IP instead of 127.0.0.1 from Windows. ` +
      `For Zebra local dev, set enable_cookie_auth=false (or supply auth your node expects).`
    );
  }
  return base;
}
