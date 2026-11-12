/** Public wallet facts for the desk. Never returns a mnemonic or a NEAR private key. */

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { profileRoot } from "./companion.mjs";

export const ZCASH_AGENT_ADDRESS = "t1YVKP6WGcvdJYb1XHehkfLYWXvBCt3BcZG";
export const NEAR_AGENT_ACCOUNT = "nozy-agent.near";
export const RHEA_CONTRACT = "lst.rhealab.near";
export const RHEA_TOKEN = "token.rhealab.near";
export const RHEA_MARKET_URL = "https://app.rhea.finance/#near|token.rhealab.near";

export function defaultZcashKeyFile() {
  const fromEnv = String(process.env.NOZY_AGENT_ZCASH_KEYFILE ?? "").trim();
  if (fromEnv) return fromEnv;
  return path.join(os.homedir(), "Nozy-key-backup", "zcash", "agent-zcash.json");
}

export function zebraUrl() {
  const fromEnv = String(process.env.NOZY_ZEBRA_URL ?? "").trim();
  if (fromEnv) return fromEnv.replace(/\/$/, "");
  try {
    const file = path.join(path.dirname(profileRoot()), "config", "config.json");
    const config = JSON.parse(fs.readFileSync(file, "utf8"));
    const url = String(config.zebra_url ?? "").trim();
    if (url) return url.replace(/\/$/, "");
  } catch {
    // Fall through to the operator node used by this desk.
  }
  return "http://172.20.199.206:8232";
}

/** Address only. The mnemonic field is dropped before anything is returned. */
export function publicZcashKey(json) {
  const address = String(json?.transparent_address ?? "").trim();
  if (!/^t1[1-9A-HJ-NP-Za-km-z]+$/.test(address)) {
    throw new Error("Zcash key file has no transparent address.");
  }
  return {
    address,
    derivation: String(json?.derivation_path ?? "m/44'/133'/0'/0/0"),
  };
}

export function readPublicZcashKey(file = defaultZcashKeyFile(), readFile = fs.readFileSync) {
  const json = JSON.parse(readFile(file, "utf8"));
  return publicZcashKey(json);
}

export function zatToZec(zatoshis) {
  const value = BigInt(zatoshis || 0);
  const whole = value / 100_000_000n;
  const frac = (value % 100_000_000n).toString().padStart(8, "0");
  return `${whole.toString()}.${frac}`;
}

export function formatUnits(raw, decimals, places = 4) {
  const value = BigInt(raw || 0);
  const scale = 10n ** BigInt(decimals);
  const whole = value / scale;
  const frac = (value % scale).toString().padStart(Number(decimals), "0").slice(0, places);
  return `${whole.toString()}.${frac}`;
}

export function yoctoToNear(yocto) {
  return formatUnits(yocto, 24);
}

export function idleShield() {
  return {
    phase: "idle",
    line: "The shield waits for a confirm card.",
    txid: null,
    feeZec: null,
    shieldedZec: null,
  };
}

export function applyShieldText(state, text) {
  const next = { ...state };
  for (const raw of String(text).split(/\r?\n/)) {
    const line = raw.trim();
    if (!line) continue;
    next.line = line;
    if (line.startsWith("Reading")) next.phase = "reading";
    if (line.startsWith("Proving")) next.phase = "proving";
    if (line.startsWith("Shield transaction built")) next.phase = "built";
    if (line.startsWith("Broadcast")) next.phase = "broadcast";
    const tx = line.match(/^TXID:\s*(\S+)/);
    if (tx) next.txid = tx[1];
    const fee = line.match(/^Fee:\s*([0-9.]+)/);
    if (fee) next.feeZec = fee[1];
    const value = line.match(/^Shielded value:\s*([0-9.]+)/);
    if (value) next.shieldedZec = value[1];
  }
  return next;
}

export function finishShield(state, code, errorText) {
  if (code === 0 && state.txid) {
    return { ...state, phase: "done", line: `Broadcast ${state.txid}` };
  }
  const reason = String(errorText ?? "")
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)
    .pop();
  return {
    ...state,
    phase: "error",
    line: reason && !/mnemonic|seed|private/i.test(reason) ? reason.slice(0, 300) : "Shield failed.",
  };
}
