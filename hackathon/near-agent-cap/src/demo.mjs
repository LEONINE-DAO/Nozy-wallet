import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));

function beat(title, script, args) {
  console.log(`\n=== ${title} ===`);
  const result = spawnSync(process.execPath, [script, ...args], {
    cwd: path.join(here, ".."),
    encoding: "utf8",
  });
  if (result.stdout) process.stdout.write(result.stdout);
  if (result.stderr) process.stderr.write(result.stderr);
  console.log(`exit ${result.status}`);
  return result.status ?? 1;
}

console.log("Beat 1: no human wallet is loaded.");
console.log("Signer account: nozy-agent.testnet");
console.log("These beats print unsigned transactions. They do not sign or broadcast.");

const under = beat("Beat 2: 1 NEAR is under the cap of 5", "src/propose.mjs", ["--amount", "1"]);
const over = beat("Beat 3: 10 NEAR is over the cap", "src/propose.mjs", ["--amount", "10"]);
const confirmed = beat("Beat 4: the same 10 NEAR with confirmation", "src/propose.mjs", [
  "--amount",
  "10",
  "--confirmed",
]);
const stranger = beat("Beat 5: pay a stranger", "src/pay.mjs", [
  "--to",
  "stranger.near",
  "--amount",
  "1",
]);
const invoice = beat("Beat 6: draft a shielded invoice", "src/invoice.mjs", [
  "--amount",
  "0.01",
  "--memo",
  "Human pays from the shielded wallet",
]);

if (under !== 0 || over !== 1 || confirmed !== 0 || stranger !== 1 || invoice !== 0) {
  process.exitCode = 1;
}
