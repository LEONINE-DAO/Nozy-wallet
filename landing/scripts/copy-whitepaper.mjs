/**
 * Copy the hand-authored white paper into landing/public for static hosting.
 */
import { copyFileSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = join(here, "..", "..");
const src = join(repoRoot, "docs", "reference", "NozyWallet_Whitepaper.md");
const destDir = join(here, "..", "public", "whitepaper");
const dest = join(destDir, "whitepaper.md");

mkdirSync(destDir, { recursive: true });
copyFileSync(src, dest);
console.log(`Copied white paper → ${dest}`);
