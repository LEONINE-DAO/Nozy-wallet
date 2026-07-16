# Store listing copy — NozyWallet Mobile

Draft text for Google Play and Apple App Store. Edit before submit.

---

## App name

NozyWallet

## Short description (Play, 80 chars)

Shielded Zcash companion wallet — Orchard privacy via your API or Nozy hosted.

## Full description

NozyWallet Mobile is a privacy-first companion for Orchard shielded ZEC.

Connect to a NozyWallet API server — on your home PC or via Nozy hosted — to create a wallet, sync with the Zcash network, receive shielded ZEC, and send fully private transactions.

**Features**

- Orchard shielded balance and unified addresses
- Send and receive with ZIP-317 fee estimates
- Transaction history with explorer links
- Wallet profiles and account security settings
- Ironwood (NU6.3) readiness tools (when your API supports it)

**How it works**

This app is a **companion client**. Wallet scan, proving, and sync run on the API server you configure, which connects to a Zebra node. Your phone does not download the full blockchain.

For maximum control, run your own API at home. For convenience, use Nozy hosted with an API key from your operator.

**Privacy**

- On-chain: Orchard shielded pool hides amounts and addresses
- Off-chain: if you use a hosted API, the operator can see when you sync and send. Read the in-app risk disclosure before using hosted mode.

## Category

Finance

## Content rating notes

- Cryptocurrency wallet
- No gambling, no user-generated public content
- Users manage their own funds; no in-app purchases in v1

## Support URL

https://github.com/LEONINE-DAO/Nozy-wallet/issues

## Privacy policy URL

https://leonine-dao.github.io/Nozy-wallet/privacy/

(Hosted on the NozyWallet GitHub Pages landing site. Technical architecture: [privacy model](https://leonine-dao.github.io/Nozy-wallet/book/nozy/privacy-model.html).)

## Keywords (App Store)

zcash, zec, orchard, shielded, privacy, wallet, cryptocurrency

---

## Data safety (Google Play) — draft answers

| Data type | Collected? | Shared? | Notes |
|-----------|------------|---------|-------|
| Wallet password | Stored on device only | No | AsyncStorage session |
| API key | Stored on device only | Sent to user-configured API | HTTPS to hosted/self server |
| Seed phrase | Shown at create/restore via API | Depends on API deployment | Disclosed in About |
| Transaction data | Via API from Zebra | No third-party analytics in v1 | |

**Encryption in transit:** Yes (HTTPS for production / hosted)  
**Users can request deletion:** Wallet data on device cleared by uninstall; server data depends on operator

---

## Apple privacy nutrition labels — draft

- **Data linked to you:** None collected by app publisher directly in v1 companion model
- **Data not linked to you:** None (no analytics SDK in v1)
- **Encryption:** Uses standard HTTPS; app uses encryption — exempt (`ITSAppUsesNonExemptEncryption: false`)

---

## Screenshot checklist

1. Welcome — hosted connection + API key
2. Dashboard — balance, sync bar, receive/send
3. Send — amount, fee, review
4. Receive — QR + unified address
5. Settings — connection, account, about/privacy
