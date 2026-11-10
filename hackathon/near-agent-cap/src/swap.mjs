/** NEAR Intents quote. Dry quotes do not create a deposit. Nothing is signed here. */

export const QUOTE_URL = "https://1click.chaindefuser.com/v0/quote";
const QUOTE_HOST = "1click.chaindefuser.com";
const RPC_HOSTS = {
  mainnet: "https://rpc.mainnet.fastnear.com",
  testnet: "https://rpc.testnet.near.org",
};

function allowedUrl(url, host) {
  try {
    return new URL(url).host === host;
  } catch {
    return false;
  }
}

export async function viewAccount(accountId, { fetchImpl = fetch, rpcUrl } = {}) {
  const network = String(accountId).endsWith(".testnet") ? "testnet" : "mainnet";
  const url = rpcUrl ?? RPC_HOSTS[network];
  const host = new URL(RPC_HOSTS[network]).host;
  if (!allowedUrl(url, host)) {
    return { exists: false, reason: "account lookup stays on the NEAR RPC." };
  }
  const response = await fetchImpl(url, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: "1",
      method: "query",
      params: { request_type: "view_account", finality: "final", account_id: accountId },
    }),
  });
  const json = await response.json();
  if (json.error) return { exists: false };
  return { exists: true, amount: json.result?.amount ?? null };
}

/**
 * Ask 1Click for a quote. The host is fixed. A dry quote has no deposit address.
 */
export async function requestQuote(request, { fetchImpl = fetch, url = QUOTE_URL, token } = {}) {
  if (!allowedUrl(url, QUOTE_HOST)) {
    return { ok: false, reason: "NEAR Intents quotes stay on 1click.chaindefuser.com." };
  }
  const headers = { "content-type": "application/json" };
  const session = String(token ?? "").trim();
  if (session) headers.authorization = `Bearer ${session}`;
  const response = await fetchImpl(url, {
    method: "POST",
    headers,
    body: JSON.stringify(request),
  });
  const json = await response.json().catch(() => ({}));
  if (!response.ok) {
    return { ok: false, reason: String(json.message || "NEAR Intents did not quote this pair.") };
  }
  const quote = json.quote ?? {};
  return {
    ok: true,
    unsigned: true,
    egress: "clearnet",
    quote: {
      dry: Boolean(request.dry),
      amountIn: quote.amountInFormatted ?? quote.amountIn ?? request.amount,
      amountOut: quote.amountOutFormatted ?? quote.amountOut ?? null,
      minAmountOut: quote.minAmountOut ?? null,
      depositAddress: quote.depositAddress ?? null,
    },
  };
}
