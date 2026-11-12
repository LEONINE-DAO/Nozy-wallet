/**
 * Metro 0.81 calls `require("image-size")(buffer)`.
 * image-size 2.0.3 (the release that fixes GHSA-w3rx-r6r6-pgpr and
 * GHSA-5p2g-fcmc-qvqq) only exposes a named CJS export, so that call throws.
 * Keep the patched package and restore the callable export Metro expects.
 */
const fs = require("fs");
const path = require("path");

const file = path.join(__dirname, "..", "node_modules", "image-size", "dist", "cjs", "index.js");
if (!fs.existsSync(file)) {
  process.exit(0);
}

const marker = "/* nozy: callable image-size export */";
const source = fs.readFileSync(file, "utf8");
if (source.includes(marker)) {
  process.exit(0);
}

fs.appendFileSync(
  file,
  `\n${marker}\n` +
    "const nozyImageSize = exports.imageSize;\n" +
    "module.exports = nozyImageSize;\n" +
    "module.exports.imageSize = nozyImageSize;\n" +
    "module.exports.default = nozyImageSize;\n" +
    "module.exports.types = exports.types;\n" +
    "module.exports.disableTypes = exports.disableTypes;\n"
);
