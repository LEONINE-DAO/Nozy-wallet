/**
 * NozyPay hosted merchant API client (shielded ZEC / Ironwood only).
 * https://github.com/Lowo88/NozyPay
 *
 * Invoice create/status for the merchant dashboard. Customer pay path is ZIP-321
 * in the wallet (`zip321.ts`) — do not pull ZGo’s API into NozyWallet.
 */

export type NozyPayInvoice = {
  id: string;
  status: string;
  payment_address: string;
  amount_zec: number;
  amount_zatoshis: number;
  zcash_uri: string;
  settle_pool: string;
  memo?: string | null;
  product_name?: string | null;
  expires_at: string;
};

function normalizeBase(url: string): string {
  return url.trim().replace(/\/+$/, "");
}

export async function nozypayHealth(baseUrl: string): Promise<{
  shielded_only: boolean;
  settle_pool: string;
  currency: string;
  transparent_zec: boolean;
}> {
  const res = await fetch(`${normalizeBase(baseUrl)}/health`);
  if (!res.ok) throw new Error(`NozyPay health failed (${res.status})`);
  return res.json();
}

export async function nozypayCreateInvoice(
  baseUrl: string,
  body: {
    payment_address: string;
    amount_zec: number;
    memo?: string;
    product_name?: string;
    amount_fiat?: number;
    fiat_currency?: string;
  },
): Promise<NozyPayInvoice> {
  if (/^t[123m]/i.test(body.payment_address.trim())) {
    throw new Error("Transparent ZEC is not supported by NozyPay");
  }
  const res = await fetch(`${normalizeBase(baseUrl)}/api/invoices`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(body),
  });
  const data = await res.json().catch(() => ({}));
  if (!res.ok) {
    throw new Error(data.error || `Create invoice failed (${res.status})`);
  }
  return data as NozyPayInvoice;
}

export async function nozypayGetInvoice(
  baseUrl: string,
  id: string,
): Promise<NozyPayInvoice> {
  const res = await fetch(`${normalizeBase(baseUrl)}/api/invoices/${id}`);
  const data = await res.json().catch(() => ({}));
  if (!res.ok) {
    throw new Error(data.error || `Get invoice failed (${res.status})`);
  }
  return data as NozyPayInvoice;
}
