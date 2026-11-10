/** Draft a shielded invoice. The agent does not create an address and does not pay. */

const args = process.argv.slice(2);

function flag(name, fallback) {
  const index = args.indexOf(name);
  if (index === -1) return fallback;
  const value = args[index + 1];
  if (value == null || value.startsWith("--")) return fallback;
  return value;
}

const amount = flag("--amount", "0.01");
const memo = flag("--memo", "Human pays from the shielded wallet");

if (!/^\d+(\.\d+)?$/.test(amount) || Number(amount) <= 0) {
  console.error("amount must be a positive ZEC decimal");
  process.exitCode = 1;
} else {
  console.log(
    JSON.stringify(
      {
        action: "draft_invoice",
        amount_zec: amount,
        memo,
        status: "draft",
        payer: "human Nozy wallet",
        agent_may_sign: false,
        payment_address: null,
        settlement: "The human wallet creates a fresh shielded address and pays this invoice.",
      },
      null,
      2,
    ),
  );
}

setTimeout(() => process.exit(process.exitCode ?? 0), 50);
