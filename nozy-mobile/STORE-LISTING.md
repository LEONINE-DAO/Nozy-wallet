# Store listing copy — NozyWallet Mobile 1.0.0

Copy for Google Play. Match the binary: keys stay on the phone, compact sync goes to public lightwalletd, this version receives shielded ZEC. Spending from the phone is the next release. Do not describe this build as a companion that holds keys on a server.

iOS is not a submit candidate until `libnozy_ffi` is built on a Mac and linked in the Xcode project. The copy below is the Android listing.

---

## App name

NozyWallet

## Short description (Play, 80 chars)

Shielded Zcash wallet. Keys stay on your phone. Receive and sync ZEC.

(68 characters)

## Full description

NozyWallet is a shielded Zcash wallet. Your seed and keys stay on this phone. The app syncs compact blocks from lightwalletd at lwd.nozywallet.org and shows a unified address so you can receive Orchard ZEC.

**In this version**

- Create or restore a wallet on the phone
- Compact sync over HTTPS (default lwd.nozywallet.org)
- Shielded balance and a unified receive address
- Seed and password stored in the device keystore

**Not in this version**

Spending ZEC from the phone. Send opens an explanation and points you to Receive. An on-phone Orchard spend proof is the next release.

**Privacy**

- On-chain: Orchard shielded pool
- Off-chain: compact sync does not upload your seed. The lightwalletd operator can see that a client connected.
- No analytics SDK in this version

A self-hosted nozywallet-api is an advanced option in Settings. It is not the store default, and this listing does not point users at the operator wallet on api.nozywallet.org.

## Category

Finance

## Content rating notes

- Cryptocurrency wallet
- No gambling, no user-generated public content
- Users hold their own keys; no in-app purchases in 1.0.0

## Support URL

https://github.com/LEONINE-DAO/Nozy-wallet/issues

## Privacy policy URL

https://leonine-dao.github.io/Nozy-wallet/privacy/

## Keywords (App Store, when iOS ships)

zcash, zec, orchard, shielded, privacy, wallet, cryptocurrency

---

## Data safety (Google Play) — answers for 1.0.0

| Data type | Collected by the publisher? | Shared? | Notes |
|-----------|-----------------------------|---------|-------|
| Seed phrase | No. Stored on device only | No | Android Keystore via expo-secure-store. Shown once at create/restore. |
| Wallet password | No. Stored on device only | No | Same keystore. Not sent to lightwalletd. |
| Unified address | No publisher collection | The user can copy it | Compact sync does not upload the seed. |
| API key | Only if the user types a self-hosted companion key | Sent only to the URL they set | Store default does not use an API key. |
| Diagnostics | Not collected | No analytics SDK | |

**Encryption in transit:** Yes. Release builds allow HTTPS only.  
**Data deletion:** Uninstall clears app data. There is no publisher account to delete.

Suggested Play form: Data is encrypted in transit. Users can request that data be deleted (uninstall). No data is shared with third parties by the app publisher. Financial info is not collected by the developer; it stays on device.

---

## Apple privacy nutrition labels — draft (do not submit iOS yet)

- Data linked to you: none collected by the publisher
- Data not linked to you: none (no analytics SDK)
- Encryption: standard HTTPS; `ITSAppUsesNonExemptEncryption` is false

---

## Screenshot set

Captured from the NozyPixel emulator at 1080×2160 (Play’s 2:1 limit). The receive address and QR are covered so the store image does not publish a live unified address.

1. `store-assets/android/01-welcome.png` — keys stay on the phone
2. `store-assets/android/02-home.png` — balance and compact sync
3. `store-assets/android/03-receive.png` — receive layout, address covered
4. `store-assets/android/04-send.png` — spend is the next release
5. `store-assets/android/05-settings.png` — connection and account
6. `store-assets/android/06-about.png` — privacy and lwd.nozywallet.org
