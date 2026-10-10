const STORAGE_KEY = "nozy_web_api_base";
const STORAGE_CODE = "nozy_web_wallet_code";

export function getApiBase(): string {
  return (
    localStorage.getItem(STORAGE_KEY)?.replace(/\/$/, "") || "http://127.0.0.1:3000"
  );
}

export function getWalletCode(): string {
  return localStorage.getItem(STORAGE_CODE) ?? "";
}

export function setWalletCode(code: string) {
  const trimmed = code.trim();
  if (trimmed) localStorage.setItem(STORAGE_CODE, trimmed);
  else localStorage.removeItem(STORAGE_CODE);
}

export function needsRegisterCode(err: unknown): boolean {
  const raw = err instanceof Error ? err.message : String(err ?? "");
  return /api key|x-api-key|authorization:\s*bearer/i.test(raw);
}

export function merchantMessage(err: unknown, fallback: string): string {
  const raw = err instanceof Error ? err.message : fallback;
  if (needsRegisterCode(err)) {
    return "This register needs its wallet code. Paste it once, then open your wallet.";
  }
  if (/failed to fetch|networkerror|load failed/i.test(raw)) {
    return "Can't reach your wallet on this computer. Open Nozy, then try again.";
  }
  if (/switch to business/i.test(raw)) {
    return "Switch to your business wallet before taking a payment.";
  }
  return raw || fallback;
}

export function setApiBase(url: string) {
  localStorage.setItem(STORAGE_KEY, url.replace(/\/$/, ""));
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const code = getWalletCode();
  const res = await fetch(`${getApiBase()}${path}`, {
    ...init,
    headers: {
      "Content-Type": "application/json",
      ...(code ? { "X-API-Key": code } : {}),
      ...(init?.headers ?? {}),
    },
  });
  if (!res.ok) {
    let msg = res.statusText;
    try {
      const j = await res.json();
      msg = j.error || j.message || msg;
    } catch {
      /* ignore */
    }
    throw new Error(msg);
  }
  return res.json() as Promise<T>;
}

export type Profile = {
  role: string;
  orchard_account: number;
  business_display_name?: string;
  linked_zns_name?: string;
  linked_zns_display?: string;
  receive_address?: string;
  business_address?: string;
  personal_address?: string;
  network: string;
};

export const api = {
  getProfile: (password?: string) => {
    const q = password ? `?password=${encodeURIComponent(password)}` : "";
    return request<Profile>(`/api/profile${q}`);
  },
  updateProfile: (body: {
    password?: string;
    role?: string;
    business_display_name?: string;
  }) =>
    request<Profile>("/api/profile", {
      method: "POST",
      body: JSON.stringify(body),
    }),
  linkZns: (body: { name: string; password?: string }) =>
    request<{
      linked: boolean;
      name: string;
      display: string;
      address: string;
      business_address: string;
    }>("/api/zns/link", { method: "POST", body: JSON.stringify(body) }),
  unlinkZns: () =>
    request<{ linked: boolean }>("/api/zns/link", { method: "DELETE" }),
  getBalance: () =>
    request<{ balance_zec: number; available_zec?: number }>("/api/balance"),
  sync: (password?: string) =>
    request<{ success: boolean }>("/api/sync", {
      method: "POST",
      body: JSON.stringify({ password: password ?? null }),
    }),
  createInvoice: (body: {
    amount_zec: number;
    product_name?: string;
    memo?: string;
    password?: string;
  }) =>
    request<MerchantInvoice>("/api/business/invoices", {
      method: "POST",
      body: JSON.stringify(body),
    }),
  listInvoices: (limit = 20) =>
    request<{ invoices: MerchantInvoice[] }>(`/api/business/invoices?limit=${limit}`),
  getInvoice: (id: string) =>
    request<MerchantInvoice>(`/api/business/invoices/${encodeURIComponent(id)}`),
  cancelInvoice: (id: string) =>
    request<MerchantInvoice>(`/api/business/invoices/${encodeURIComponent(id)}/cancel`, {
      method: "POST",
      body: JSON.stringify({}),
    }),
  createGiftCard: (body: { amount_zec: number; label?: string; password?: string }) =>
    request<GiftCardCreated>("/api/gift-cards", {
      method: "POST",
      body: JSON.stringify(body),
    }),
  listGiftCards: (limit = 20) =>
    request<{ gift_cards: GiftCard[] }>(`/api/gift-cards?limit=${limit}`),
  cancelGiftCard: (id: string) =>
    request<GiftCard>(`/api/gift-cards/${encodeURIComponent(id)}/cancel`, {
      method: "POST",
      body: JSON.stringify({}),
    }),
  redeemGiftCard: (body: { claim_code: string; password?: string }) =>
    request<GiftCard>("/api/gift-cards/redeem", {
      method: "POST",
      body: JSON.stringify(body),
    }),
};

export type GiftCardStatus = "unfunded" | "funded" | "redeeming" | "redeemed" | "cancelled";

export type GiftCard = {
  gift_id: string;
  status: GiftCardStatus;
  payment_address: string;
  amount_zatoshis: number;
  amount_zec: number;
  label?: string;
  created_at: string;
  zcash_uri: string;
  funding_txid?: string;
  redeem_txid?: string;
  recipient?: string;
  payout_zatoshis?: number;
};

export type GiftCardCreated = GiftCard & { claim_code: string };

export type InvoiceStatus = "open" | "detected" | "confirmed" | "expired" | "cancelled";

export type MerchantInvoice = {
  invoice_id: string;
  status: InvoiceStatus;
  payment_address: string;
  amount_zatoshis: number;
  amount_zec: number;
  product_name?: string;
  memo?: string;
  created_at: string;
  expires_at: string;
  zcash_uri: string;
  detected_txid?: string;
  confirmed_txid?: string;
};
