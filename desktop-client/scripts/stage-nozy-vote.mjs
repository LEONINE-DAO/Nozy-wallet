import { spawnSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const desktopRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const repoRoot = path.resolve(desktopRoot, "..");
const ext = process.platform === "win32" ? ".exe" : "";

function hostTriple() {
  const out = spawnSync("rustc", ["-vV"], { encoding: "utf8" });
  if (out.status !== 0) {
    throw new Error(out.stderr || "rustc -vV failed");
  }
  const host = out.stdout.split(/\r?\n/).find((line) => line.startsWith("host:"));
  if (!host) throw new Error("rustc -vV did not report host");
  return host.slice("host:".length).trim();
}

const triple = hostTriple();
const destDir = path.join(desktopRoot, "src-tauri", "binaries");
const dest = path.join(destDir, `nozy-vote-${triple}${ext}`);
const built = path.join(repoRoot, "tools", "nozy-vote", "target", "release", `nozy-vote${ext}`);

if (existsSync(dest)) {
  console.log(`nozy-vote sidecar already staged: ${dest}`);
  process.exit(0);
}

if (!spawnSync("cargo", ["--version"], { encoding: "utf8" }).stdout) {
  throw new Error("cargo is required to stage nozy-vote");
}

const build = spawnSync(
  "cargo",
  ["build", "--release", "--manifest-path", path.join("tools", "nozy-vote", "Cargo.toml")],
  { cwd: repoRoot, stdio: "inherit" }
);
if (build.status !== 0) {
  process.exit(build.status ?? 1);
}

mkdirSync(destDir, { recursive: true });
copyFileSync(built, dest);
console.log(`staged ${dest}`);
