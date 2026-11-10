# Nozy Agent Desk

A NEAR agent can stake and quote on Rhea. The human Zcash seed stays in Nozy and is never the NEAR signer.

NozyWallet is an existing shielded Zcash wallet (Orchard / Ironwood). That wallet is baseline from before this hackathon. The work in this directory is the NEAR slice: a second account, a function-call key, and a local spending cap. Judges review this directory.

Built for NEAR HACKATHON. Tracks this slice fits: `security`, `agents`, and `privacy`. Privacy here is the NEAR AI Cloud gateway quote. The desk verifies that quote with Intel DCAP, then checks the nonce and signer inside the verified quote, the RTMR3 event log, and the MRCONFIGID of `app_compose`. The accepted TCB status is `UpToDate`. A quote that fails any of those checks stays unverified. The desk does not check NVIDIA GPU evidence or container image signatures.

## Problem

An agent that can move money should not hold the shielded Zcash seed, and it should not be able to pay an arbitrary account. NEAR supplies the agent account and a key that can call one contract. The cap in this process decides whether the attached stake is allowed.

## What a judge can run

Node 18 or newer. From this directory:

```bash
npm install
npm test
npm run gui
```

`npm test` runs 41 offline checks. No network and no key.

`npm run gui` serves the desk at http://127.0.0.1:4173. The page reads public balances for `nozy-agent.near` from `https://rpc.mainnet.fastnear.com`. Stake, swap, and shield actions stay unsigned until a person confirms, and this window does not broadcast them.

| Do this | See this |
| --- | --- |
| `npm test` | 41 checks pass. |
| `npm run gui`, then open http://127.0.0.1:4173 | Two cards. Zcash transparent receive is `t1YVKP6WGcvdJYb1XHehkfLYWXvBCt3BcZG`. NEAR signer is `nozy-agent.near`, with NEAR, rNEAR, and RHEA balances. Stake contract is `lst.rhealab.near`. |
| On the desk, ask `Stake 1 NEAR` | An unsigned proposal under the 5 NEAR cap. The function-call key is the only signer this page will use. |
| On the desk, ask `Stake 10 NEAR` | The proposal waits for the confirmation card. |
| On the desk, ask `Pay a stranger` | Refusal. No transfer is built. |
| On the desk, ask `Swap 1 NEAR to RHEA` | An unsigned dry quote. Deposit type stays `ORIGIN_CHAIN`. No deposit address is created. |
| On the desk, ask to swap shielded ZEC | Refusal before any quote. Shielded ZEC stays in Nozy. See [Zcash forum 57497](https://forum.zcashcommunity.com/t/my-experience-exiting-shielded-zec-from-zodl-to-near-intents-589k-usdt-still-held-50-days-despite-a-written-compliance-clearance/57497). |
| On the desk, ask `Private RHEA quote` | Refusal unless `NEAR_INTENTS_JWT` is set on this computer. No session is stored here. |
| `GET http://127.0.0.1:4173/api/tee` with `NEAR_AI_API_KEY` in `.env` | Intel DCAP status `UpToDate`, debug mode off, the nonce and signer bound in the quote, the configuration measurement, and RTMR3. `verified` is true only when every one of those checks passes. |
| `GET /api/tee` with no key | `NEAR AI is not connected on this machine.` |

The same cap is available from the command line. These commands print unsigned testnet transactions. They do not sign or broadcast. The default signer is `nozy-agent.testnet` and the receiver is `lst.ref-dev.testnet`.

| Command | Result |
| --- | --- |
| `npm run propose -- --amount 1` | Unsigned `deposit_and_stake` for 1 NEAR. |
| `npm run propose -- --amount 10` | `amount is over the cap`. Exit code 1. |
| `npm run propose -- --amount 10 --confirmed` | Unsigned 10 NEAR transaction. The confirmation flag is the only change. |
| `npm run propose -- --amount 1 --signer human.near --human human.near` | `human wallet cannot sign agent stake`. Exit code 1. |
| `npm run pay -- --to stranger.near --amount 1` | `agent cannot pay a stranger`. Exit code 1. |
| `npm run invoice -- --amount 0.01 --memo "Human pays from the shielded wallet"` | Draft invoice. `agent_may_sign` is false and `payment_address` is null. |
| `npm run demo` | Those beats, in order. |

A drafted invoice on the desk uses the Nozy companion at http://127.0.0.1:3000 when that process is already running. The agent cannot pay the invoice. Start the companion before unlocking. The human profile password stays on this computer.

## Mainnet account

Account: [`nozy-agent.near`](https://nearblocks.io/address/nozy-agent.near).

| Key | Public key | What it can do |
| --- | --- | --- |
| Owner | `ed25519:66jynpxbv9Esd4NN8UknCf2221wAM8ZW8NNkPEweNJLP` | Full access. Recovery. It is not loaded into the desk page. |
| Agent | `ed25519:D1N42CBduTgUmfw9oYwH6CEDWunGZyMZ3b1Xj8a6cZxw` | Function calls on `lst.rhealab.near` only. Methods: `storage_deposit`, `deposit`, `deposit_and_stake`, `stake`, `stake_all`, `unstake`, `unstake_all`, `withdraw`, `withdraw_all`. Allowance 0.25 NEAR, which pays gas. |

The allowance does not limit the stake. The local cap does. Default cap is 5 NEAR. `stake_all`, `unstake_all`, and `withdraw_all` need the confirmation flag.

The agent key cannot transfer NEAR, call `intents.near`, or sign a Rhea swap batch. A NEAR to RHEA price is an unsigned 1Click quote (`https://1click.chaindefuser.com/v0/quote`). That quote host is clearnet. Signed traffic that leaves this computer is meant to go through Nym. The agent cannot turn Nym off.

RHEA token reads use `token.rhealab.near` (18 decimals). rNEAR reads use `lst.rhealab.near` (24 decimals). Market: https://app.rhea.finance/#near|token.rhealab.near

Secrets stay outside this repository: OS keychain, `NEAR_AI_API_KEY` and `NEAR_INTENTS_JWT` in a gitignored `.env`, and the agent Zcash key file pointed at by `NOZY_AGENT_ZCASH_KEYFILE`. The desk drops the mnemonic before it answers `/api/setup`.

Earlier testnet practice, still on chain and not the desk signer: `nozy-agent.testnet`, agent key `ed25519:D3tycEnNQeZSaiZusvfUj7Av2AS2hPErvKaJ8pcaw5up` on `lst.ref-dev.testnet`. Register [8BmgoCPokXRjbv23ujrUdQQpX1nSuU95ZJxL7tKe1Fsc](https://explorer.testnet.near.org/transactions/8BmgoCPokXRjbv23ujrUdQQpX1nSuU95ZJxL7tKe1Fsc). Stake 1 NEAR [DwugLm1puATJHqj53B74oTHxo9Et5epYDYt5sZxB9dmr](https://explorer.testnet.near.org/transactions/DwugLm1puATJHqj53B74oTHxo9Et5epYDYt5sZxB9dmr).

## Libraries and baseline

- `@rhea-finance/rnear-sdk` builds the unsigned stake transaction.
- NEAR RPC `https://rpc.mainnet.fastnear.com` reads account and token balances.
- 1Click `https://1click.chaindefuser.com/v0/quote` prices NEAR, USDC, USDT, and RHEA. A missing partner JWT uses the public fee. Confidential quotes need `NEAR_INTENTS_JWT`.
- NEAR AI Cloud `https://cloud-api.near.ai/v1/attestation/report` and `https://cloud-api.near.ai/v1/chat/completions`. The model host is allowlisted to `cloud-api.near.ai`. A model may propose a stake. The policy still decides. Model id used by the ask path: `zai-org/GLM-5.1-FP8`.
- `@phala/dcap-qvl` verifies the Intel TDX quote and reads the measurements from that verified quote. Collateral comes from the library's default PCCS.

The Zcash wallet, companion, and extension outside this directory are the pre-hackathon product. The extension Rhea page mirrors the same public balances. It does not add a second signer.

## AI tools

Cursor, using Grok, drafted the policy, the desk, the attestation checks, and this README. The human author reviews the account, the recording, and the submission.
