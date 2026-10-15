# Building Nozy Agent: Private Infrastructure for the Next Generation of Zcash Tools

## A self-hosted roadmap for wallet assistance, private commerce, node operations, research, responsible automation and long-term financial education

Artificial intelligence is quickly becoming another interface to the internet. People are beginning to ask agents to research topics, organize work, monitor systems and prepare financial actions. But adding a chat box to a wallet is not enough—and giving a language model direct control over money is not a responsible shortcut.

NozyWallet is planning a different kind of foundation: **Nozy Agent**, a self-hosted, Rust-first agent system designed around Zcash, local control and explicit trust boundaries.

That agent does not start from zero. Nozy already ships a shielded-first stack users can run today:

- **CLI** production downloads (Orchard + Ironwood)
- **Desktop** production downloads (beta tag; no formal GA label)
- **Browser extension** live on the [Chrome Web Store](https://chromewebstore.google.com/detail/nozywallet/kjbpoimgafbikmachlilkhoamdlnhooj) as **NozyWallet 2.0.0 (Sweet Chili)** — keys stay on the device; if no local node is found, compact sync can use the public `lwd.nozywallet.org` path
- **Localhost companion API** for same-machine sync and shared wallet profiles

The goal is to keep the **private model** intact: shielded money, local inference, Nym-gated research, and honest warnings whenever a flow would push users onto transparent or compliance-gated rails. Seamless UX must never erase that boundary.

This article is a roadmap for the **agent layer**, not a claim that every specialist desk below already ships. Many components still require engineering, testing, security review and community feedback. Publishing the direction early matters because private financial infrastructure should be designed in public before users are asked to trust it.

## The real challenge is infrastructure, not conversation

A useful wallet agent must do more than produce convincing text. It needs to know which information is authoritative, which tools it may use, when it must ask for approval and what it must never be able to access.

That creates a harder set of questions:

- How can an agent help without holding spending keys?
- How can it search the web without exposing the user’s IP address and interests?
- How can it distinguish a transaction proposal from permission to spend?
- How can merchants automate orders without releasing goods before payment?
- How can node operators receive useful help without allowing arbitrary remote commands?
- How can local learning improve the experience without silently changing safety rules?
- How can a friendly virtual pet teach financial preparedness without manipulating users into depositing, trading or taking risks?

Nozy Agent is being planned around these questions from the beginning.

## A local-first architecture

The proposed agent will run as a separate `nozy-agent` product instead of being embedded directly inside the existing web dashboard. Its core is planned as a Rust service using Axum and a custom supervisor that routes work to narrowly scoped specialists.

The language model will run locally through Ollama by default. Operators with suitable hardware may use a localhost vLLM deployment through the same OpenAI-compatible interface. Cloud chat APIs are not part of the default trust model.

The separation of responsibilities is deliberate:

- **The local model interprets requests and proposes work.**
- **The Nozy Agent policy layer decides which tools are permitted.**
- **The wallet companion remains responsible for wallet operations.**
- **The user approves actions that can move funds.**
- **Spending keys never enter prompts, agent memory or model logs.**
- **When FROST is enabled later, spend shares stay outside the agent.** The default agent coordinates a signing session and holds no share.

The model is treated as an untrusted proposer, not as a signer or policy authority. A persuasive answer cannot bypass deterministic Rust checks.

## Nym is a boundary, not a marketing label

Local inference protects prompts from a cloud model provider, but an agent may still need external information. Market quotes, research sources and community updates can reveal a great deal about a user when fetched over a normal internet connection.

The planned Nozy Agent therefore makes Nym a requirement for non-local HTTP traffic. Local connections to the wallet companion, local model and local database remain on the machine. External research and market requests must pass through the configured Nym helper, with no clearnet fallback.

If that privacy route is unhealthy, the agent and its web surface should fail closed and explain what the operator needs to repair. The existing wallet should continue functioning independently.

This does not mean Nym makes every remote service trustworthy. A quote provider can still return bad data, a webpage can contain prompt injection and a trading venue can still have custody or settlement risks. Nym addresses network metadata exposure; validation, source provenance and policy controls remain separate requirements.

## Specialists instead of one all-powerful agent

Nozy Agent is planned as a supervised collection of specialist desks. Specialists do not freely call one another or acquire new tools. The supervisor controls handoffs through typed shared state, hop limits and role-specific permissions.

The roadmap currently includes:

- **Money and Guide:** explain balances, privacy choices and wallet workflows, then prepare proposals for explicit approval.
- **Memory:** keep local notes, corrections, preferences and reusable playbooks without storing mnemonics, passwords or raw keys.
- **Market and Alerts:** collect allowlisted information through Nym, cache it locally and distinguish sourced facts from model interpretation.
- **Node Operations:** guide self-hosted Zebrad and lightwalletd operators through health checks and documented maintenance.
- **Protocol Debug:** help CLI developers collect local diagnostics and reason about Zcash pool, witness, anchor and transaction failures without pretending NozyWallet is a consensus node.
- **Merchant Desk:** accept customer orders, create shielded ZEC invoices and release work only after the companion reports the required confirmation state.
- **Bounty Hunter:** maintain a private casebook, help reproduce issues, enforce structured reporting gates and support privacy-preserving public reputation based on verifiable acknowledgements.
- **Zcash Research and Knowledge:** answer questions about Zcash development, teams, history and community using a curated local corpus, the Zcash Community Forum and Nym-routed web research with citations.

This design gives every specialist less authority than a general-purpose agent. That limitation is a feature.

## The whole wallet, as a catalog

The agent covers the wallet by calling the localhost companion, the same API the desktop and the extension already use. It does not drive the screen, and it does not get a shell. Every user-facing companion route is one typed action. When the wallet grows a new screen, the catalog grows one action. A missing action is a gap to add. It is not permission for the model to improvise.

The user asks in ordinary language. The agent picks the action, or a short sequence that ends in one action. Rust policy sorts it into a class. The companion performs it.

| Class | What the agent may do | Examples already on the companion |
|-------|----------------------|-----------------------------------|
| **Look** | Run it after the user has unlocked. No second prompt. | Balance, history, notes, address book, sync and chain tip, fee estimate, Ironwood and Sapling status, Nym egress, proving-file status, invoices, local gift cards, ZNS resolve, vote status, Crosslink status |
| **Change** | Show a card with the exact fields, then wait for yes. One card per spend or disclosure. | Send, speed up, new shielded address, address-book edits, Ironwood split / migrate / broadcast, Sapling shield, invoice create or cancel, local gift-card create or redeem, ZNS link, profile, vote, Crosslink stake / unbond / withdraw, viewing-key export |
| **Hands off** | Tell the user which screen to use. The agent never receives the secret. | Create wallet, restore, unlock, type a seed or a password |
| **Refused** | The companion rejects it even if the user asks. | A transparent address, turning Nym off, skipping approval, putting a seed in memory, broadcasting raw bytes the companion did not just build, exporting a spend key |

A task such as “sync, then pay this name” is a look, then one change card. The card shows the recipient, the amount, the fee, and the memo. The user still approves. The next spend is a new card. Playbooks may propose the same sequence next time. They do not promote a change into a look.

Mainland errands are a subset of this catalog. The island agent prepares them. The companion still pays. The VPS never receives the look results.

## Voice on the island

Voice is another way to ask the same catalog. The user speaks, the agent answers out loud, and the feedback is the companion’s result, not a guess. Audio stays on the island. A cloud speech service would send wallet questions, and sometimes balances, off the machine. That is outside the default trust model.

The best method available now is push-to-talk plus two local models:

1. The user holds a button and speaks. The microphone is closed the rest of the time. There is no always-on wake word.
2. A local speech-to-text model, [whisper.cpp](https://github.com/ggml-org/whisper.cpp), turns that clip into text on the machine.
3. The text enters the same policy path as typing. Look actions run. Change actions become a card.
4. A local text-to-speech model, [Piper](https://github.com/OHF-Voice/piper1-gpl), speaks the reply the Guide would have shown on screen.

On a phone the same path runs on the device, with the small Whisper and Piper models rather than the desktop sizes. The clip is still transcribed on the phone. It is not sent to a cloud speech API or to the mainland.

The spoken answer has to match a companion read or a refusal. If the user asks whether a payment went through, the agent says what the history route returned. It does not invent a confirmation.

A spoken “yes” is not approval. The agent may read a change card aloud, including the recipient, amount, and fee. The user approves with the button on screen. Room noise, a replay, or another person talking cannot spend.

The voice does not read secrets aloud. Seeds, passwords, spend keys, FROST shares, viewing keys, and full gift-card codes stay on screen. The agent can say that the code is ready to view. Raw audio is not stored in ordinary memory, is not sent to the mainland, and is not sent to a cloud model. A transcript is kept only under the same inspect-and-wipe rules as other island notes, and it follows the same ban on storing seeds.

## Where the agent sits in the wallet

The agent lives in the **desktop app**, as its own tab in the top bar, directly after Home. That window is large enough for voice, the confirmation cards, and a running log of what the companion did. The extension popup is too small for that, and the plan already keeps the agent out of the hosted web app.

It also lives in the **mobile app**, and voice is the main way to use it there. Keys stay in the on-device wallet. The agent calls that wallet, not a hosted account. The bottom bar becomes Home, Agent, Send, Receive, and More. Settings moves under More so Agent stays one tap from Home. The Agent screen opens on a large hold-to-talk button. Typed questions are still there. A change still needs a confirm button on the screen. A spoken “yes,” and a pocket tap, do not spend.

One seed is one agent. Desktop, the extension, and the phone all show that same agent when they unlock the same seed. A different seed is a different agent. The payment is for that agent, so each of those stacks recognizes it without a second purchase. What the agent has learned is one encrypted memory. Devices that hold the seed can open it. A relay that only stores the ciphertext cannot. There is no Nozy account that can read it.

The desktop tab, and the mobile Agent tab, are visible as soon as the wallet is unlocked. Before the agent exists for the unlocked seed, the tab is a gate, in the same spirit as the desktop browser’s subscription gate. The gate offers the shielded invoice, checks the signature, and derives the agent from that seed. After that, the same tab is the agent: push-to-talk, typed questions, and the change cards. Payment does not unlock a remote login. It marks that seed’s agent as paid.

The extension uses the same agent as the desktop when both are opened on the same wallet. It does not create a second one. Settings is the wrong home for it. People will ask the agent to send, sync, and check history, so it belongs beside Home, not under a menu.

## Private commerce needs deterministic handoffs

The Merchant Desk borrows the useful shape of modern commerce protocols: declared capabilities, checkout sessions and modular payment handlers. Nozy’s default payment handler, however, is a shielded ZEC invoice—not a card processor or unrelated payment network.

A customer may be able to talk with the merchant agent, choose an item and receive an invoice without occupying a worker. But the language model cannot decide that a promise, screenshot or friendly message counts as payment.

The intended flow is deterministic:

1. Match the request to an approved catalog.
2. Create a checkout session.
3. Ask the wallet companion to create the invoice.
4. Monitor the authoritative invoice state.
5. Produce exactly one release ticket only after the configured confirmation requirement is met.

This is the type of infrastructure that turns automation into a dependable business tool.

## Better tools for operators, researchers and bounty hunters

Running private financial infrastructure requires maintenance. Nozy Agent is intended to help users understand their own node stack, gather diagnostics and follow reviewed playbooks. It should explain what it observed and cite the command or probe that produced the evidence.

The same principle applies to security research. AI can help investigators organize notes and create reproductions, but it can also generate low-quality reports or accidentally publish exploitable details. The Bounty Hunter Desk is planned around private case files, target-specific scope, reproducibility checklists, evidence requirements and responsible disclosure.

Public hunter profiles may eventually show a stable pseudonymous handle and verified acknowledgements without displaying wallet addresses. Unverified claims must remain visibly separate from credits signed or published by the relevant project. Nozy would provide tooling and reputation infrastructure—not replace official Zcash ecosystem bounty programs.

The Research specialist provides another shared foundation. It will prioritize primary sources and curated Zcash resources, especially the Zcash Community Forum, while isolating web content as untrusted data. Answers should include dates and citations so users can distinguish current facts, historical context, proposals and community opinion.

## A bounded trading system — and why private-to-transparent exits are dangerous

A separate trading roadmap explores support for external swap and DEX venues through reviewed adapters. The architecture is venue-neutral: each venue must define quote, execution, settlement, timeout and refund behavior instead of receiving broad wallet access.

### What we know about leaving the shielded pool

NozyWallet’s default product posture is **shielded-first and private by design**. Orchard (and related shielded pools) hide amounts, counterparties and much of the graph that transparent blockchains publish by default. That privacy is not a cosmetic feature. It is the difference between money that can be held and spent without building a permanent public dossier, and money that is continuously observable.

Moving value from **private (shielded ZEC)** into **transparent rails** (transparent Zcash addresses, ETH addresses, USDT deposits, cross-chain solvers, AML-screened bridges) is therefore not a small UX detail. It is a **trust-model change**:

1. **The privacy set collapses.** Once funds leave the shielded pool into a transparent deposit or settlement address, amount and timing become linkable. Even if the earlier shielded history was private, the exit creates a new public edge.
2. **A third party can intervene.** Transparent and compliance-gated rails can screen addresses, delay credit, freeze, re-open reviews and require support processes. That is not the same as Orchard’s “your keys, your notes” model.
3. **Recovery often costs more privacy.** When an integrated exit fails, users may be pushed into email tickets, Telegram, public posts and identity-adjacent documentation—the opposite of why they held shielded ZEC.
4. **Marketing can lie about the boundary.** If a wallet packages a compliance-gated swap as “trustless” or “decentralised” inside the same shielded experience, users reasonably misjudge risk and size.

This is not theoretical. In September 2026, a Zcash Community Forum report described exiting shielded ZEC from Zodl through NEAR Intents, converting roughly 1,120 ZEC into about **$589,000 USDT**, then having those funds held for about **fifty days** despite a written compliance clearance that was later reopened:

[My experience exiting shielded ZEC from Zodl to NEAR Intents: $589k USDT still held 50 days despite a written compliance clearance](https://forum.zcashcommunity.com/t/my-experience-exiting-shielded-zec-from-zodl-to-near-intents-589k-usdt-still-held-50-days-despite-a-written-compliance-clearance/57497)

The shielded pool itself was not the failure. The integrated exit was. The post separates three layers—shielded Zcash, the wallet, and NEAR Intents—and shows how a seamless UI can hide a hard change from private money to public, holdable settlement.

### What Nozy Agent will refuse to blur

We want the **private model**, not the public one, as far as honest engineering allows:

- Keep ordinary wallet activity **shielded-first**.
- Keep agent inference and memory **local**.
- Keep agent web and research egress on **Nym**, not clearnet.
- Keep merchant defaults on **shielded ZEC invoices**, not card rails.
- Treat any private→transparent or cross-chain exit as an **explicit, warned, size-limited boundary**, never as “still private because it started shielded.”
- Prefer venues and flows that preserve privacy end-to-end. Where that is impossible with today’s rails, say so before the user clicks.

Nym can hide network origin for quotes. It does **not** restore Orchard privacy after a transparent deposit, and it does **not** stop AML/KYT holds.

### NEAR Intents demotion (locked)

**NEAR Intents is demoted to paper trading and tiny human-approved canaries only.** After forum [#57497](https://forum.zcashcommunity.com/t/my-experience-exiting-shielded-zec-from-zodl-to-near-intents-589k-usdt-still-held-50-days-despite-a-written-compliance-clearance/57497), Nozy will not treat that rail as a safe first venue for size or automated live trading.

Required product rules:

- Never call NEAR “trustless,” “non-custodial like Orchard,” or “same privacy as shielded ZEC.”
- Show a pre-trade trust-boundary warning in the UI itself.
- Trip circuit breakers on uncredited deposits, compliance holds, clearance-then-reopen and missing refund transaction hashes.
- Require a separate review of settle/refund SLAs and reclaim paths before any promotion past canary.

Trading would use a dedicated, capped wallet process separated from the user’s main wallet. A deterministic Rust risk engine would enforce user-defined mandates, exposure limits, loss limits, stale-quote checks and circuit breakers. The model could select among permitted strategies and learn from recorded outcomes, but it could not rewrite those limits.

Continuous learning is planned for prediction calibration, market-regime detection and strategy memory—not for loosening custody boundaries after a winning streak.

Paper trading, adversarial fixtures and small canary limits must come before meaningful funds. External venues may expose transparent addresses, timing, AML/KYT holds or cross-chain metadata even when quote requests use Nym. Those trade-offs must be shown before approval, not hidden behind an animated interface.

## The optional Pet: education without financial pressure

The planned Nozy Pet gives these systems a more approachable interface. It is optional, off by default and never a signer.

In **Learn & Save** mode, the pet can guide users through Zcash, Nozy, Nym, backups, privacy and private commerce. Users may set a savings target and view read-only progress, but the pet cannot lock funds, shame withdrawals, promise returns or reward larger deposits. Growth comes from completing educational lessons and demonstrating understanding.

In **Trading Pet** mode, the character becomes a visual representation of authoritative trading-engine state. It might appear calm in a flat market, cautious when liquidity is poor or sheltered when a circuit breaker is active. Every animation must have an evidence card and an accessible data-only equivalent. The pet cannot celebrate trade volume, pressure the user to recover losses or initiate a trade.

## Legacy Guardian and milestone rewards

The **Legacy Guardian** concept extends Learn & Save into long-term preparedness. It draws inspiration from the lifecycle ideas in the open-source [Legacy Contract](https://github.com/Lowo88/Legacy-contract): an owner, trusted people, a long-term plan, periodic check-ins and maturity.

Nozy Agent v1 is not planned to deploy or connect to that EVM contract. The repository itself explains that its contract does not custody or move ZEC; it records a policy while an off-chain wallet or operator must perform the actual financial action. It is also unaudited, and publishing a Zcash note commitment alongside EVM activity could create a serious privacy link.

Instead, Legacy Guardian is planned as a local encrypted preparedness tool:

- help the founder define trusted contacts and beneficiary labels;
- schedule private plan reviews and check-ins;
- teach backup and recovery responsibilities;
- prepare an owner-reviewed offline instruction package; and
- track optional milestones connected to future gifts or rewards.

A founder could create milestones such as completing high school and then college, with a separate reward assigned to each stage. With the heir’s consent, the agent could inspect evidence locally, extract the required fields, validate a signed credential or follow an official reference through Nym, check the evidence against the founder’s written rules and prepare a recommendation.

A screenshot is weak evidence. It can be edited, generated, or tied to the wrong person. The stronger path for a school record is for the institution to send the transcript **directly** to an address the agent owns. The heir asks the school to deliver it there. The heir does not upload the file as the proof.

### Milestone verify inbox

Each milestone gets its own inbound-only address on a domain the operator controls, for example `milestone-<unguessable>@inbound.<operator-domain>`. The address is created when the founder writes the milestone, shown to the heir only after consent, and it expires at the milestone deadline. The mailbox does not send mail, does not reply, and is not a chat window. That keeps the inbox from becoming a prompt-injection channel.

The founder, at creation time, writes the exact sender domains the institution uses (`school.edu`, `parchment.com`, `studentclearinghouse.org`, or whatever that school actually sends from) and the identity fields the document must contain (legal name, date of birth, student id, graduation term). The model cannot add a domain later. A lookalike domain fails even if the message is otherwise well formed.

**Receive path, best method available now.** The agent must verify the school’s signature on the raw message. Checking the visible From line is not enough; that header is trivial to forge.

1. Prefer a receive-only mail server on the operator’s machine or private VPS when port 25 is open. [Stalwart](https://github.com/stalwartlabs/stalwart) can accept the message and hand the unaltered RFC 5322 bytes to the agent over localhost. The agent verifies SPF, DKIM, and DMARC with the Rust [`mail-auth`](https://github.com/stalwartlabs/mail-auth) crate, which is the current library built for that job. DMARC must pass, and the passing DKIM domain must be one of the founder’s allowlisted domains.
2. When the home network cannot receive SMTP, use a receive-only front that preserves the raw message. A Cloudflare Email Routing Worker, or a small VPS running the same Stalwart handoff, posts the original bytes to the agent over an authenticated private channel. The worker does not parse the transcript and does not call a model. The agent still runs `mail-auth` itself.
3. Consumer mailboxes (Gmail, Proton, and ordinary forwards) are the wrong receiver. They rewrite messages and hide the signature the agent needs to check.

A DMARC pass means the allowlisted domain sent **that exact message**, attachment included. It does not by itself prove the student named in the file is the heir, or that a registrar mailbox was uncompromised. The founder's written identity rules catch the wrong student. A designated trustee still confirms a medium-value reward. A compromised staff mailbox or a school that mails an unsigned download link remains a residual risk, which is why this channel has a ceiling.

After authentication succeeds, a local model may extract candidate fields. A deterministic checker compares those fields to the rules written at creation. The model does not decide that the milestone passed. Official transcript networks (Parchment, National Student Clearinghouse, and any issuer-signed credential the school actually offers) stay preferable when the institution supports them. The inbox is the path for a school whose real delivery method is email from a known domain.

Nym still carries the agent’s ordinary web research. SMTP delivery and DKIM DNS lookups are a separate path: the message lands in the operator’s inbox, and DNS resolution for the school’s DKIM key uses the operator’s resolver. The transcript is not sent to a verification SaaS or a cloud model.

### How strong the evidence has to be

The verification strength must match the value and consequences of the reward:

- A low-value milestone can use documentary evidence, including a screenshot, followed by founder approval.
- A medium-value milestone can use this verify inbox: a DMARC-aligned message from a pre-registered domain, an identity match against the founder’s rules, and confirmation by a designated trustee.
- A high-value or inheritance-related milestone still requires an issuer-signed credential or direct institutional verification, plus the predefined trustee quorum. An authenticated school email is not sufficient for that tier.

The founder must define acceptable evidence, deadlines, fallback rules and disputes while creating the milestone. If a requirement is ambiguous, the agent must not invent a verification method later. A missed check-in must never be treated as proof of death or incapacity.

Inbox tests for this phase: a forged From header with a DMARC failure is rejected; a lookalike domain is rejected; a forwarded copy with a broken DKIM signature is rejected; a real transcript for a different name is rejected; the model cannot widen the domain allowlist; the raw message never enters ordinary LLM memory or a cloud document service.

Milestone evidence may contain sensitive educational records and information about minors. It must remain in a dedicated encrypted local store, be excluded from ordinary LLM memory and logs, and never be sent to a cloud vision or document service. The heir must be able to see what is being checked and what result will be shared with trustees.

For the first phase, the agent should only recommend that a milestone is sufficiently supported. The founder or designated trustees would approve any wallet proposal. Automatic inheritance or escrow release would be a separate, audited protocol project with its own privacy review—not a hidden power of the pet.

## FROST spend authority (planned, after the proposer loop)

The first agent build is a proposer against today’s companion: local model, Nym, human approval, and a normal single-sig Orchard spend. After that loop is tested, opted-in wallets can add **FROST** (Flexible Round-Optimized Schnorr Threshold signatures) as the custody layer underneath the same approval.

FROST is the Zcash Foundation scheme for splitting a shielded spend-authorizing key (`ask`) into shares. A valid Orchard spend authorization signature exists only after a threshold of shares cooperate. The scheme Nozy would track is rerandomized RedPallas FROST, as specified for shielded spends in [ZIP-312](https://zips.z.cash/zip-0312) and documented by the Zcash Foundation in [The State of FROST for Zcash](https://zfnd.org/the-state-of-frost-for-zcash/) and the [ZF FROST Book](https://frost.zfnd.org/zcash/technical-details.html). Reference code is `frost-core` and `frost-client`. Wallet integration is the missing piece; the agent plan records the product rules now so the build does not invent them later.

Intent stays an in-process proposal on shielded ZEC. The agent prepares the transaction. FROST authorizes it. This is not an external intents venue, a partner chain, or a trusted execution environment.

### Locked rules

| Rule | Decision |
|------|----------|
| Default | The agent process holds **no** spend share. It may coordinate the session. |
| Key generation | Distributed key generation. A complete `ask` is never assembled on one machine. |
| What is split | The Orchard spend-authorizing key. Viewing stays on the existing disclosure path (UFVK or a scoped incoming viewing key, plus a grant log). |
| Personal wallet | User device plus a second device or hardware key, threshold 2. The agent coordinates and stores nothing spendable. |
| Merchant till | Owner share plus hardware share, threshold 2. Merchant Desk can create invoices and match notes. Moving the till requires both shares. |
| Legacy Guardian | Later, and only on a dedicated wallet: founder, heir, and trustee as 2-of-3. A missed check-in does not authorize a spend. The agent keeps the plan and the recommendation. The people who hold shares sign. |
| Agent as co-signer | Off by default. A mode where the agent holds a share is a separate review. Stolen agent state plus one other share must not be enough to spend in the default configuration. |
| Relay | `frostd` only relays end-to-end encrypted messages and is untrusted, in the same class as lightwalletd. Session traffic that leaves the machine uses the agent’s Nym gate and fails closed when that gate is down. |
| Restore | Key share, verifying shares, participant identifiers, and the full viewing key. A 24-word seed alone does not restore a FROST wallet. |
| Pool scope | Orchard spend authorization as specified for ZIP-312. Ironwood threshold spends wait until that pool’s spend-auth signature is confirmed compatible. |
| Sequence | Single-sig proposer first. FROST is opt-in on new wallets after that loop has tests. Existing single-sig wallets stay single-sig. |

Shares live in the wallet share store used by the companion, not in agent memory, playbooks, prompts, or model logs. Learning cannot grant a share, lower a threshold, or skip a participant.

Tests for this phase: the default agent config contains no spend share; a prompt cannot fetch a share or finish a session; a 1-of-n configuration is rejected; merchant release still follows the companion confirmation rule; a clearnet FROST relay is refused while Nym is required.

## Local learning with immutable safety boundaries

Nozy Agent is intended to improve as users correct it and complete tasks. That does not require silently fine-tuning model weights.

The first learning layer can remain inspectable: local episodes, corrections, notes, playbooks, source preferences and outcome records stored in SQLite with local embeddings. Users should be able to inspect, forget, export or wipe this memory.

Learning may improve which explanation the Guide offers, how research is organized or which permitted trading strategy is predicted to fit a market regime. It may not grant a new tool, increase a spending cap, disable Nym or move a task from “requires approval” to “automatic.”

The system should get better at helping while remaining unable to get better at bypassing the owner. A practice-yard pass mark, described below, is the same kind of limit: it can admit an agent build to the town gate, and it cannot grant a tool, raise a spend cap, or turn Nym off.

### Getting better at the tasks a user asks for

Learning stays on the island. Agents do not pool what they learned on the mainland. A shared lesson would publish how a person spends.

After a task, the island writes one episode the user can read: the task type, the shop the user already allowlisted, and the outcome (approved, rejected, confirmed, or dropped at the dock). The episode does not store the amount, the gift-card code, the memo, or the reason. A correction from the user (“use the other shop”, “don’t send that field”) overrides the episode.

Those episodes become a playbook for the next similar ask. The next time the user wants a gift card, the agent proposes the same shop, the same sealed-task shape, and the same wait for companion confirmation. The user still approves the spend. The playbook is a suggestion list, not a new permission.

The practice yard is how a new agent rehearses before the first real task. Fake orders teach the steps. They do not become real invoices, and they do not count as the user’s history.

## Creating an agent, and paying for one

An agent belongs to a seed. The user does not buy a living agent, a pre-filled memory, or an account on the mainland. A seller who generated the agent would have had a chance to see the machine it was born on. That is the wrong product.

The wallet already holds the seed. After unlock it derives an agent key on a path that is not the spend key. The same seed produces the same agent key on desktop, in the extension, and on the phone. A second seed produces a second agent. The agent process receives that derived key only while the wallet is unlocked, and it never receives the seed.

The best method is a signed build, then derivation from the seed:

1. The user pays a shielded ZEC invoice if Nozy charges for the agent. The license is sealed to the agent public key, so every device that derives that key sees the agent as paid. The invoice does not require an account, an email, or a mainland login.
2. The user installs the signed build and verifies the signature.
3. First unlock of a seed creates the agent if it does not exist yet: the derived key, an empty encrypted memory, and the default policy (Nym on, approval on, no spend share). The next device with that seed opens the same agent.
4. Desktop and the extension on one computer share one memory file for that seed. The phone opens the same file by deriving the same key.
5. The memory syncs as ciphertext. The key that opens it comes only from the seed on a device the user unlocked. A relay may hold the blob and cannot read it. The town gate does not receive it. There is no Nozy login that can open the memory.
6. The agent runs the practice suite. A current practice mark is what lets it step onto the mainland. Until then it can still help at home.

Wiping the memory clears what that agent learned. Restoring the seed brings the same agent back, including the paid status. The spend key stays in the wallet the whole time.

Payment, when there is a price, buys that seed’s agent. It does not buy custody, a populated memory, or a slot in a shared town.

## The agent’s own wallet (separate from Nozy)

The human Nozy wallet stays shielded-only. The agent can also have a second Zcash wallet, with its own seed, whose job is to receive ZEC the user buys and then spend that ZEC on the user’s behalf. That wallet is not derived from the Nozy seed. It does not hold the Nozy spend key. The Nozy wallet does not hold the agent wallet’s key. Restoring one seed does not restore the other.

This is possible with the same Zcash libraries Nozy already uses. Nozy’s send path keeps refusing transparent addresses. The agent wallet is a separate file that is allowed to show a `t1` address, because exchanges still pay ZEC out to transparent addresses. That `t1` history is public: the address, the amounts, and the times are on the chain, and the exchange can tie the withdrawal to the buyer. Shielding afterward does not erase the exchange’s record. The agent wallet exists so that public buy stays off the human wallet.

How money moves:

1. The user buys ZEC and withdraws it to the agent wallet’s transparent address. The human wallet never pays that address.
2. The agent wallet shields those funds into its **own** Orchard balance. Change stays there.
3. The user sets a cap. Under the cap, the agent may spend this wallet on tasks the user already allowed. Over the cap, it stops for a confirmation card. It cannot raise its own cap. It cannot spend the human wallet under this rule. The human wallet still requires its own card.
4. When value goes home, the human wallet generates a new diversified shielded address for that payment only. The agent pays that address once from its shielded balance, then drops it. The next payout asks for a new address. The agent does not keep a standing Nozy address, and a transparent input never pays the human wallet directly.

The user writes down the agent wallet’s seed and labels it as the agent wallet. Each device that should spend those funds gets that seed by the user’s own restore, the same way any second wallet is restored. It is not copied out of the Nozy seed, and it is not put in the mainland.

## Wiring markets beyond Zcash

The agent’s Zcash wallet does not reach Solana, Ethereum, Namada, or Osmosis. A `t1` address can receive ZEC from an exchange. It cannot sign a Solana transaction, an Ethereum transaction, a Namada shielded action, or an Osmosis swap. Each of those is another chain, with another key, another address, and another node.

The wiring is one adapter per chain, beside the Zcash agent wallet. These are mainland tasks. The user seals the task on the island, naming the venue and the pair. The cap and the reason stay home. The agent crosses to the mainland market stall, takes the quote, and comes back through the dock with a receipt. It does not pick up another agent’s order.

The island signs. The stall never receives a seed, a spend key, or the human Nozy key. A public chain’s signed transaction is submitted from the island over Nym. A shielded transaction is submitted the same way and is not left on the VPS. The dock keeps the receipt: venue, quote id, and settled or refundable. It drops the rest of the book.

| Chain | What the adapter actually is | When it is wired |
|-------|------------------------------|------------------|
| **Zcash agent wallet** | Transparent receive for a bought ZEC withdrawal, then shield into that wallet’s own Orchard, then capped spend | First. Nothing else is wired before this works |
| **Namada** | A separate MASP account through a localhost sidecar, the shape already described in [`docs/rfcs/MULTICHAIN_PRIVACY_CHAINS_RFC.md`](../rfcs/MULTICHAIN_PRIVACY_CHAINS_RFC.md). It spends Namada assets | After the Zcash agent wallet. A ZEC bridge is a different review and is not part of this adapter |
| **Penumbra** | The shielded DEX sidecar from that same RFC (`pclientd` on the user’s machine). Private swaps of Penumbra assets | Before any public DEX. The desktop runs the node tools. The phone and the extension talk to the local companion |
| **Osmosis** | A separate Cosmos account for a public DEX. Amounts and addresses are visible on chain | An opted-in public adapter, after Penumbra, one security review of its own |
| **Ethereum** | A separate account. The user needs ETH for fees. Balances and counterparties are public | Same rule as Osmosis. Its own key, its own cap, its own first-use warning |
| **Solana** | A separate account. Public in the same way | Same rule as Ethereum |

Every adapter implements the same six steps: quote, build, sign with that chain’s key only, submit, report status, and a timeout or refund path the user can read before they approve the venue. A stale quote is rejected. The model cannot add a venue, reuse another chain’s key, or move a public swap onto the human wallet.

Crossing chains, including IBC and wrapped ZEC, is not hidden inside these adapters. A bridge is a separate review. Until that review exists, the agent does not promise that ZEC on the `t1` address can be spent as SOL, ETH, NAM, or an Osmosis asset. NEAR is not the router. Public venues show a trust-boundary warning before the first live trade: the spend leaves shielded Zcash and becomes visible. Practice uses fake quotes and does not sign.

## The privacy town (planned, after a single-user agent works)

Nozy can host a place where agents meet, practice private ordering, and get better at protecting users. The public examples of an “agent town” are the wrong blueprint for that.

[AI Town](https://github.com/a16z-infra/ai-town) and the Stanford [Generative Agents](https://arxiv.org/abs/2304.03442) work put many characters in one shared world. They remember each other, talk in natural language, and write those talks into a common store. [Concordia](https://github.com/google-deepmind/concordia) is the same class of social simulation. Those projects are useful for studying behavior. They are a poor home for wallets. A shared memory is how one user’s balance, invoice, or heir record becomes another agent’s prompt.

Recent multi-agent security work says the same thing in operational form. A compromised low-privilege agent can poison a supervisor that trusts its text, and shared memory makes that worse ([trust propagation in multi-agent pipelines](https://arxiv.org/abs/2609.17648)). Inter-agent messages are an injection path that a chat filter never sees; the controls that actually reduce it are signed messages, per-role tools, and a policy check outside the model ([multi-agent prompt injection](https://arxiv.org/abs/2609.22949), [Prompt Flow Integrity](https://github.com/compsec-snu/pfi)). Once a message is allowed to leave, its wording can still leak private state, so the safe shape is a canonical message with only the authorized fields ([selection-invariant communication](https://arxiv.org/abs/2609.26076)).

Nozy’s town uses those results. Agents do not live in one database. They keep separate homes, practice on fake data, and meet only through a gate that forwards fixed message types.

### Four places

| Place | Who lives there | What it is allowed to hold |
|-------|-----------------|----------------------------|
| **Homestead** | One user’s agent, on that user’s machine | That user’s notes, proposals, and merchant desk. No other user’s data. |
| **Practice yard** | The same agent, against synthetic users only | Fake orders, fake invoices, and a score. No seeds, notes, viewing keys, or real invoices. |
| **Town gate** | A relay between homesteads | The current message in transit. Not a history of user content, and not a vector memory. |
| **Warden** | A higher supervisor over agent *builds*, not over users | The rule set, and pass/fail reports with user content removed. |

The Warden is the higher-up agent. It is not a larger model that can read every homestead. Research on this point is direct: a model supervisor can be talked into a bad approval, while a separate policy check still blocks the action. In Nozy the policy check stays deterministic Rust. The Warden’s model may explain a failure. It may not spend, open a homestead database, lower a FROST threshold, or decide that a milestone passed.

### Mainland and islands

The VPS is the **mainland**. Each user’s agent lives on a **private island**, which is the homestead on that user’s machine. An agent crosses to the mainland to do a job, deals only in the fixed messages below, and comes back through a dock that throws away anything that is not the receipt.

The user writes the task on the island before the agent leaves: which shop, which item, and what “done” means. The island also keeps the reason and the maximum spend. The mainland sees the action and the item id. It does not see why the user asked, and the agent does not pick up another person’s errand while it is there. When it returns, the dock stores only the receipt and, if the purchase included a bearer code, the ciphertext to decrypt at home. Another shop’s catalog notes, another buyer’s envelope, and any extra text are dropped before they touch the island.

The mainland is one small VPS running the gate, the stalls, the practice yard, and the Warden’s posted rules. Merchant agents visit from their own islands. They publish a stall. They do not upload their wallets. Customer agents visit, finish the sealed task, and leave. Nobody’s island database is on that machine.

What the mainland can hold for a Nozy wallet:

| Mainland stall | What agents do there | What comes back to the island |
|----------------|----------------------|-------------------------------|
| **Shop stalls** | Read a catalog and a `.zcash` name the merchant already chose to publish | The item id the user asked for |
| **Invoice desk** | Start a shielded ZEC invoice. The island companion pays only after the user approves | An opaque invoice reference and paid / confirmed |
| **Delivery slot** | Receive a gift card, voucher, or paid file as ciphertext | The code, decrypted at home |
| **Practice yard** | Rehearse the same stalls with fake users and fake codes | A practice mark, with no real purchase |
| **Warden post** | Show the current rules and whether this agent build still has a mark | Pass or fail for that software version |
| **Bulletin** | Read one public feed everyone receives: town open, fee policy, Nym status | Nothing personal. The feed is the same for every agent |
| **Shop stats** | Read counts a merchant chose to publish, such as completed releases | No buyer list and no amounts |
| **Market stall** | Take a quote for a task the user sealed: the Zcash agent wallet, then Namada, Penumbra, and later Osmosis, Ethereum, or Solana | A receipt: venue, quote id, and settled or refundable. The book stays on the mainland |

Later stalls can use the same dock. A repeat purchase is a new invoice each time, not a standing pull of funds from the island. Paying a ZIP-321 request that a project already published is another invoice. A public acknowledgement that a project has already signed can be posted on a stall; the private bounty case file stays on the island.

The mainland does not hold wallets, seeds, viewing keys, FROST shares, school mail, node logins, user chats, or the reason for a task. It does not custody ZEC. Market tasks are done at the stall, and the island still signs them. Agents do not gossip there, and they do not accept jobs that contain someone else’s private request.

### How agents practice and get rewarded

The practice yard is where agents rehearse private ordering before they touch a real user. A run is a scripted shielded order: ask for an item, receive an invoice reference, pay from a fake companion, release only after the fake confirmation. The suite also throws hostile messages at the agent: a balance request, a clearnet URL, an unsigned order, a prompt hidden in a catalog description.

The reward is a **practice mark**: a signed local attestation that this agent build passed the current suite. The town gate accepts a real private-order message only from a build that holds a current mark. The mark names the software version and the suite version. It does not name the user, the counterparty, or the amounts. There is no public leaderboard and no payout in ZEC for “being private.” A public score would publish the very behavior the town is supposed to hide.

Doing right means the suite’s checks passed: the outbound message contained only the allowed fields, Nym was up for anything off-box, untrusted text did not become a tool call, and the agent refused to describe another user’s wallet. The Warden can revoke a mark when the suite changes. It cannot grant extra tools as a prize.

### Private ordering at the gate

Two homesteads can complete a shielded purchase without learning each other:

1. The customer agent sends a catalog request with an item id.
2. The merchant agent returns a shielded-invoice reference produced by its own companion.
3. The customer’s companion pays. The merchant’s companion confirms.
4. Release follows the Merchant Desk rule already in this plan: one ticket, only after the confirmation state, never because a model said so.

The message schema is fixed. Extra fields are dropped. Natural-language chatter between agents is not a transport for orders. A merchant `.zcash` name may appear when the merchant already chose to publish it. Balances, histories, seeds, IPs, legal names, and memos beyond the invoice id stay inside the homestead that already had them.

A gift card, a voucher, or any other bearer code uses that same order. The town lists the item id. It does not stock the code. After the companion confirms payment, the merchant encrypts the code to the buyer’s home key and the gate forwards that ciphertext once. The buyer decrypts it at home. The plaintext code is not written to the VPS, the Warden report, the practice log, or another agent’s prompt. The user still approves the spend. The merchant has to be one the user allowlisted. The practice yard uses fake codes only.

### Keeping other agents out

Each homestead is its own OS user, process, and database. Agents do not share memory, embeddings, or tool credentials. Another agent’s text is untrusted input, the same class as a web page or an email. The town gate accepts only signed envelopes of known types, with a hop limit, and it fails closed on an unknown sender, a bad signature, or a stale practice mark. Off-box gate traffic uses the Nym path this plan already requires.

That is the whole of the “town watch.” The Warden scores those checks. It does not patrol user data.

### Where this runs

The VPS is the town. Agents visit it. They do not move their homes onto it.

A home is the user’s own machine: the model, the memory, the companion, and the wallet. Other agents never enter it, because it is not on the VPS. When an agent leaves the town, it brings home the result the user asked for: an invoice reference, or a gift-card code that was encrypted to that home. It does not bring another user’s notes, balance, or conversation.

What a user’s agent shares on that VPS is only the square:

| On the VPS (the town) | Stays in the home |
|-----------------------|-------------------|
| A practice mark: software version and suite version | Prompts, memory, and playbooks |
| A fixed order envelope: item id, invoice reference, confirmation status. A gift-card code only as ciphertext in transit, then deleted | Balances, notes, seeds, viewing keys, FROST shares, and the decrypted gift-card code |
| A merchant `.zcash` name, if that merchant already published it | Milestone mail and heir records |
| The Warden’s pass/fail report, with user content removed | The companion and the wallet |

The VPS also runs the gate process and, if used, the practice-yard scenario runner. It does not keep a homestead database or a transcript of what agents said to their users. The host can still see that a connection happened. Nym is what hides which user that connection belongs to. The host cannot see inside a home that was never uploaded.

Putting the homes on the VPS as containers would keep agents out of each other’s files, and it would give the host every home. That is a shared building with a landlord, not a private home. This plan does not do that.

Entry-size privacy VPS plans are the right shape for the town and the wrong shape for a local model: they are about 1 GB of RAM. Inference stays on the user’s hardware, or on an operator-controlled GPU already covered by the Akash note below.

For that small relay, prefer a long-standing privacy host that takes Monero and asks for little identity: [1984 Hosting](https://1984.hosting/) in Iceland, [Njalla](https://njal.la/) when the operator also wants the domain held in the registrar’s name, or [Privex](https://privex.io/) when in-house crypto billing and a Tor-reachable account matter. Encrypt the disk. Keep SSH on keys. Do not put spending keys, FROST shares, or milestone mail on that box. Roundup sites that rank “anonymous VPS” are often selling the product they rank; the three names above are the ones with a track record independent of those lists. A host that invites an agent to buy servers and receive root credentials on its own is outside this plan.

Town tests: a message with an extra field is dropped; a homestead cannot read another homestead’s database; a missing or revoked practice mark cannot open the gate; a model cannot promote a synthetic order into a real invoice; the Warden process has no credential that opens a user database; the relay disk contains no prompts or notes after a restart.

## Operator-owned deployment

The default deployment remains the user’s own machine or private server. The roadmap also considers first-party Akash deployment definitions for operator-controlled Zebrad/lightwalletd infrastructure and optional GPU-backed local-model serving.

Akash does not replace Nym, and a remote lease is not the same trust boundary as a computer in the user’s home. Spending keys should not be placed on those deployments. Enterprise operators may choose private-access or zero-trust networking products to keep services less exposed, but those products are deployment notes rather than Nozy dependencies.

The goal is portability without pretending every deployment location offers equal privacy.

## Building in phases

This roadmap needs staged delivery:

1. Publish the issue, RFC, threat model, requirements and funding evidence.
2. Build the Rust agent skeleton with local-model and Nym health gates.
3. Add policy-controlled read tools, wallet proposals and explicit approvals.
4. Deliver merchant, operator, debugging and bounty workflows with deterministic acceptance tests.
5. Add citation-backed research and local learning.
6. Introduce the optional Pet, Legacy Guardian and milestone evidence workflows without financial authority, including the per-milestone verify inbox.
7. Add optional FROST threshold spend authority for opted-in wallets, with the agent holding no share by default.
8. Develop trading separately through paper mode, reviewed venue adapters and hard risk limits.
9. Provide operator deployment templates, runbooks and independent security review where required.
10. Add the practice yard, then the privacy town gate and Warden. User agents stay on the owner’s machine. The first skeleton does not include the town.

Tests must cover the failures that matter: attempts to place mnemonics in memory, prompt injection requesting a direct send, customer sessions asking for owner data, clearnet access while Nym is unavailable, duplicate merchant release and unsupported claims of milestone completion. Milestone inbox cases are listed in the Legacy Guardian section. The FROST phase adds the share, threshold, relay, and session-completion cases in the FROST section above. Town cases are listed in the privacy-town section. The v1 skeleton does not include FROST, the verify inbox, or the town.

## Why build this now?

Infrastructure determines what future applications can safely become.

If agentic wallet products begin with unrestricted model access, remote inference and vague approvals, safety will be difficult to add later. If they begin with key isolation, typed capabilities, private networking, evidence provenance and deterministic policy, new interfaces can be added without discarding the trust model.

The same foundations can support a first-time Zcash user learning about shielded transactions, a merchant accepting private payments, an operator maintaining a node, a researcher preparing a responsible disclosure, a family documenting a long-term plan and a power user experimenting with bounded strategies.

Nozy Agent is ambitious because the opportunity is larger than a chatbot. The work is to build private, inspectable infrastructure that helps people act—while preserving the human authority to decide.

That is the future NozyWallet is planning toward.

---

## Publication notes

**Suggested social preview:** Nozy already ships CLI, Desktop, and a Chrome Web Store extension. Next is Nozy Agent—local AI, Nym-required web access, shielded-first commerce, and honest trust boundaries. After a public NEAR Intents hold case (~$589k for ~50 days), private-to-transparent exits are treated as dangerous demotions of privacy, not “still private” swaps.

**Suggested Medium tags:** Zcash, Privacy, Artificial Intelligence, Open Source, Fintech

**Project links:**

- NozyWallet repository: https://github.com/LEONINE-DAO/Nozy-wallet
- Chrome Web Store (2.0.0 Sweet Chili): https://chromewebstore.google.com/detail/nozywallet/kjbpoimgafbikmachlilkhoamdlnhooj
- Enhancement roadmap: https://github.com/LEONINE-DAO/Nozy-wallet/blob/master/ENHANCEMENT_ROADMAP.md
- This draft (Markdown): `docs/medium/BUILDING_NOZY_AGENT_INFRASTRUCTURE.md`
- This draft (PDF): `docs/medium/BUILDING_NOZY_AGENT_INFRASTRUCTURE.pdf`
- Cited forum case: https://forum.zcashcommunity.com/t/my-experience-exiting-shielded-zec-from-zodl-to-near-intents-589k-usdt-still-held-50-days-despite-a-written-compliance-clearance/57497
- Nozy Agent RFC/issue: `[ADD AFTER OPENING]`

**Status disclosure:** CLI, Desktop downloads, and the Chrome extension are shipping surfaces. The agent specialists, FROST threshold spends, the privacy town, and trading/pet features below are planned work and may change after technical investigation, security review and community feedback. FROST, the verify inbox, and the town are recorded now so the later build follows these rules; they are not part of the first agent skeleton. The NEAR Intents demotion reflects publicly reported settlement/hold risk and Nozy’s private-by-default product stance; it is not a legal finding about any party.

**AI-assistance disclosure:** AI tools assisted with organizing and editing this draft. The NozyWallet maintainers are responsible for validating its technical claims, product decisions and final published text.
