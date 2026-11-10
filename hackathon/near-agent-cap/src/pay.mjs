import { decimalToYocto, evaluateTransaction } from "./policy.mjs";

const args = process.argv.slice(2);

function flag(name, fallback) {
  const index = args.indexOf(name);
  if (index === -1) return fallback;
  const value = args[index + 1];
  if (value == null || value.startsWith("--")) return fallback;
  return value;
}

const to = flag("--to", "stranger.near");
const amount = flag("--amount", "1");
const signerId = flag("--signer", process.env.NEAR_ACCOUNT_ID ?? "nozy-agent.testnet");
const humanWalletIds = (flag("--human", process.env.NEAR_HUMAN_WALLET_IDS ?? ""))
  .split(",")
  .map((id) => id.trim())
  .filter(Boolean);

const yocto = decimalToYocto(amount);
if (!yocto.ok) {
  console.error(yocto.reason);
  process.exitCode = 1;
} else {
  const tx = {
    signerId,
    receiverId: to,
    actions: [
      {
        type: "Transfer",
        params: { deposit: yocto.yocto.toString() },
      },
    ],
  };
  const decision = evaluateTransaction(tx, {
    network: "testnet",
    humanWalletIds,
  });
  if (!decision.ok) {
    console.error(decision.reason);
    process.exitCode = 1;
  } else {
    console.log(JSON.stringify(tx, null, 2));
  }
}

setTimeout(() => process.exit(process.exitCode ?? 0), 50);
