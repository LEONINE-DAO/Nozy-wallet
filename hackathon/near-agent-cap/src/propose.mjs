import { RNearClient } from "@rhea-finance/rnear-sdk";
import { evaluateTransaction } from "./policy.mjs";

const args = process.argv.slice(2);

function flag(name, fallback) {
  const index = args.indexOf(name);
  if (index === -1) return fallback;
  const value = args[index + 1];
  if (value == null || value.startsWith("--")) return fallback;
  return value;
}

const amount = flag("--amount", "1");
const cap = flag("--cap", "5");
const signerId = flag("--signer", process.env.NEAR_ACCOUNT_ID ?? "nozy-agent.testnet");
const confirmed = args.includes("--confirmed");
const humanWalletIds = (flag("--human", process.env.NEAR_HUMAN_WALLET_IDS ?? ""))
  .split(",")
  .map((id) => id.trim())
  .filter(Boolean);

const client = new RNearClient({ network: "testnet" });
const tx = await client.buildStakeTransaction({
  accountId: signerId,
  nearAmount: amount,
});
tx.signerId = signerId;

const decision = evaluateTransaction(tx, {
  network: "testnet",
  humanWalletIds,
  capNear: cap,
  confirmed,
});

if (!decision.ok) {
  console.error(decision.reason);
  process.exitCode = 1;
} else {
  console.log(JSON.stringify(tx, null, 2));
}

setTimeout(() => process.exit(process.exitCode ?? 0), 50);
