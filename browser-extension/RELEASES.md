# Browser extension releases (Chrome & Edge)

## For maintainers — publish a GitHub Release

1. **Version** — Set `version` in `manifest.json` and add a section to `CHANGELOG.md` for this release.
2. **Automatic attach** — When you **publish any GitHub Release** on this repo, workflow **`extension-release-bundles`** builds WASM + popup and **attaches**:
   - `nozy-extension-chromium-<manifestVersion>.zip`
   - `nozy-extension-firefox-<manifestVersion>.zip` (copy of Chromium zip; validate in Firefox before AMO).
3. **Manual extension-only release** — **Actions** → **extension-release-bundles** → **Run workflow** with input matching `manifest.json` (e.g. `0.1.4`). Creates/updates tag **`extension-v0.1.4`** and a dedicated release.
4. **CRX** — CI ships **zip** only. Signed **`.crx`** / store uploads use Chrome Web Store or Edge Add-ons (see **`STORE_SUBMISSION_CHECKLIST.md`**).
5. **Share** — Point users to **`browser-extension/README.md`** and **`COMPANION.md`**.

## For users — install from a release zip

- **Chrome**: `chrome://extensions` → enable **Developer mode** → **Load unpacked** (folder extracted from the chromium zip).
- **Edge**: `edge://extensions` → **Developer mode** → **Load unpacked**.

Shielded sync with Zebrad/lightwalletd still needs the **desktop companion** (`nozywallet-api`); see **`COMPANION.md`**.

## Store listings (optional)

When ready for Chrome Web Store / Edge Add-ons, follow **`STORE_SUBMISSION_CHECKLIST.md`**.
