import { execSync } from "child_process";
import fs from "fs";
import path from "path";

const DIST_DIR = path.resolve("dist");
const DOCS_DIR = path.resolve("docs");

console.log("Building project...");
execSync("npm run build", { stdio: "inherit" });

console.log("Removing old docs folder...");
if (fs.existsSync(DOCS_DIR)) {
  fs.rmSync(DOCS_DIR, { recursive: true, force: true });
}

console.log("Moving dist to docs...");
fs.renameSync(DIST_DIR, DOCS_DIR);

console.log(
  "Move completed successfully. Now, you can push the changes to GitHub."
);
