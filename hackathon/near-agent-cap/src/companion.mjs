/** Talks to the local Nozy companion. Never returns a seed, password, or API key. */

import fs from "node:fs";
import os from "node:os";
import path from "node:path";

const BASE = (process.env.NOZY_API ?? "http://127.0.0.1:3000").replace(/\/$/, "");

export function profileRoot() {
  const override = String(process.env.NOZY_WALLET_DATA ?? "").trim();
  if (override) return override;
  const appData = process.env.APPDATA || path.join(os.homedir(), "AppData", "Roaming");
  return path.join(appData, "nozy", "nozy", "data");
}

/** The companion key stays on this machine. Callers must not log the return value. */
export function loadCompanionKey({
  envKey = process.env.NOZY_API_KEY,
  root = profileRoot(),
  readFile = (file) => fs.readFileSync(file, "utf8"),
} = {}) {
  const fromEnv = String(envKey ?? "").trim();
  if (fromEnv) return fromEnv;

  let active = "";
  try {
    const manifest = JSON.parse(readFile(path.join(root, "profiles", "manifest.json")));
    active = String(manifest.active_id ?? "");
  } catch {
    active = "";
  }

  const files = [];
  if (/^[A-Za-z0-9_-]+$/.test(active)) {
    files.push(path.join(root, "profiles", active, "companion_api_key"));
  }
  files.push(path.join(root, "companion_api_key"));
  for (const file of files) {
    try {
      const text = String(readFile(file) ?? "").trim();
      if (text) return text;
    } catch {
      // Try the next location.
    }
  }
  return "";
}

export function activeWalletName({
  root = profileRoot(),
  readFile = (file) => fs.readFileSync(file, "utf8"),
} = {}) {
  try {
    const manifest = JSON.parse(readFile(path.join(root, "profiles", "manifest.json")));
    const active = String(manifest.active_id ?? "");
    const profile = (manifest.profiles ?? []).find((item) => item.id === active);
    return profile?.name ? String(profile.name) : null;
  } catch {
    return null;
  }
}

export function companionHeaders(key, hasBody) {
  const headers = {};
  if (hasBody) headers["content-type"] = "application/json";
  if (key) headers["X-API-Key"] = key;
  return headers;
}

async function call(requestPath, { method = "GET", body } = {}) {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), 8000);
  try {
    const response = await fetch(`${BASE}${requestPath}`, {
      method,
      headers: companionHeaders(loadCompanionKey(), Boolean(body)),
      body: body ? JSON.stringify(body) : undefined,
      signal: controller.signal,
    });
    const text = await response.text();
    let json = null;
    if (text) {
      try {
        json = JSON.parse(text);
      } catch {
        json = { error: "Companion returned a response this desk cannot use." };
      }
    }
    return { status: response.status, json };
  } catch {
    return {
      status: 0,
      json: { error: "Nozy companion is not running at 127.0.0.1:3000." },
    };
  } finally {
    clearTimeout(timer);
  }
}

function errorText(json, fallback) {
  if (json && typeof json === "object" && !Array.isArray(json) && json.error) {
    return String(json.error);
  }
  return fallback;
}

/** Active-role unified address. The seed and password are not returned. */
export async function receiveAddress(password = "") {
  const query = new URLSearchParams({ password: String(password ?? "") });
  const profile = await call(`/api/profile?${query}`);
  if (profile.status !== 200) return null;
  const address = profile.json?.receive_address;
  return typeof address === "string" && address.startsWith("u") ? address : null;
}

/** Unlock succeeds or fails. The companion's seed preview is discarded. */
export function unlockView(status, json) {
  if (status === 200) return { ok: true, unlocked: true };
  return { ok: false, reason: errorText(json, "Could not unlock the human wallet.") };
}

export async function deskSnapshot() {
  const exists = await call("/api/wallet/exists");
  if (exists.status === 0) {
    return {
      companion: false,
      exists: false,
      role: null,
      balanceZec: null,
      walletName: activeWalletName(),
      nym: "Companion offline",
    };
  }

  const profile = await call("/api/profile");
  const status = await call("/api/wallet/status");
  const egress = await call("/api/privacy/send-egress");
  const vpn = await call("/api/privacy/nym-vpn-app");

  let nym = "Required off-box";
  if (egress.status === 200 && egress.json?.label) {
    nym = String(egress.json.label);
  }
  if (vpn.status === 200 && vpn.json?.connected) {
    nym = `Nym ${vpn.json.mode || "connected"}`;
  }

  return {
    companion: true,
    exists: Boolean(exists.json?.exists),
    hasPassword: Boolean(exists.json?.has_password),
    walletName: activeWalletName(),
    role: profile.status === 200 ? profile.json?.role ?? null : null,
    network: profile.status === 200 ? profile.json?.network ?? null : null,
    balanceZec: status.status === 200 ? status.json?.confirmed_zec ?? null : null,
    nym,
  };
}

/** Public invoice fields only. Release follows the companion status. */
export function invoiceView(status, json) {
  if (status === 0) {
    return { ok: false, reason: errorText(json, "Nozy companion is not running.") };
  }
  if (status !== 200 || !json || typeof json !== "object" || Array.isArray(json)) {
    return { ok: false, reason: errorText(json, "Invoice not found.") };
  }
  const state = String(json.status ?? "");
  return {
    ok: true,
    invoice: {
      invoice_id: json.invoice_id ?? null,
      status: state,
      amount_zec: json.amount_zec ?? null,
      payment_address: json.payment_address ?? null,
      zcash_uri: json.zcash_uri ?? null,
      detected_txid: json.detected_txid ?? null,
      confirmed_txid: json.confirmed_txid ?? null,
      release: state === "confirmed",
    },
  };
}

export async function invoiceStatus(id) {
  if (!/^[A-Za-z0-9_-]+$/.test(String(id ?? ""))) {
    return { ok: false, reason: "Invoice id is not valid." };
  }
  const response = await call(`/api/business/invoices/${id}`);
  return invoiceView(response.status, response.json);
}

function writeActiveCompanionKey(key) {
  const trimmed = String(key ?? "").trim();
  if (!trimmed) return;
  let active = "";
  try {
    const manifest = JSON.parse(fs.readFileSync(path.join(profileRoot(), "profiles", "manifest.json"), "utf8"));
    active = String(manifest.active_id ?? "");
  } catch {
    return;
  }
  if (!/^[A-Za-z0-9_-]+$/.test(active)) return;
  const file = path.join(profileRoot(), "profiles", active, "companion_api_key");
  if (fs.existsSync(file)) return;
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, `${trimmed}\n`);
}

/** New profile. The seed preview from the companion is discarded. */
export async function createDeskWallet(password) {
  const key = loadCompanionKey();
  const created = await call("/api/wallet/create", {
    method: "POST",
    body: { password },
  });
  if (created.status !== 200) {
    return { ok: false, reason: errorText(created.json, "Could not create the wallet.") };
  }
  writeActiveCompanionKey(key);
  const unlocked = await unlockWallet(password);
  return { ...unlocked, created: true, walletName: activeWalletName() };
}

export async function unlockWallet(password) {
  const response = await call("/api/wallet/unlock", {
    method: "POST",
    body: { password },
  });
  return unlockView(response.status, response.json);
}

export async function draftInvoice({ amount, memo, password }) {
  const zec = Number(amount);
  if (!password) {
    return { ok: false, reason: "Unlock the human wallet before drafting an invoice." };
  }
  if (!Number.isFinite(zec) || zec <= 0) {
    return { ok: false, reason: "amount must be a positive ZEC decimal" };
  }

  const profile = await call("/api/profile");
  if (profile.status === 0) {
    return { ok: false, reason: errorText(profile.json, "Nozy companion is not running.") };
  }
  const previous = profile.json?.role ?? "personal";

  if (previous !== "business") {
    const switched = await call("/api/profile", {
      method: "POST",
      body: { role: "business", password },
    });
    if (switched.status !== 200) {
      return { ok: false, reason: errorText(switched.json, "Could not use the business profile.") };
    }
  }

  try {
    const created = await call("/api/business/invoices", {
      method: "POST",
      body: {
        amount_zec: zec,
        memo: memo || "Human pays from the shielded wallet",
        product_name: "Nozy agent draft",
        password,
      },
    });
    if (created.status !== 200) {
      return { ok: false, reason: errorText(created.json, "Could not draft the invoice.") };
    }
    const invoice = created.json ?? {};
    return {
      ok: true,
      invoice: {
        action: "draft_invoice",
        invoice_id: invoice.invoice_id ?? null,
        amount_zec: String(invoice.amount_zec ?? zec),
        status: invoice.status ?? "open",
        payer: "human Nozy wallet",
        agent_may_sign: false,
        payment_address: invoice.payment_address ?? null,
        zcash_uri: invoice.zcash_uri ?? null,
        expires_at: invoice.expires_at ?? null,
        settlement: "The human wallet pays this shielded invoice. The agent cannot sign it.",
      },
    };
  } finally {
    if (previous !== "business") {
      await call("/api/profile", {
        method: "POST",
        body: { role: previous, password },
      });
    }
  }
}
