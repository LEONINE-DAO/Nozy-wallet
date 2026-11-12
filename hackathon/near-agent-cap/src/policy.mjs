/** Local cap for an agent rNEAR stake. No network. No keys. */

export const RNEAR_CONTRACTS = {
  mainnet: "lst.rhealab.near",
  testnet: "lst.ref-dev.testnet",
};

export const ALLOWED_METHODS = [
  "storage_deposit",
  "deposit",
  "deposit_and_stake",
  "stake",
  "stake_all",
  "unstake",
  "unstake_all",
  "withdraw",
  "withdraw_all",
];

/** Assets this agent may quote on NEAR Intents. Shielded ZEC is not one of them. */
export const SWAP_ASSETS = {
  NEAR: {
    symbol: "NEAR",
    assetId: "nep141:wrap.near",
    decimals: 24,
  },
  USDC: {
    symbol: "USDC",
    assetId: "nep141:17208628f84f5d6ad33f0da3bbbeb27ffcb398eac501a31bd6ad2011e36133a1",
    decimals: 6,
  },
  USDT: {
    symbol: "USDT",
    assetId: "nep141:usdt.tether-token.near",
    decimals: 6,
  },
  RHEA: {
    symbol: "RHEA",
    assetId: "nep141:token.rhealab.near",
    decimals: 18,
  },
};

const YOCTO = 10n ** 24n;
const UNBOUNDED = new Set(["stake_all", "unstake_all", "withdraw_all"]);
const ZEC_SWAP = /zec|shielded|omft\.near/i;

export function decimalToYocto(amount) {
  const scaled = amountToBase(amount, 24);
  if (!scaled.ok) {
    const reason = scaled.reason.includes("decimal places")
      ? "amount has more than 24 decimal places"
      : "amount must be a decimal NEAR string";
    return { ok: false, reason };
  }
  return { ok: true, yocto: scaled.base };
}

export function amountToBase(amount, decimals) {
  const text = String(amount).trim();
  const places = Number(decimals);
  if (!/^\d+(\.\d+)?$/.test(text)) {
    return { ok: false, reason: "amount must be a decimal string" };
  }
  const [whole, frac = ""] = text.split(".");
  if (frac.length > places) {
    return { ok: false, reason: `amount has more than ${places} decimal places` };
  }
  const padded = frac.padEnd(places, "0");
  const scale = 10n ** BigInt(places);
  return { ok: true, base: BigInt(whole) * scale + BigInt(padded || "0") };
}

export function resolveSwapAsset(name) {
  const key = String(name ?? "").trim().toUpperCase();
  if (key === "WNEAR") return SWAP_ASSETS.NEAR;
  return SWAP_ASSETS[key] ?? null;
}

function fail(reason) {
  return { ok: false, reason };
}

/**
 * @param {object} input
 * @param {"mainnet"|"testnet"} [input.network]
 * @param {string} input.signerId
 * @param {string[]} [input.humanWalletIds] account ids that must never sign
 * @param {string} input.receiverId
 * @param {string} input.methodName
 * @param {string} [input.amountNear] human decimal; required except for *_all
 * @param {string} [input.capNear] default "5"
 * @param {boolean} [input.confirmed] user pressed the confirmation card
 */
export function evaluateAction(input) {
  const network = input.network ?? "testnet";
  const contract = RNEAR_CONTRACTS[network];
  if (!contract) return fail("unknown network");

  const signerId = String(input.signerId ?? "").trim();
  if (!signerId) return fail("signer is required");

  const humanWalletIds = input.humanWalletIds ?? [];
  if (humanWalletIds.includes(signerId)) {
    return fail("human wallet cannot sign agent stake");
  }

  if (input.receiverId !== contract) {
    return fail(`receiver must be ${contract}`);
  }

  if (!ALLOWED_METHODS.includes(input.methodName)) {
    return fail("method is not allowed");
  }

  const cap = decimalToYocto(input.capNear ?? "5");
  if (!cap.ok) return fail(cap.reason);

  if (UNBOUNDED.has(input.methodName)) {
    if (!input.confirmed) {
      return fail("full exit needs confirmation");
    }
    return { ok: true };
  }

  if (input.amountNear == null || input.amountNear === "") {
    return fail("amount is required");
  }
  const amount = decimalToYocto(input.amountNear);
  if (!amount.ok) return fail(amount.reason);

  if (amount.yocto > cap.yocto && !input.confirmed) {
    return fail("amount is over the cap");
  }

  return { ok: true };
}

/**
 * A NEAR Intents quote for an allowlisted NEAR asset.
 * `live` asks 1Click for a deposit address and always needs the confirm card.
 */
export function evaluateSwap(input) {
  const described = `${input.from ?? ""} ${input.to ?? ""} ${input.assetIn ?? ""} ${input.assetOut ?? ""}`;
  if (ZEC_SWAP.test(described)) {
    return fail("Shielded ZEC does not go through NEAR Intents.");
  }
  const from = resolveSwapAsset(input.from ?? "NEAR");
  const to = resolveSwapAsset(input.to ?? "USDC");
  if (!from || !to) {
    return fail("This agent swaps NEAR, USDC, USDT, and RHEA on NEAR.");
  }
  if (from.assetId === to.assetId) return fail("swap needs two different assets");
  if (ZEC_SWAP.test(from.assetId) || ZEC_SWAP.test(to.assetId)) {
    return fail("Shielded ZEC does not go through NEAR Intents.");
  }

  const amount = amountToBase(input.amount ?? "1", from.decimals);
  if (!amount.ok) return fail(amount.reason);
  const cap = amountToBase(input.cap ?? "5", from.decimals);
  if (!cap.ok) return fail(cap.reason);
  if (amount.base > cap.base && !input.confirmed) {
    return { ok: false, reason: "amount is over the cap", needsConfirm: true };
  }
  if (input.live && !input.confirmed) {
    return { ok: false, reason: "a deposit address needs confirmation", needsConfirm: true };
  }

  const confidentiality = String(input.confidentiality ?? "public").trim().toLowerCase();
  if (!["public", "basic", "advanced"].includes(confidentiality)) {
    return fail("confidentiality must be public, basic, or advanced");
  }

  const account = String(input.accountId ?? "").trim();
  if (!account) return fail("mainnet account is required");
  const recipient = resolveRecipient(account, input.recipient);
  if (!recipient.ok) return recipient;

  const request = {
    dry: !input.live,
    swapType: "EXACT_INPUT",
    slippageTolerance: 100,
    originAsset: from.assetId,
    depositType: "ORIGIN_CHAIN",
    destinationAsset: to.assetId,
    amount: amount.base.toString(),
    recipient: recipient.account,
    recipientType: "INTENTS",
    refundTo: account,
    refundType: "ORIGIN_CHAIN",
    deadline: input.deadline,
  };
  if (confidentiality !== "public") request.confidentiality = confidentiality;

  return {
    ok: true,
    from: from.symbol,
    to: to.symbol,
    request,
  };
}

/**
 * Every action in a built NEAR transaction must pass.
 * deposit is yoctoNEAR. FunctionCall args may include amount for unstake.
 */
export function evaluateTransaction(tx, options = {}) {
  const actions = tx.actions ?? [];
  if (actions.length === 0) return fail("transaction has no actions");

  const signerId = String(tx.signerId ?? "").trim();
  const humanWalletIds = options.humanWalletIds ?? [];
  if (humanWalletIds.includes(signerId)) {
    return fail("human wallet cannot sign agent stake");
  }

  for (const action of actions) {
    if (action.type === "Transfer") {
      return fail("agent cannot pay a stranger");
    }
    if (action.type !== "FunctionCall") {
      return fail("only function-call actions are allowed");
    }
    const params = action.params ?? {};
    const methodName = params.methodName;
    let amountNear = options.amountNear;
    if (params.deposit != null && methodName !== "unstake") {
      const yocto = BigInt(params.deposit);
      amountNear = formatYocto(yocto);
    }
    if (methodName === "unstake" || methodName === "withdraw" || methodName === "stake") {
      const args = parseArgs(params.args);
      if (args.amount != null) amountNear = formatYocto(BigInt(args.amount));
    }
    const result = evaluateAction({
      network: options.network ?? "testnet",
      signerId,
      humanWalletIds,
      receiverId: tx.receiverId,
      methodName,
      amountNear,
      capNear: options.capNear ?? "5",
      confirmed: options.confirmed ?? false,
    });
    if (!result.ok) return result;
  }
  return { ok: true };
}

const HIRE_PURPOSES = new Set(["api", "compute", "data"]);
const NEAR_ACCOUNT = /^[a-z0-9][a-z0-9\-_]{0,62}[a-z0-9]\.near$/;
const REFUSED_PAYEES = new Set([
  "stranger.near",
  "human.near",
  "nozy-agent.near",
  "nozy-agent.testnet",
]);

function resolveRecipient(payer, recipient) {
  if (recipient == null || recipient === "") return { ok: true, account: payer };
  const account = String(recipient).trim().toLowerCase();
  if (!NEAR_ACCOUNT.test(account) || REFUSED_PAYEES.has(account) || account === payer) {
    return fail("agent cannot pay a stranger");
  }
  return { ok: true, account };
}

/**
 * NEAR agent pays another agent for API, compute, or data.
 * The quote is unsigned. Refund stays with the payer. Shielded ZEC is refused.
 */
export function evaluateHire(input) {
  const purpose = String(input.purpose ?? "").trim().toLowerCase();
  if (!HIRE_PURPOSES.has(purpose)) {
    return fail("hire purpose must be api, compute, or data");
  }
  const payee = String(input.payee ?? "").trim().toLowerCase();
  if (!NEAR_ACCOUNT.test(payee) || REFUSED_PAYEES.has(payee)) {
    return fail("agent cannot pay a stranger");
  }
  const described = `${input.asset ?? ""} ${input.from ?? ""} ${input.to ?? ""} ${payee}`;
  if (ZEC_SWAP.test(described) || input.agent === "zcash") {
    return fail("Shielded ZEC does not go through NEAR Intents.");
  }
  const swap = evaluateSwap({
    from: "NEAR",
    to: input.to && input.to !== "NEAR" ? input.to : "USDC",
    amount: input.amount ?? "1",
    cap: input.cap ?? "5",
    confirmed: input.confirmed,
    live: input.live,
    accountId: input.accountId ?? "nozy-agent.near",
    recipient: input.payee,
    deadline: input.deadline,
    confidentiality: "public",
  });
  if (!swap.ok) return swap;
  return {
    ok: true,
    role: input.role ?? "hire",
    purpose,
    payee: swap.request.recipient,
    from: swap.from,
    to: swap.to,
    request: swap.request,
  };
}

/** Accept the lower NEAR price. A higher counter waits for the confirm card. */
export function evaluateNegotiation(input) {
  const offer = amountToBase(input.offer, 24);
  if (!offer.ok) return fail(offer.reason);
  const counter = amountToBase(input.counter, 24);
  if (!counter.ok) return fail(counter.reason);
  if (offer.base === 0n || counter.base === 0n) return fail("amount must be greater than zero");
  if (counter.base > offer.base && !input.confirmed) {
    return { ok: false, reason: "the counter is above the offer", needsConfirm: true };
  }
  const accepted = counter.base <= offer.base ? input.counter : input.offer;
  return evaluateHire({
    ...input,
    amount: accepted,
    role: "negotiate",
  });
}

function parseArgs(args) {
  if (args == null) return {};
  if (typeof args === "object") return args;
  try {
    return JSON.parse(args);
  } catch {
    return {};
  }
}

function formatYocto(yocto) {
  const whole = yocto / YOCTO;
  const frac = (yocto % YOCTO).toString().padStart(24, "0").replace(/0+$/, "");
  return frac.length === 0 ? whole.toString() : `${whole}.${frac}`;
}
