//! Valar `zcash_voting` integration (config auth, active round, hotkey, bundles).

use anyhow::{anyhow, bail, Context, Result};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use zcash_voting::config::{
    resolve_dynamic_voting_config, resolve_static_voting_config, ResolveVotingConfigOptions,
    ResolvedVotingConfig,
};
use zcash_voting::prelude::{
    generate_random_voting_hotkey, BundleLayout, Network, NoteInfo, VotingDb, VotingHotkey,
    WitnessData,
};
use zcash_voting::{PirClientBlocking, VotingRoundParams};

use crate::config::Environment;
use crate::urls;

pub fn voting_db_path(data_dir: &Path) -> PathBuf {
    data_dir.join("voting.sqlite")
}

pub fn hotkey_path(data_dir: &Path, network: NetworkKind) -> PathBuf {
    data_dir.join(format!("hotkey-{}.bin", network.as_str()))
}

#[derive(Debug, Clone, Copy)]
pub enum NetworkKind {
    Mainnet,
    Testnet,
}

impl NetworkKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mainnet => "mainnet",
            Self::Testnet => "testnet",
        }
    }

    pub fn to_sdk(self) -> Network {
        match self {
            Self::Mainnet => Network::Mainnet,
            Self::Testnet => Network::Testnet,
        }
    }
}

/// Resolve static+dynamic config with Ed25519 round authentication.
pub fn resolve_config_sdk(source: &str) -> Result<ResolvedVotingConfig> {
    let (static_url, _) = crate::config::parse_pinned_source(source)?;
    let static_bytes = http_get(&static_url)?;
    // Hash pin is enforced inside resolve_static_voting_config via the source string.
    let resolved_static = resolve_static_voting_config(source, &static_bytes)
        .map_err(|e| anyhow!("resolve static config: {e}"))?;
    let dynamic_bytes = http_get(&resolved_static.dynamic_config_url)?;
    resolve_dynamic_voting_config(
        resolved_static,
        &dynamic_bytes,
        ResolveVotingConfigOptions::default(),
    )
    .map_err(|e| anyhow!("resolve dynamic config: {e}"))
}

fn http_get(url: &str) -> Result<Vec<u8>> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(format!("nozy-vote/{}", env!("CARGO_PKG_VERSION")))
        .build()?;
    let resp = client
        .get(url)
        .header("Cache-Control", "no-cache")
        .send()
        .with_context(|| format!("GET {url}"))?;
    if !resp.status().is_success() {
        bail!("GET {url} → HTTP {}", resp.status());
    }
    Ok(resp.bytes()?.to_vec())
}

#[derive(Debug, Deserialize)]
struct ActiveRoundResponse {
    round: Option<ActiveRound>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ActiveRound {
    pub vote_round_id: String,
    pub snapshot_height: u64,
    #[serde(default)]
    pub snapshot_blockhash: Option<String>,
    pub vote_end_time: u64,
    pub nullifier_imt_root: String,
    pub nc_root: String,
    pub status: u32,
    pub ea_pk: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub proposals: Vec<ActiveProposal>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ActiveProposal {
    pub id: u32,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub options: Vec<ActiveOption>,
    #[serde(default)]
    pub zip_number: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ActiveOption {
    /// Valar/protobuf JSON omits default `0`, so the first option often has no `index` field.
    #[serde(default)]
    pub index: u32,
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// Probe vote servers for the active round (status == 1).
pub fn fetch_active_round(vote_server_urls: &[String]) -> Result<ActiveRound> {
    let mut last_err = None;
    for base in vote_server_urls {
        let url = format!(
            "{}/shielded-vote/v1/rounds/active",
            base.trim_end_matches('/')
        );
        match http_get(&url) {
            Ok(bytes) => {
                let parsed: ActiveRoundResponse = serde_json::from_slice(&bytes)
                    .with_context(|| format!("decode active round from {url}"))?;
                match parsed.round {
                    Some(r) if r.status == 1 => return Ok(r),
                    Some(r) => {
                        last_err = Some(anyhow!(
                            "{url}: round present but status={} (want ACTIVE=1)",
                            r.status
                        ));
                    }
                    None => {
                        last_err = Some(anyhow!("{url}: no active round"));
                    }
                }
            }
            Err(e) => last_err = Some(e),
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow!("no vote servers configured")))
}

pub fn vote_round_id_hex(active: &ActiveRound) -> Result<String> {
    // Chain may send base64 or already-hex; accept both.
    if active.vote_round_id.len() == 64
        && active
            .vote_round_id
            .chars()
            .all(|c| c.is_ascii_hexdigit())
    {
        return Ok(active.vote_round_id.to_ascii_lowercase());
    }
    let raw = B64
        .decode(active.vote_round_id.as_bytes())
        .or_else(|_| hex::decode(&active.vote_round_id))
        .context("decode vote_round_id (expected base64 or hex)")?;
    Ok(hex::encode(raw))
}

pub fn b64_field(label: &str, value: &str) -> Result<Vec<u8>> {
    B64.decode(value.as_bytes())
        .with_context(|| format!("decode base64 field {label}"))
}

pub fn active_to_round_params(active: &ActiveRound) -> Result<VotingRoundParams> {
    Ok(VotingRoundParams {
        vote_round_id: vote_round_id_hex(active)?,
        snapshot_height: active.snapshot_height,
        ea_pk: b64_field("ea_pk", &active.ea_pk)?,
        nc_root: b64_field("nc_root", &active.nc_root)?,
        nullifier_imt_root: b64_field("nullifier_imt_root", &active.nullifier_imt_root)?,
    })
}

pub fn init_or_load_hotkey(data_dir: &Path, network: NetworkKind) -> Result<VotingHotkey> {
    let path = hotkey_path(data_dir, network);
    if path.exists() {
        let secret = fs::read(&path).with_context(|| format!("read {}", path.display()))?;
        return VotingHotkey::from_stored_secret(&secret, network.to_sdk())
            .map_err(|e| anyhow!("restore hotkey: {e}"));
    }
    let hotkey = generate_random_voting_hotkey(network.to_sdk())
        .map_err(|e| anyhow!("generate hotkey: {e}"))?;
    fs::write(&path, hotkey.stored_secret()).with_context(|| format!("write {}", path.display()))?;
    // Restrictive perms where possible (best-effort on Windows).
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
    }
    Ok(hotkey)
}

pub fn open_voting_db(data_dir: &Path, wallet_id: &str) -> Result<VotingDb> {
    let path = voting_db_path(data_dir);
    let db = VotingDb::open(path.to_str().context("voting db path utf-8")?)
        .map_err(|e| anyhow!("open voting db: {e}"))?;
    db.set_wallet_id(wallet_id);
    Ok(db)
}

pub fn create_round_from_active(
    db: &VotingDb,
    network: NetworkKind,
    active: &ActiveRound,
) -> Result<VotingRoundParams> {
    let params = active_to_round_params(active)?;
    let session = serde_json::json!({
        "title": active.title,
        "description": active.description,
        "vote_end_time": active.vote_end_time,
    });
    let session_s = session.to_string();
    match db.create_round(network.to_sdk(), &params, Some(&session_s)) {
        Ok(()) => Ok(params),
        Err(e) => {
            // Idempotent enough for CLI: if round exists, continue with params.
            let msg = e.to_string();
            if msg.contains("UNIQUE") || msg.to_lowercase().contains("already") {
                Ok(params)
            } else {
                Err(anyhow!("create_round: {e}"))
            }
        }
    }
}

/// JSON note export produced by `nozy vote-export-notes`.
#[derive(Debug, Deserialize, Serialize)]
pub struct NoteExportFile {
    pub format: String,
    pub network: String,
    pub ufvk: String,
    pub orchard_fvk_hex: String,
    pub seed_fingerprint_hex: String,
    pub account_index: u32,
    pub notes: Vec<ExportedNote>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExportedNote {
    pub commitment_hex: String,
    pub nullifier_hex: String,
    pub value: u64,
    pub position: u64,
    pub diversifier_hex: String,
    pub rho_hex: String,
    pub rseed_hex: String,
    pub scope: u32,
    pub root_hex: String,
    pub auth_path_hex: Vec<String>,
}

pub fn load_note_export(path: &Path) -> Result<NoteExportFile> {
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let file: NoteExportFile = serde_json::from_slice(&bytes)?;
    if file.format != "nozy-vote-notes-v1" {
        bail!("unsupported note export format {:?}", file.format);
    }
    Ok(file)
}

pub fn exported_to_note_infos(file: &NoteExportFile) -> Result<Vec<NoteInfo>> {
    let mut out = Vec::with_capacity(file.notes.len());
    for n in &file.notes {
        out.push(NoteInfo {
            commitment: hex::decode(&n.commitment_hex)?,
            nullifier: hex::decode(&n.nullifier_hex)?,
            value: n.value,
            position: n.position,
            diversifier: hex::decode(&n.diversifier_hex)?,
            rho: hex::decode(&n.rho_hex)?,
            rseed: hex::decode(&n.rseed_hex)?,
            scope: n.scope,
            ufvk_str: file.ufvk.clone(),
        });
    }
    Ok(out)
}

pub fn exported_to_witnesses(file: &NoteExportFile) -> Result<Vec<WitnessData>> {
    let mut out = Vec::with_capacity(file.notes.len());
    for n in &file.notes {
        let auth_path: Result<Vec<Vec<u8>>, _> =
            n.auth_path_hex.iter().map(|h| hex::decode(h)).collect();
        let auth_path = auth_path?;
        if auth_path.len() != 32 {
            bail!(
                "note position {} auth_path length {} (want 32)",
                n.position,
                auth_path.len()
            );
        }
        out.push(WitnessData {
            note_commitment: hex::decode(&n.commitment_hex)?,
            position: n.position,
            root: hex::decode(&n.root_hex)?,
            auth_path,
        });
    }
    Ok(out)
}

pub fn ensure_bundles_and_witnesses(
    db: &VotingDb,
    round_id: &str,
    notes: &[NoteInfo],
    witnesses: &[WitnessData],
) -> Result<BundleLayout> {
    let layout = db
        .ensure_bundles(round_id, notes)
        .map_err(|e| anyhow!("ensure_bundles: {e}"))?;
    if layout.bundle_count == 0 || layout.eligible_weight == 0 {
        let total: u64 = notes.iter().map(|n| n.value).sum();
        // One ballot = BALLOT_DIVISOR (12_500_000 zat = 0.125 ZEC) per zcash_voting governance.
        bail!(
            "exported Ironwood notes total {total} zat ({:.8} ZEC), but NU7 voting requires at least \
             12_500_000 zat (0.125 ZEC) in one bundle. Bundles below that threshold are dropped \
             (see zcash_voting BALLOT_DIVISOR). Add more Ironwood balance held at the snapshot, \
             then re-export.",
            total as f64 / 100_000_000.0
        );
    }
    // Bundle 0 for MVP single-bundle wallets.
    db.store_witnesses(round_id, 0, witnesses)
        .map_err(|e| anyhow!("store_witnesses: {e}"))?;
    Ok(layout)
}

pub fn resolve_vote_server_urls(
    env: Environment,
    static_source: Option<&str>,
) -> Result<(ResolvedVotingConfig, Vec<String>)> {
    let source = urls::static_source(env, static_source);
    let resolved = resolve_config_sdk(&source)?;
    let urls: Vec<String> = resolved
        .vote_servers
        .iter()
        .map(|s| s.url.clone())
        .collect();
    if urls.is_empty() {
        bail!("resolved config has no vote_servers");
    }
    Ok((resolved, urls))
}

#[derive(Debug, Serialize)]
pub struct SigningRequestFile {
    pub format: String,
    pub account_index: u32,
    pub network: String,
    pub seed_fingerprint_hex: String,
    pub sighash_hex: String,
    pub alpha_hex: String,
    pub round_id: String,
    pub bundle_index: u32,
}

pub const SIGNING_REQUEST_FORMAT: &str = "nozy-vote-delegation-sign-v1";
pub const SIG_FILE_FORMAT: &str = "nozy-vote-delegation-sig-v1";

#[derive(Debug, Deserialize)]
pub struct SigFile {
    pub format: String,
    pub round_id: String,
    pub bundle_index: u32,
    pub sighash_hex: String,
    pub spend_auth_sig_hex: String,
}

pub fn write_signing_request(
    path: &Path,
    network: NetworkKind,
    round_id: &str,
    bundle_index: u32,
    req: &zcash_voting::prelude::DelegationSigningRequest,
) -> Result<()> {
    let network_s = match req.network {
        Network::Mainnet => "mainnet",
        Network::Testnet => "testnet",
        Network::Regtest => "regtest",
    };
    // Sanity: CLI network flag should match request.
    if network.to_sdk() != req.network {
        bail!(
            "signing request network {:?} != CLI network {}",
            req.network,
            network.as_str()
        );
    }
    let file = SigningRequestFile {
        format: SIGNING_REQUEST_FORMAT.into(),
        account_index: req.account_index,
        network: network_s.into(),
        seed_fingerprint_hex: hex::encode(req.seed_fingerprint),
        sighash_hex: hex::encode(req.sighash),
        alpha_hex: hex::encode(req.alpha),
        round_id: round_id.to_string(),
        bundle_index,
    };
    fs::write(path, serde_json::to_vec_pretty(&file)?)
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub fn load_sig_file(path: &Path) -> Result<SigFile> {
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let file: SigFile = serde_json::from_slice(&bytes)?;
    if file.format != SIG_FILE_FORMAT {
        bail!("unsupported sig format {:?} (want {SIG_FILE_FORMAT})", file.format);
    }
    Ok(file)
}

pub fn connect_pir_client(
    resolved: &ResolvedVotingConfig,
) -> Result<PirClientBlocking> {
    use std::sync::Arc;
    use zcash_voting::prelude::connect_pir_blocking;
    use zcash_voting::HyperTransport;

    let pir_url = resolved
        .pir_endpoints
        .first()
        .map(|e| e.url.as_str())
        .context("resolved config has no pir_endpoints")?;
    connect_pir_blocking(
        resolved.pir_layout,
        pir_url,
        Arc::new(HyperTransport::new()),
    )
    .map_err(|e| anyhow!("connect PIR: {e}"))
}

/// PIR warm-up + ZKP1 prove for bundle 0.
pub fn pir_and_prove(
    db: &VotingDb,
    round_id: &str,
    notes: &[NoteInfo],
    keys: &zcash_voting::prelude::DelegationKeys,
    pir: &PirClientBlocking,
    network: NetworkKind,
) -> Result<()> {
    use zcash_voting::prelude::NoopProgressReporter;
    use zcash_voting::precompute::delegation_pir;

    db.ensure_padded_secrets(round_id, 0, notes)
        .map_err(|e| anyhow!("ensure_padded_secrets: {e}"))?;
    let report = delegation_pir(db, round_id, 0, notes, pir, network.to_sdk())
        .map_err(|e| anyhow!("delegation PIR precompute: {e}"))?;
    eprintln!(
        "PIR precompute: cached={}, fetched={}",
        report.cached, report.fetched
    );

    let progress = NoopProgressReporter;
    zcash_voting::delegate::prove(db, round_id, 0, notes, keys, pir, &progress)
        .map_err(|e| anyhow!("delegation prove (ZKP1): {e}"))?;
    Ok(())
}

#[derive(Debug, Deserialize)]
struct DelegateVoteResponse {
    tx_hash: String,
    #[serde(default)]
    code: i32,
    #[serde(default)]
    log: String,
}

/// Assemble submission + POST /delegate-vote + record tx hash.
pub fn submit_delegation(
    db: &VotingDb,
    round_id: &str,
    vote_server_urls: &[String],
    sig: &SigFile,
) -> Result<String> {
    use zcash_voting::prelude::{
        delegation_submission, record_submission, DelegationSigner,
    };

    let sighash: [u8; 32] = hex::decode(sig.sighash_hex.trim())?
        .try_into()
        .map_err(|_| anyhow!("sighash must be 32 bytes"))?;
    let spend_sig: [u8; 64] = hex::decode(sig.spend_auth_sig_hex.trim())?
        .try_into()
        .map_err(|_| anyhow!("spend_auth_sig must be 64 bytes"))?;

    let submission = delegation_submission(
        db,
        round_id,
        sig.bundle_index,
        DelegationSigner::signature(spend_sig, sighash),
    )
    .map_err(|e| anyhow!("assemble delegation submission: {e}"))?;

    let body = submission
        .to_wire_json()
        .map_err(|e| anyhow!("delegation wire json: {e}"))?;

    let client = reqwest::blocking::Client::builder()
        .user_agent(format!("nozy-vote/{}", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(180))
        .build()?;

    let mut last_err = None;
    for base in vote_server_urls {
        let url = format!(
            "{}/shielded-vote/v1/delegate-vote",
            base.trim_end_matches('/')
        );
        match client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(body.clone())
            .send()
        {
            Ok(resp) => {
                let status = resp.status();
                let text = resp.text().unwrap_or_default();
                if !status.is_success() {
                    last_err = Some(anyhow!("POST {url} → HTTP {status}: {text}"));
                    continue;
                }
                let parsed: DelegateVoteResponse = serde_json::from_str(&text)
                    .with_context(|| format!("decode delegate-vote response: {text}"))?;
                if parsed.code != 0 {
                    last_err = Some(anyhow!(
                        "delegate-vote code={} log={} tx={}",
                        parsed.code,
                        parsed.log,
                        parsed.tx_hash
                    ));
                    continue;
                }
                record_submission(db, round_id, sig.bundle_index, &parsed.tx_hash)
                    .map_err(|e| anyhow!("record_submission: {e}"))?;
                return Ok(parsed.tx_hash);
            }
            Err(e) => last_err = Some(anyhow!("POST {url}: {e}")),
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow!("all vote servers failed")))
}

/// Poll GET /tx/{hash} until confirmed or timeout.
pub fn wait_tx_confirm(vote_server_urls: &[String], tx_hash: &str) -> Result<serde_json::Value> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(format!("nozy-vote/{}", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    for attempt in 1..=60 {
        for base in vote_server_urls {
            let url = format!(
                "{}/shielded-vote/v1/tx/{}",
                base.trim_end_matches('/'),
                tx_hash
            );
            match client.get(&url).send() {
                Ok(resp) if resp.status().as_u16() == 404 => {}
                Ok(resp) if resp.status().is_success() => {
                    let v: serde_json::Value = resp.json()?;
                    let height = v.get("height").and_then(|h| {
                        h.as_str()
                            .map(|s| !s.is_empty())
                            .or_else(|| h.as_u64().map(|n| n > 0))
                    });
                    let code = v.get("code").and_then(|c| c.as_i64()).unwrap_or(-1);
                    if height == Some(true) && code == 0 {
                        return Ok(v);
                    }
                    if code != 0 && code != -1 {
                        bail!("tx {tx_hash} failed validation: {v}");
                    }
                }
                Ok(resp) => {
                    eprintln!("poll {url} → HTTP {}", resp.status());
                }
                Err(e) => eprintln!("poll {url}: {e}"),
            }
        }
        eprintln!("waiting for tx confirm… ({attempt}/60)");
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
    bail!("timed out waiting for tx {tx_hash} confirmation")
}

pub fn tx_events_from_response(v: &serde_json::Value) -> Result<Vec<zcash_voting::prelude::TxEvent>> {
    let events = v
        .get("events")
        .cloned()
        .unwrap_or_else(|| serde_json::json!([]));
    serde_json::from_value(events).context("decode tx events")
}

pub fn confirm_delegation_from_tx(
    db: &VotingDb,
    round_id: &str,
    bundle_index: u32,
    tx_hash: &str,
    tx_json: &serde_json::Value,
) -> Result<zcash_voting::prelude::DelegationConfirmation> {
    use zcash_voting::prelude::confirm_delegation_submission;
    let events = tx_events_from_response(tx_json)?;
    confirm_delegation_submission(db, round_id, bundle_index, tx_hash, &events)
        .map_err(|e| anyhow!("confirm_delegation_submission: {e}"))
}

pub fn confirm_vote_from_tx(
    db: &VotingDb,
    round_id: &str,
    bundle_index: u32,
    proposal_id: u32,
    tx_hash: &str,
    tx_json: &serde_json::Value,
) -> Result<zcash_voting::prelude::VoteConfirmation> {
    use zcash_voting::prelude::confirm_vote_submission;
    let events = tx_events_from_response(tx_json)?;
    confirm_vote_submission(db, round_id, bundle_index, proposal_id, tx_hash, &events)
        .map_err(|e| anyhow!("confirm_vote_submission: {e}"))
}

/// Sync vote tree + derive VAN witness for bundle 0.
pub fn derive_van_witness(
    db: &VotingDb,
    round_id: &str,
    bundle_index: u32,
    vote_server_url: &str,
) -> Result<zcash_voting::prelude::VanWitness> {
    use zcash_voting::prelude::{sync_vote_tree, van_witness};
    let anchor_height = sync_vote_tree(db, round_id, vote_server_url)
        .map_err(|e| anyhow!("sync_vote_tree: {e}"))?;
    van_witness(db, round_id, bundle_index, anchor_height)
        .map_err(|e| anyhow!("van_witness: {e}"))
}

/// Parse `proposal_id=choice` pairs using ACTIVE round option counts.
pub fn drafts_from_choices(
    active: &ActiveRound,
    choices: &[String],
    single_share: bool,
) -> Result<Vec<zcash_voting::prelude::DraftVote>> {
    use zcash_voting::prelude::DraftVote;
    if choices.is_empty() {
        bail!("pass at least one --choices proposal_id=option_index");
    }
    let mut drafts = Vec::new();
    for raw in choices {
        let (pid_s, choice_s) = raw
            .split_once('=')
            .ok_or_else(|| anyhow!("bad choice {raw:?}; want proposal_id=option_index"))?;
        let proposal_id: u32 = pid_s.trim().parse().context("parse proposal_id")?;
        let choice: u32 = choice_s.trim().parse().context("parse option_index")?;
        let proposal = active
            .proposals
            .iter()
            .find(|p| p.id == proposal_id)
            .ok_or_else(|| anyhow!("proposal_id {proposal_id} not in ACTIVE round"))?;
        let num_options = u32::try_from(proposal.options.len())
            .context("too many options")?;
        if choice >= num_options {
            bail!(
                "choice {choice} out of range for Q{proposal_id} (0..{})",
                num_options.saturating_sub(1)
            );
        }
        drafts.push(DraftVote {
            proposal_id,
            choice,
            num_options,
            vc_tree_position: 0, // updated after cast-vote confirms
            single_share,
        });
    }
    drafts.sort_by_key(|d| d.proposal_id);
    Ok(drafts)
}

#[derive(Debug, Deserialize)]
struct CastVoteResponse {
    tx_hash: String,
    #[serde(default)]
    code: i32,
    #[serde(default)]
    log: String,
}

pub fn submit_cast_vote(vote_server_urls: &[String], body: &str) -> Result<String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(format!("nozy-vote/{}", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(180))
        .build()?;
    let mut last_err = None;
    for base in vote_server_urls {
        let url = format!(
            "{}/shielded-vote/v1/cast-vote",
            base.trim_end_matches('/')
        );
        match client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send()
        {
            Ok(resp) => {
                let status = resp.status();
                let text = resp.text().unwrap_or_default();
                if !status.is_success() {
                    last_err = Some(anyhow!("POST {url} → HTTP {status}: {text}"));
                    continue;
                }
                let parsed: CastVoteResponse = serde_json::from_str(&text)
                    .with_context(|| format!("decode cast-vote response: {text}"))?;
                if parsed.code != 0 {
                    last_err = Some(anyhow!(
                        "cast-vote code={} log={} tx={}",
                        parsed.code,
                        parsed.log,
                        parsed.tx_hash
                    ));
                    continue;
                }
                return Ok(parsed.tx_hash);
            }
            Err(e) => last_err = Some(anyhow!("POST {url}: {e}")),
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow!("all vote servers failed for cast-vote")))
}

/// POST helper shares (immediate submit_at=0 for CLI MVP).
pub fn submit_helper_shares(
    vote_server_urls: &[String],
    committed: &zcash_voting::prelude::CommittedVote,
    db: &VotingDb,
    vc_tree_position: u64,
) -> Result<usize> {
    use rand::seq::SliceRandom;

    let payloads = committed.share_payloads();
    if payloads.is_empty() {
        bail!("committed vote has no share payloads");
    }
    let client = reqwest::blocking::Client::builder()
        .user_agent(format!("nozy-vote/{}", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let mut rng = rand::thread_rng();
    let mut servers = vote_server_urls.to_vec();
    servers.shuffle(&mut rng);

    let mut sent = 0usize;
    for (i, payload) in payloads.iter().enumerate() {
        let body = payload
            .to_wire_json(Some(vc_tree_position), 0)
            .map_err(|e| anyhow!("share wire json: {e}"))?;
        let base = &servers[i % servers.len()];
        let url = format!("{}/shielded-vote/v1/shares", base.trim_end_matches('/'));
        let mut ok = false;
        let mut last_err = None;
        for attempt in 1..=3 {
            match client
                .post(&url)
                .header("Content-Type", "application/json")
                .body(body.clone())
                .send()
            {
                Ok(resp) if resp.status().is_success() => {
                    ok = true;
                    break;
                }
                Ok(resp) => {
                    last_err = Some(anyhow!(
                        "share POST {url} → HTTP {} (attempt {attempt})",
                        resp.status()
                    ));
                }
                Err(e) => last_err = Some(anyhow!("share POST {url}: {e}")),
            }
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
        if !ok {
            return Err(last_err.unwrap_or_else(|| anyhow!("share submit failed")));
        }
        let share_index = payload.enc_share.share_index;
        committed
            .record_share(db, share_index, &[base.clone()], 0)
            .map_err(|e| anyhow!("record_share: {e}"))?;
        sent += 1;
    }
    Ok(sent)
}

/// Cast all drafts for bundle 0 (serialized per-proposal).
pub fn cast_ballots(
    db: &VotingDb,
    active: &ActiveRound,
    vote_server_urls: &[String],
    hotkey: &VotingHotkey,
    drafts: &[zcash_voting::prelude::DraftVote],
    no_wait: bool,
) -> Result<()> {
    use zcash_voting::prelude::{
        CommittedVote, NoopProgressReporter, VoteSigner,
    };

    let rid = vote_round_id_hex(active)?;
    let server0 = vote_server_urls
        .first()
        .context("no vote servers")?
        .as_str();

    println!("syncing vote tree + deriving VAN witness…");
    let mut van = derive_van_witness(db, &rid, 0, server0)?;

    for draft in drafts {
        println!(
            "casting Q{} choice {} ({} options)…",
            draft.proposal_id, draft.choice, draft.num_options
        );
        let progress = NoopProgressReporter;
        let committed = CommittedVote::commit(
            db,
            &rid,
            0,
            draft,
            &van,
            VoteSigner::hotkey(hotkey),
            &progress,
        )
        .map_err(|e| anyhow!("vote commit Q{}: {e}", draft.proposal_id))?;

        let body = committed
            .signed_commitment(db)
            .map_err(|e| anyhow!("signed commitment: {e}"))?
            .to_wire_json()
            .map_err(|e| anyhow!("cast-vote wire json: {e}"))?;
        let tx_hash = submit_cast_vote(vote_server_urls, &body)?;
        committed
            .record_submission(db, &tx_hash)
            .map_err(|e| anyhow!("record vote tx: {e}"))?;
        println!("  submitted cast-vote tx {tx_hash}");

        if no_wait {
            println!("  skipped wait/shares (--no-wait); resume later");
            continue;
        }

        let conf_json = wait_tx_confirm(vote_server_urls, &tx_hash)?;
        let conf = confirm_vote_from_tx(db, &rid, 0, draft.proposal_id, &tx_hash, &conf_json)?;
        println!(
            "  confirmed VAN leaf {} / VC position {}",
            conf.van_leaf_position, conf.vc_tree_position
        );
        committed
            .record_vc_position(db, conf.vc_tree_position)
            .map_err(|e| anyhow!("record_vc_position: {e}"))?;

        let n = submit_helper_shares(vote_server_urls, &committed, db, conf.vc_tree_position)?;
        println!("  queued {n} helper share(s)");

        // Next proposal consumes the new VAN from this cast.
        van = derive_van_witness(db, &rid, 0, server0)?;
    }
    Ok(())
}
