/**
 * Copy the hand-authored white paper (+ logo) into landing/public for static hosting.
 */
import { copyFileSync, existsSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = join(here, "..", "..");
const srcMd = join(repoRoot, "docs", "reference", "NozyWallet_Whitepaper.md");
const srcLogo = join(
  repoRoot,
  "docs",
  "reference",
  "NozyWallet_Whitepaper_logo.png",
);
const destDir = join(here, "..", "public", "whitepaper");
const destMd = join(destDir, "whitepaper.md");
const destLogo = join(destDir, "logo.png");

mkdirSync(destDir, { recursive: true });
copyFileSync(srcMd, destMd);
console.log(`Copied white paper → ${destMd}`);

if (existsSync(srcLogo)) {
  copyFileSync(srcLogo, destLogo);
  console.log(`Copied logo → ${destLogo}`);
} else {
  console.warn(`Logo missing: ${srcLogo}`);
}