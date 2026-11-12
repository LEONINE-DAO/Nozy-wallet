/** Live status for the two agent wallets. Signing keys stay in files on this computer. */

import path from "node:path";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import { deskSnapshot, receiveAddress } from "./companion.mjs";
import {
  NEAR_AGENT_ACCOUNT,
  RHEA_CONTRACT,
  RHEA_MARKET_URL,
  RHEA_TOKEN,
  applyShieldText,
  defaultZcashKeyFile,
  finishShield,
  formatUnits,
  idleShield,
  readPublicZcashKey,
  yoctoToNear,
  zatToZec,
  zebraUrl,
} from "./wallets.mjs";

const here = path.dirname(fileURLToPath(import.meta.url));
const nozyBin = process.env.NOZY_BIN
  || path.join(here, "..", "..", "..", "target", "debug", process.platform === "win32" ? "nozy.exe" : "nozy");

let shield = idleShield();
let running = false;

async function rpc(url, body) {
  const response = await fetch(url, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(body),
    signal: AbortSignal.timeout(8000),
  });
  const json = await response.json();
  if (!response.ok || json.error) {
    throw new Error(json.error?.message || "RPC did not answer");
  }
  return json.result;
}

async function zcashStatus(address) {
  const url = zebraUrl();
  try {
    const info = await rpc(url, { jsonrpc: "2.0", id: "1", method: "getblockchaininfo", params: [] });
    const utxos = await rpc(url, {
      jsonrpc: "2.0",
      id: "1",
      method: "getaddressutxos",
      params: [{ addresses: [address] }],
    });
    const zatoshis = (Array.isArray(utxos) ? utxos : []).reduce(
      (sum, coin) => sum + BigInt(coin.satoshis || 0),
      0n,
    );
    return {
      up: true,
      height: info.blocks ?? null,
      balanceZat: zatoshis.toString(),
      balanceZec: zatToZec(zatoshis),
    };
  } catch {
    return { up: false, height: null, balanceZat: "0", balanceZec: null };
  }
}

const NEAR_RPC = "https://rpc.mainnet.fastnear.com";

async function fungibleBalance(contract, decimals) {
  const args = Buffer.from(JSON.stringify({ account_id: NEAR_AGENT_ACCOUNT })).toString("base64");
  const call = await rpc(NEAR_RPC, {
    jsonrpc: "2.0",
    id: "1",
    method: "query",
    params: {
      request_type: "call_function",
      finality: "final",
      account_id: contract,
      method_name: "ft_balance_of",
      args_base64: args,
    },
  });
  const text = Buffer.from(call.result ?? []).toString("utf8").replaceAll('"', "");
  return formatUnits(text || "0", decimals);
}

async function nearStatus() {
  const fullAccess = "Needs a confirm card. This page does not load that key.";
  let near = null;
  let rnear = null;
  let rheaToken = null;
  try {
    const account = await rpc(NEAR_RPC, {
      jsonrpc: "2.0",
      id: "1",
      method: "query",
      params: { request_type: "view_account", finality: "final", account_id: NEAR_AGENT_ACCOUNT },
    });
    near = yoctoToNear(account.amount ?? "0");
  } catch {
    near = null;
  }
  try {
    rnear = await fungibleBalance(RHEA_CONTRACT, 24);
  } catch {
    rnear = null;
  }
  try {
    rheaToken = await fungibleBalance(RHEA_TOKEN, 18);
  } catch {
    rheaToken = null;
  }
  return {
    up: near != null,
    account: NEAR_AGENT_ACCOUNT,
    near,
    rnear,
    rheaToken,
    token: RHEA_TOKEN,
    market: RHEA_MARKET_URL,
    rhea: RHEA_CONTRACT,
    fullAccess,
  };
}

export async function walletSetup() {
  let zcash = { attached: false, address: null, derivation: null };
  try {
    const key = readPublicZcashKey();
    zcash = { attached: true, address: key.address, derivation: key.derivation };
  } catch {
    zcash = { attached: false, address: null, derivation: null };
  }
  const human = await deskSnapshot();
  let destination = null;
  if (human.companion && human.hasPassword === false) {
    destination = await receiveAddress("");
  }
  const chain = zcash.address ? await zcashStatus(zcash.address) : { up: false, height: null, balanceZec: null };
  const near = await nearStatus();
  return {
    computer: true,
    nym: human.nym,
    human: {
      name: human.walletName,
      companion: human.companion,
      unlocked: human.hasPassword === false,
      destination,
    },
    zcash: { ...zcash, ...chain, feeZec: "0.00150000" },
    near,
    shield,
  };
}

export function shieldStatus() {
  return shield;
}

export function startShield() {
  if (running) return { ok: true, shield };
  let keyFile;
  try {
    keyFile = defaultZcashKeyFile();
    readPublicZcashKey(keyFile);
  } catch (error) {
    shield = { ...idleShield(), phase: "error", line: error.message };
    return { ok: false, shield };
  }
  running = true;
  shield = { ...idleShield(), phase: "reading", line: "Reading the transparent coin." };
  const child = spawn(nozyBin, ["-m", "shield-transparent", "--key-file", keyFile], {
    windowsHide: true,
  });
  let errorText = "";
  const take = (chunk) => {
    shield = applyShieldText(shield, chunk.toString("utf8"));
  };
  child.stdout.on("data", take);
  child.stderr.on("data", (chunk) => {
    errorText += chunk.toString("utf8");
    take(chunk);
  });
  child.on("close", (code) => {
    running = false;
    shield = finishShield(shield, code, errorText);
  });
  child.on("error", (error) => {
    running = false;
    shield = finishShield(shield, 1, error.message);
  });
  return { ok: true, shield };
}
