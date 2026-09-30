# Extension store submission checklist

**Chrome Web Store (live):** https://chromewebstore.google.com/detail/nozywallet/pnlmnkallkmelflckjkmohemibfahoce  
**Listed version:** **0.1.7** on the store today. Next upload is **Sweet Chili 0.1.26** from `nozy-extension-chromium-0.1.26.zip` once the release asset is attached.

**Version:** align with `browser-extension/manifest.json` (currently **0.1.26**).  
**GitHub Release zip (done for 0.1.8):**  
https://github.com/LEONINE-DAO/Nozy-wallet/releases/tag/extension-v0.1.8

## Pre-submit

- [x] Version bump in `manifest.json`, popup `package.json`, `CHANGELOG.md`, `RELEASES.md`
- [x] CI release workflow produces chromium + firefox zips (`extension-release.yml`)
- [x] GitHub Release assets attached (`nozy-extension-chromium-*.zip`, firefox twin)
- [ ] Extension icons in `manifest.json` (`icons/` 16/32/48/128) — **still open**
- [ ] Store screenshots under `store-assets/chrome/` (and edge/firefox as needed)
- [ ] Privacy policy URL live (landing Privacy page)
- [ ] Listing copy final pass — [`store-assets/STORE_LISTING.md`](store-assets/STORE_LISTING.md)

## Chrome Web Store

- [x] Developer account + payment
- [x] Upload chromium zip from Release
- [x] Single purpose description; remote code policy OK (WASM bundled)
- [x] Host permissions justified (companion localhost + HTTPS for sync/ZNS)
- [x] Submit for review — **live** (listing above)

## Edge Add-ons

- [ ] Reuse chromium package where possible
- [ ] Submit

## Firefox AMO (optional for grant window)

- [ ] Upload firefox zip; review MV3 notes
- [ ] Submit

## Honesty for reviewers

Nozy extension is Orchard-first with optional **local companion API**. It is not a full Zebrad node. Nym is opt-in via Settings (companion privacy APIs + optional `:9068` LWD mixnet proxy) — do not claim default mixnet routing in the store listing.
