/**
 * NEAR-only agent.
 *
 * The model is NEAR AI Cloud (`nearai/cloud-api`). It may propose.
 * The local policy decides. Nothing is signed here.
 * Off-box HTTP is allowlisted to cloud-api.near.ai, the same rule as
 * IronClaw's endpoint allowlist (`nearai/ironclaw`). The deprecated
 * `nearai/nearai` hub is not used. Other chains are other agents.
 */

import { act } from "./act.mjs";
import { requestQuote, viewAccount } from "./swap.mjs";

export const NEAR_ACCOUNT = "nozy-agent.testnet";
export const MAINNET_ACCOUNT = "nozy-agent.near";
export const NEAR_INFERENCE_HOST = "cloud-api.near.ai";
export const NEAR_INFERENCE = `https://${NEAR_INFERENCE_HOST}/v1/chat/completions`;
const MODEL = "zai-org/GLM-5.1-FP8";

const FOREIGN = ["ethereum", "solana", "bitcoin", "namada", "osmosis", "penumbra"];

const TOOLS = [
  {
    type: "function",
    function: {
      name: "stake",
      description: "Propose staking NEAR for rNEAR on the Rhea testnet contract.",
      parameters: {
        type: "object",
        properties: { amount: { type: "string" } },
        required: ["amount"],
      },
    },
  },
  {
    type: "function",
    function: {
      name: "pay",
      description: "Propose a NEAR transfer. The policy refuses a stranger.",
      parameters: {
        type: "object",
        properties: {
          to: { type: "string" },
          amount: { type: "string" },
        },
        required: ["to", "amount"],
      },
    },
  },
  {
    type: "function",
    function: {
      name: "hire",
      description:
        "Propose an unsigned NEAR Intents payment to another agent for api, compute, or data. Do not use this for shielded ZEC.",
      parameters: {
        type: "object",
        properties: {
          to: { type: "string" },
          amount: { type: "string" },
          purpose: { type: "string", enum: ["api", "compute", "data"] },
        },
        required: ["to", "amount", "purpose"],
      },
    },
  },
];

export function interpret(text) {
  const lower = String(text ?? "").trim().toLowerCase();
  const number = lower.match(/(\d+(?:\.\d+)?)/);
  const amount = number ? number[1] : undefined;
  const foreign = FOREIGN.find((chain) => lower.includes(chain));
  if (foreign) return { chain: foreign, kind: "foreign" };
  const purpose = ["api", "compute", "data"].find((item) => lower.includes(item)) ?? "";
  const payee = lower.match(/\b([a-z0-9][a-z0-9\-_]{0,62}[a-z0-9]\.near)\b/)?.[1] ?? "";
  if (
    /\b(zec|shielded)\b/.test(lower) &&
    /\b(swap|intents?|hire|negotiate)\b/.test(lower) &&
    (/\bnear\b/.test(lower) || payee.endsWith(".near"))
  ) {
    return { chain: "near", kind: "zec-intent" };
  }
  if (/\b(zec|shielded)\b/.test(lower) && /\b(swap|intents?)\b/.test(lower)) {
    return { chain: "near", kind: "zec-intent" };
  }
  if (/\bhire\b/.test(lower)) {
    return { chain: "near", kind: "hire", amount: amount || "1", to: payee, purpose };
  }
  if (/\bnegotiate\b/.test(lower)) {
    const offer = lower.match(/\boffer\s+(\d+(?:\.\d+)?)/)?.[1];
    const counter = lower.match(/\bcounter\s+(\d+(?:\.\d+)?)/)?.[1];
    const zec = /\b(zec|shielded)\b/.test(lower);
    return {
      chain: zec ? "zcash" : "near",
      kind: "negotiate",
      amount: counter || offer || amount || "1",
      offer,
      counter,
      to: zec ? "" : payee,
      purpose,
    };
  }
  if (/\bearn\b/.test(lower)) {
    const zec = /\b(zec|shielded)\b/.test(lower);
    return { chain: zec ? "zcash" : "near", kind: "earn", amount: amount || "1", purpose };
  }
  if (/\bspend\b/.test(lower) && /\b(zec|shielded)\b/.test(lower)) {
    return { chain: "zcash", kind: "frost-spend", amount: amount || "0.01", to: payee };
  }
  if (/\b(swap|intents?)\b/.test(lower)) {
    const named = [...lower.matchAll(/\b(near|wnear|usdc|usdt|rhea)\b/g)].map((match) => match[1]);
    const toWord = lower.match(/\bto\s+(near|wnear|usdc|usdt|rhea)\b/)?.[1];
    const fromWord = named.find((word) => word !== toWord) ?? "near";
    return {
      chain: "near",
      kind: "swap",
      amount: amount || "1",
      from: (fromWord === "wnear" ? "near" : fromWord).toUpperCase(),
      to: (toWord === "wnear" ? "near" : toWord ?? "usdc").toUpperCase(),
      confidentiality: /\b(private|confidential)\b/.test(lower) ? "basic" : "public",
    };
  }
  if (lower.includes("invoice") || lower.includes("zcash") || /\bzec\b/.test(lower)) {
    return {
      chain: "zcash",
      kind: "invoice",
      amount: amount || "0.01",
      memo: "Human pays from the shielded wallet",
    };
  }
  if (lower.includes("stranger") || lower.includes("pay") || lower.includes("send")) {
    return { chain: "near", kind: "pay", amount: amount || "1", to: "stranger.near" };
  }
  const network = lower.includes("testnet") ? "testnet" : "mainnet";
  if (lower.includes("unstake all")) return { chain: "near", kind: "unstake_all", network };
  if (lower.includes("unstake")) return { chain: "near", kind: "unstake", amount: amount || "1", network };
  if (lower.includes("withdraw all")) return { chain: "near", kind: "withdraw_all", network };
  if (lower.includes("withdraw")) return { chain: "near", kind: "withdraw", amount: amount || "1", network };
  if (lower.includes("storage")) return { chain: "near", kind: "storage", amount: amount || "0.00125", network };
  if (lower.includes("stake all")) return { chain: "near", kind: "stake_all", network };
  if (lower.includes("stake")) return { chain: "near", kind: "stake", amount: amount || "1", network };
  if (lower.includes("nym")) return { chain: "near", kind: "disable-nym" };
  return { chain: "near", kind: "unknown" };
}

/** Local decision. A handoff means another agent owns the action. */
export function propose(input) {
  const read = input.text
    ? interpret(input.text)
    : {
        chain: input.chain ?? "near",
        kind: input.kind,
        amount: input.amount,
        to: input.to,
        from: input.from,
        offer: input.offer,
        counter: input.counter,
        purpose: input.purpose,
        memo: input.memo,
        network: input.network,
        confidentiality: input.confidentiality,
      };

  if (read.kind === "foreign") {
    return {
      ok: false,
      agent: null,
      chain: read.chain,
      reason: `No agent is running for ${read.chain}.`,
    };
  }

  if (read.kind === "frost-spend" || (read.chain === "zcash" && read.kind === "spend")) {
    if (String(read.to ?? "").endsWith(".near")) {
      return {
        ok: false,
        agent: "zcash",
        reason: "The Zcash agent does not pay through NEAR Intents.",
      };
    }
    if (!input.confirmed) {
      return {
        ok: false,
        agent: "zcash",
        needsConfirm: true,
        frost: { threshold: 2, agentHoldsShare: false, pool: "orchard" },
        reason: "A shielded FROST spend waits for the confirm card.",
      };
    }
    return {
      ok: true,
      agent: "zcash",
      unsigned: true,
      broadcast: false,
      rail: "zcash",
      amount: read.amount,
      frost: { threshold: 2, agentHoldsShare: false, pool: "orchard" },
      reason:
        "The Zcash wallet broadcasts this after two FROST signers authorize the v5 transaction. This desk holds no spend share.",
    };
  }

  if (read.chain === "zcash" && read.kind === "earn") {
    if (!["api", "compute", "data"].includes(read.purpose)) {
      return {
        ok: false,
        agent: "zcash",
        reason: "hire purpose must be api, compute, or data",
      };
    }
    return {
      ok: true,
      agent: "zcash",
      unsigned: true,
      signer: null,
      role: "earn",
      rail: "zcash",
      invoice: {
        asset: "ZEC",
        amount: read.amount,
        purpose: read.purpose,
        payer: "human Nozy wallet",
        agentMaySign: false,
      },
    };
  }

  if (read.chain === "zcash" && read.kind === "negotiate") {
    if (String(read.to ?? "").endsWith(".near")) {
      return {
        ok: false,
        agent: "zcash",
        reason: "The Zcash agent does not pay through NEAR Intents.",
      };
    }
    const offer = Number(read.offer);
    const counter = Number(read.counter);
    if (!Number.isFinite(offer) || !Number.isFinite(counter) || offer <= 0 || counter <= 0) {
      return { ok: false, agent: "zcash", reason: "amount must be a positive decimal" };
    }
    if (counter > offer && !input.confirmed) {
      return {
        ok: false,
        agent: "zcash",
        needsConfirm: true,
        reason: "the counter is above the offer",
      };
    }
    const accepted = String(counter <= offer ? read.counter : read.offer);
    if (!input.confirmed) {
      return {
        ok: false,
        agent: "zcash",
        needsConfirm: true,
        amount: accepted,
        rail: "zcash",
        frost: { threshold: 2, agentHoldsShare: false, pool: "orchard" },
        reason: "A shielded FROST spend waits for the confirm card.",
      };
    }
    return {
      ok: true,
      agent: "zcash",
      role: "negotiate",
      unsigned: true,
      broadcast: false,
      rail: "zcash",
      amount: accepted,
      frost: { threshold: 2, agentHoldsShare: false, pool: "orchard" },
      reason:
        "The Zcash wallet broadcasts the accepted amount after two FROST signers authorize it. This desk holds no spend share.",
    };
  }

  if (read.chain === "zcash" || read.kind === "invoice") {
    return {
      ok: false,
      agent: "zcash",
      handoff: "invoice",
      amount: read.amount,
      memo: read.memo,
      reason: "The Zcash agent creates the shielded invoice. The NEAR agent does not sign it.",
    };
  }

  if (read.kind === "zec-intent") {
    return {
      ok: false,
      agent: "near",
      account: NEAR_ACCOUNT,
      reason: "Shielded ZEC does not go through NEAR Intents.",
    };
  }

  if (read.kind === "swap" || read.kind === "hire" || read.kind === "negotiate" || read.kind === "earn") {
    return {
      ...act({
        kind: read.kind,
        amount: read.amount,
        from: read.from,
        to: read.to,
        offer: read.offer,
        counter: read.counter,
        purpose: read.purpose,
        confirmed: input.confirmed,
        live: input.live,
        deadline: input.deadline,
        confidentiality: read.confidentiality,
      }),
      agent: "near",
    };
  }

  const decision = act({
    kind: read.kind,
    amount: read.amount,
    to: read.to,
    memo: read.memo,
    network: read.network ?? input.network,
    confirmed: input.confirmed,
    useHuman: input.useHuman,
  });
  return {
    ...decision,
    agent: "near",
    account: decision.tx?.signerId ?? NEAR_ACCOUNT,
    model: false,
  };
}

export function toolToAction(toolCall) {
  const name = toolCall?.function?.name;
  let args = {};
  try {
    args = JSON.parse(toolCall?.function?.arguments ?? "{}");
  } catch {
    return { chain: "near", kind: "unknown" };
  }
  if (name === "stake") return { chain: "near", kind: "stake", amount: String(args.amount ?? "") };
  if (name === "pay") {
    return { chain: "near", kind: "pay", amount: String(args.amount ?? ""), to: String(args.to ?? "") };
  }
  if (name === "hire") {
    return {
      chain: "near",
      kind: "hire",
      amount: String(args.amount ?? ""),
      to: String(args.to ?? ""),
      purpose: String(args.purpose ?? ""),
    };
  }
  return { chain: "near", kind: "unknown" };
}

/**
 * One NEAR AI Cloud completion. The returned tool is not executed here.
 * The key is never copied into the result.
 */
export function inferenceUrl(url = NEAR_INFERENCE) {
  const host = new URL(url).host;
  if (host !== NEAR_INFERENCE_HOST) {
    return { ok: false, reason: "NEAR agent HTTP is allowlisted to cloud-api.near.ai." };
  }
  return { ok: true, url };
}

export async function requestModelProposal(text, { apiKey, fetchImpl = fetch, model = MODEL, url = NEAR_INFERENCE } = {}) {
  if (!apiKey) {
    return { ok: false, model: false, reason: "NEAR AI is not connected on this machine." };
  }
  const endpoint = inferenceUrl(url);
  if (!endpoint.ok) return { ...endpoint, model: false };
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), 20000);
  try {
    const response = await fetchImpl(endpoint.url, {
      method: "POST",
      headers: {
        authorization: `Bearer ${apiKey}`,
        "content-type": "application/json",
      },
      body: JSON.stringify({
        model,
        messages: [
          {
            role: "system",
            content:
              "You are the NEAR agent for nozy-agent.near. Propose a stake, a pay, or a hire for api, compute, or data. A hire is an unsigned NEAR Intents quote. You cannot sign. Shielded ZEC is not a NEAR action.",
          },
          { role: "user", content: String(text ?? "") },
        ],
        tools: TOOLS,
      }),
      signal: controller.signal,
    });
    const json = await response.json();
    if (!response.ok) {
      const message = json?.error?.message || "NEAR AI refused the request.";
      const reason = /spending limit|credits/i.test(message)
        ? "NEAR AI has no spending limit on this key. Stake, quote, and refuse still run on this computer without it."
        : String(message);
      return { ok: false, model: true, reason };
    }
    const toolCall = json?.choices?.[0]?.message?.tool_calls?.[0];
    if (!toolCall) {
      return { ok: false, model: true, reason: "The model did not propose a NEAR action." };
    }
    return { ok: true, model: true, action: toolToAction(toolCall) };
  } catch {
    return { ok: false, model: true, reason: "NEAR AI did not answer." };
  } finally {
    clearTimeout(timer);
  }
}

/** Ask the NEAR agent. A connected model may propose. The policy still decides. */
export async function ask(input, options = {}) {
  const read = input.text ? interpret(input.text) : input;
  if (read.kind === "swap" || read.kind === "zec-intent" || read.kind === "hire" || read.kind === "negotiate") {
    const decision = propose({ ...input, ...read, text: undefined });
    if (!decision.ok || !decision.swap) return decision;
    if (input.live) {
      const account = await viewAccount(MAINNET_ACCOUNT, options);
      if (!account.exists) {
        return {
          ok: false,
          agent: "near",
          account: MAINNET_ACCOUNT,
          reason: "Mainnet account nozy-agent.near does not exist yet. Create and fund it before a swap can be deposited.",
        };
      }
    }
    const deadline = decision.swap.deadline ?? new Date(Date.now() + 30 * 60 * 1000).toISOString();
    const session = String(options.token ?? process.env.NEAR_INTENTS_JWT ?? "").trim();
    if (decision.swap.confidentiality && decision.swap.confidentiality !== "public" && !session) {
      return {
        ok: false,
        unsigned: true,
        agent: "near",
        account: MAINNET_ACCOUNT,
        from: decision.from,
        to: decision.to,
        confidentiality: decision.swap.confidentiality,
        reason: "1Click requires a user session for a confidential quote. Set NEAR_INTENTS_JWT on this computer. No deposit was created, and shielded ZEC stays in Nozy.",
      };
    }
    const quoted = await requestQuote({ ...decision.swap, deadline }, { ...options, token: session });
    return {
      ...quoted,
      agent: "near",
      account: MAINNET_ACCOUNT,
      from: decision.from,
      to: decision.to,
      payee: decision.payee,
      purpose: decision.purpose,
      role: decision.role,
      signer: decision.signer ?? null,
      confidentiality: decision.swap?.confidentiality ?? "public",
    };
  }

  const apiKey = options.apiKey ?? process.env.NEAR_AI_API_KEY;
  const understood = read.kind && read.kind !== "unknown";
  if (input.text && apiKey && !understood) {
    const model = await requestModelProposal(input.text, { ...options, apiKey });
    if (!model.ok) return { ...model, agent: "near", account: NEAR_ACCOUNT };
    const decision = propose({ ...model.action, confirmed: input.confirmed });
    return { ...decision, model: true };
  }
  const decision = propose(input.text ? { ...read, confirmed: input.confirmed, live: input.live, useHuman: input.useHuman } : input);
  if (decision.tx?.signerId === MAINNET_ACCOUNT) {
    const account = await viewAccount(MAINNET_ACCOUNT, options);
    return { ...decision, mainnetAccount: account.exists };
  }
  return decision;
}
