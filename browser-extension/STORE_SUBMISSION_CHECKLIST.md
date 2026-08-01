# Extension store submission checklist

**Version:** align with `browser-extension/manifest.json` (currently **0.1.8**).  
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

- [ ] Developer account + payment
- [ ] Upload chromium zip from Release
- [ ] Single purpose description; remote code policy OK (WASM bundled)
- [ ] Host permissions justified (companion localhost + HTTPS for sync/ZNS)
- [ ] Submit for review

## Edge Add-ons

- [ ] Reuse chromium package where possible
- [ ] Submit

## Firefox AMO (optional for grant window)

- [ ] Upload firefox zip; review MV3 notes
- [ ] Submit

## Honesty for reviewers

Nozy extension is Orchard-first with optional **local companion API**. It is not a full Zebrad node. Nym is opt-in via desktop/API surfaces when enabled — do not claim default mixnet routing in the store listing.
