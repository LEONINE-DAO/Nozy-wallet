import { RNEAR_CONTRACTS, decimalToYocto, evaluateSwap, evaluateTransaction } from "./policy.mjs";

export const AGENT_ACCOUNTS = {
  testnet: "nozy-agent.testnet",
  mainnet: "nozy-agent.near",
};
const HUMAN = "human.near";
const CAP = "5";
const GAS = "200000000000000";

const RHEA_CALLS = {
  stake: { methodName: "deposit_and_stake", attach: true },
  stake_all: { methodName: "stake_all" },
  deposit: { methodName: "deposit", attach: true },
  stake_only: { methodName: "stake", argsAmount: true },
  unstake: { methodName: "unstake", argsAmount: true },
  unstake_all: { methodName: "unstake_all" },
  withdraw: { methodName: "withdraw", argsAmount: true },
  withdraw_all: { methodName: "withdraw_all" },
  storage: { methodName: "storage_deposit", attach: true, defaultAmount: "0.00125" },
};

function rheaTransaction(network, signerId, call, amountNear) {
  const yocto = call.argsAmount || call.attach ? decimalToYocto(amountNear) : { ok: true, yocto: 0n };
  if (!yocto.ok) return { error: yocto.reason };
  const args = call.argsAmount ? { amount: yocto.yocto.toString() } : {};
  return {
    tx: {
      signerId,
      receiverId: RNEAR_CONTRACTS[network],
      actions: [
        {
          type: "FunctionCall",
          params: {
            methodName: call.methodName,
            args,
            gas: GAS,
            deposit: call.attach ? yocto.yocto.toString() : "0",
          },
        },
      ],
    },
  };
}

function transferTransaction(signerId, receiverId, amountNear) {
  const yocto = decimalToYocto(amountNear);
  if (!yocto.ok) return { error: yocto.reason };
  return {
    tx: {
      signerId,
      receiverId,
      actions: [
        {
          type: "Transfer",
          params: { deposit: yocto.yocto.toString() },
        },
      ],
    },
  };
}

function decide(tx, confirmed, network) {
  const decision = evaluateTransaction(tx, {
    network,
    humanWalletIds: [HUMAN],
    capNear: CAP,
    confirmed,
  });
  if (!decision.ok) {
    return {
      ok: false,
      reason: decision.reason,
      needsConfirm:
        decision.reason === "amount is over the cap" ||
        decision.reason === "full exit needs confirmation",
    };
  }
  return {
    ok: true,
    unsigned: true,
    tx,
  };
}

/**
 * One agent action for the GUI. Nothing is signed or broadcast.
 * @param {{ kind?: string, amount?: string, to?: string, memo?: string, confirmed?: boolean, useHuman?: boolean }} input
 */
export function act(input) {
  const kind = input.kind;
  const network = input.network === "mainnet" ? "mainnet" : "testnet";
  const useHuman = Boolean(input.useHuman);
  const signerId = useHuman ? HUMAN : AGENT_ACCOUNTS[network];
  const confirmed = Boolean(input.confirmed);
  const amount = String(input.amount ?? "").trim();

  if (kind === "disable-nym") {
    return {
      ok: false,
      reason: "agent cannot disable Nym",
    };
  }

  if (kind === "invoice") {
    const zec = amount || "0.01";
    if (!/^\d+(\.\d+)?$/.test(zec) || Number(zec) <= 0) {
      return { ok: false, reason: "amount must be a positive ZEC decimal" };
    }
    return {
      ok: true,
      invoice: {
        action: "draft_invoice",
        amount_zec: zec,
        memo: String(input.memo ?? "Human pays from the shielded wallet"),
        status: "draft",
        payer: "human Nozy wallet",
        agent_may_sign: false,
        payment_address: null,
        settlement: "The human wallet creates a fresh shielded address and pays this invoice.",
      },
    };
  }

  if (kind === "swap") {
    const decision = evaluateSwap({
      from: input.from,
      to: input.to,
      amount: amount || "1",
      cap: CAP,
      confirmed,
      live: Boolean(input.live),
      accountId: AGENT_ACCOUNTS.mainnet,
      deadline: input.deadline,
      confidentiality: input.confidentiality,
    });
    if (!decision.ok) {
      return { ok: false, reason: decision.reason, needsConfirm: Boolean(decision.needsConfirm) };
    }
    return {
      ok: true,
      unsigned: true,
      account: AGENT_ACCOUNTS.mainnet,
      from: decision.from,
      to: decision.to,
      swap: decision.request,
    };
  }

  if (kind === "pay") {
    const built = transferTransaction(signerId, String(input.to ?? "stranger.near").trim(), amount || "1");
    if (built.error) return { ok: false, reason: built.error };
    return decide(built.tx, confirmed, network);
  }

  const rhea = RHEA_CALLS[kind];
  if (rhea) {
    const built = rheaTransaction(network, signerId, rhea, amount || rhea.defaultAmount || "1");
    if (built.error) return { ok: false, reason: built.error };
    return decide(built.tx, confirmed, network);
  }

  return { ok: false, reason: "unknown action" };
}
