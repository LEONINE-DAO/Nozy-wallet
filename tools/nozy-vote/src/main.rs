//! NozyWallet coinholder vote helper (NU7 / Valar Shielded Vote).
//!
//! Tracking: https://github.com/LEONINE-DAO/Nozy-wallet/issues/273

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use nozy_vote::config::Environment;
use nozy_vote::eligibility::print_eligibility_banner;
use nozy_vote::flow;
use nozy_vote::{sdk, urls};

#[derive(Debug, Parser)]
#[command(
    name = "nozy-vote",
    about = "NozyWallet helper for Zcash coinholder voting (Valar Shielded Vote)",
    long_about = "CLI path for the NU7 coinholder vote.\n\
Eligible weight = spendable Ironwood notes at the Aug 24 2026 19:00 UTC snapshot.\n\
See README.md and issue #273."
)]
struct Cli {
    #[arg(long, env = "NOZY_VOTE_ENV", default_value = "stage", global = true)]
    env: Environment,

    #[arg(long, env = "NOZY_VOTE_STATIC_SOURCE", global = true)]
    static_source: Option<String>,

    #[arg(long, env = "NOZY_VOTE_DATA_DIR", global = true)]
    data_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Print NU7 eligibility deadlines and local helper readiness.
    Status {
        #[arg(long)]
        json: bool,
    },
    /// Fetch + Ed25519-authenticate voting config.
    Fetch {
        #[arg(long)]
        json: bool,
    },
    /// List authenticated round ids.
    Rounds {
        #[arg(long)]
        json: bool,
    },
    /// Probe vote servers for the currently ACTIVE round + proposals.
    Active {
        #[arg(long)]
        json: bool,
    },
    /// Create or load the app-owned voting hotkey (does not use wallet seed).
    HotkeyInit {
        #[arg(long, default_value = "mainnet")]
        network: String,
    },
    /// Open voting DB and create_round from the active chain round.
    InitRound {
        #[arg(long, default_value = "mainnet")]
        network: String,
        #[arg(long, default_value = "nozy")]
        wallet_id: String,
    },
    /// Import notes from `nozy vote-export-notes` JSON.
    ImportNotes {
        #[arg(long)]
        file: PathBuf,
        #[arg(long, default_value = "mainnet")]
        network: String,
        #[arg(long, default_value = "nozy")]
        wallet_id: String,
    },
    /// Prepare delegation PCZT setup + write signing request for `nozy vote-sign-delegation`.
    Delegate {
        #[arg(long)]
        round_id: Option<String>,
        #[arg(long, default_value = "mainnet")]
        network: String,
        #[arg(long, default_value = "nozy")]
        wallet_id: String,
        #[arg(long)]
        notes_file: PathBuf,
    },
    /// After signing: PIR + prove ZKP1 + submit delegation to the vote chain.
    DelegateFinish {
        #[arg(long, default_value = "mainnet")]
        network: String,
        #[arg(long, default_value = "nozy")]
        wallet_id: String,
        #[arg(long)]
        notes_file: PathBuf,
        #[arg(long, help = "Signature JSON from `nozy vote-sign-delegation`")]
        sig: PathBuf,
        #[arg(long, help = "Skip waiting for on-chain confirmation")]
        no_wait: bool,
    },
    /// Cast ballots after confirmed delegation.
    ///
    /// Choices: `proposal_id=option_index` (repeatable / comma-separated), e.g. `1=0,2=1,3=0`.
    Cast {
        #[arg(long, value_delimiter = ',')]
        choices: Vec<String>,
        #[arg(long, default_value = "mainnet")]
        network: String,
        #[arg(long, default_value = "nozy")]
        wallet_id: String,
        /// Optional: confirm this delegation tx before casting (if VAN not yet recorded).
        #[arg(long)]
        delegation_tx: Option<String>,
        /// Collapse to single-share / immediate submit (recommended near vote end).
        #[arg(long, default_value_t = false)]
        single_share: bool,
        #[arg(long, help = "Skip wait/confirm/shares after each cast-vote submit")]
        no_wait: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let data_dir = flow::ensure_data_dir(cli.data_dir.clone())?;

    match &cli.command {
        Commands::Status { json } => {
            let net = flow::parse_network("mainnet")?;
            let snap = flow::status_snapshot(
                cli.env,
                cli.static_source.as_deref(),
                &data_dir,
                net,
            )?;
            if *json {
                println!("{}", serde_json::to_string_pretty(&snap)?);
            } else {
                print_eligibility_banner();
                println!();
                println!("helper: nozy-vote {}", snap.helper_version);
                println!("env:    {}", snap.env);
                println!("data:   {}", snap.data_dir);
                println!("sdk:    enabled (zcash_voting linked)");
                println!("static: {}", snap.static_source);
            }
            Ok(())
        }
        Commands::Fetch { json } => {
            print_eligibility_banner();
            println!();
            let source = urls::static_source(cli.env, cli.static_source.as_deref());
            let resolved = sdk::resolve_config_sdk(&source)?;
            let cache = data_dir.join("resolved-config.json");
            let round_ids: Vec<String> = resolved
                .authenticated_rounds
                .iter()
                .map(|r| r.round_id.clone())
                .collect();
            let summary = serde_json::json!({
                "static_source": source,
                "source_fingerprint": resolved.source_fingerprint,
                "vote_servers": resolved.vote_servers.iter().map(|s| &s.url).collect::<Vec<_>>(),
                "pir_endpoints": resolved.pir_endpoints.iter().map(|s| &s.url).collect::<Vec<_>>(),
                "round_ids": round_ids,
            });
            std::fs::write(&cache, serde_json::to_vec_pretty(&summary)?)?;
            if *json {
                println!("{}", serde_json::to_string_pretty(&summary)?);
            } else {
                println!("ok — config authenticated");
                println!("  cached: {}", cache.display());
                println!("  rounds: {}", resolved.authenticated_rounds.len());
            }
            Ok(())
        }
        Commands::Rounds { json } => {
            let source = urls::static_source(cli.env, cli.static_source.as_deref());
            let resolved = sdk::resolve_config_sdk(&source)?;
            let round_ids: Vec<String> = resolved
                .authenticated_rounds
                .iter()
                .map(|r| r.round_id.clone())
                .collect();
            if *json {
                println!("{}", serde_json::to_string_pretty(&round_ids)?);
            } else {
                for id in &round_ids {
                    println!("{id}");
                }
            }
            Ok(())
        }
        Commands::Active { json } => {
            let active = flow::fetch_active(cli.env, cli.static_source.as_deref())?;
            if *json {
                println!("{}", serde_json::to_string_pretty(&active)?);
            } else {
                println!("active round {}", active.vote_round_id);
                println!("  snapshot: {}", active.snapshot_height);
                println!("  status:   {}", active.status);
                if let Some(t) = &active.title {
                    println!("  title:    {t}");
                }
                for p in &active.proposals {
                    println!("  Q{}: {}", p.id, p.title);
                    for o in &p.options {
                        println!("    [{}] {}", o.index, o.label);
                    }
                }
            }
            Ok(())
        }
        Commands::HotkeyInit { network } => {
            let net = flow::parse_network(network)?;
            let hk = sdk::init_or_load_hotkey(&data_dir, net)?;
            println!(
                "ok — voting hotkey ready ({} bytes secret at {})",
                hk.stored_secret().len(),
                sdk::hotkey_path(&data_dir, net).display()
            );
            println!("note: hotkey is app-owned randomness — not derived from your wallet seed.");
            Ok(())
        }
        Commands::InitRound {
            network,
            wallet_id,
        } => {
            let net = flow::parse_network(network)?;
            let (_resolved, servers) =
                sdk::resolve_vote_server_urls(cli.env, cli.static_source.as_deref())?;
            let active = sdk::fetch_active_round(&servers)?;
            let db = sdk::open_voting_db(&data_dir, wallet_id)?;
            let params = sdk::create_round_from_active(&db, net, &active)?;
            println!("ok — voting round ready");
            println!("  db:       {}", sdk::voting_db_path(&data_dir).display());
            println!("  round_id: {}", params.vote_round_id);
            println!("  snapshot: {}", params.snapshot_height);
            Ok(())
        }
        Commands::ImportNotes {
            file,
            network,
            wallet_id,
        } => {
            let net = flow::parse_network(network)?;
            let result = flow::prepare_round(
                cli.env,
                cli.static_source.as_deref(),
                &data_dir,
                net,
                wallet_id,
                file,
            )?;
            println!(
                "ok — imported {} note(s) into round {}",
                result.note_count, result.round_id
            );
            println!("  bundles: {}", result.bundle_count);
            println!("  weight:  {} zat", result.eligible_weight_zat);
            println!("  next: nozy-vote delegate --notes-file {}", file.display());
            Ok(())
        }
        Commands::Delegate {
            round_id,
            network,
            wallet_id,
            notes_file,
        } => {
            let net = flow::parse_network(network)?;
            let result = flow::prepare_delegation(
                cli.env,
                cli.static_source.as_deref(),
                &data_dir,
                net,
                wallet_id,
                notes_file,
                round_id.as_deref(),
            )?;
            println!("ok — delegation PCZT setup for round {}", result.round_id);
            println!("  notes:    {}", result.note_count);
            println!("  bundles:  {}", result.bundle_count);
            println!("  weight:   {} zat", result.eligible_weight_zat);
            println!("  sign req: {}", result.signing_request_path);
            println!();
            println!("next:");
            println!(
                "  nozy vote-sign-delegation --request {}",
                result.signing_request_path
            );
            println!(
                "  nozy-vote --env {:?} delegate-finish --notes-file {} --sig <delegation-sig.json>",
                cli.env,
                notes_file.display()
            );
            Ok(())
        }
        Commands::DelegateFinish {
            network,
            wallet_id,
            notes_file,
            sig,
            no_wait,
        } => {
            let net = flow::parse_network(network)?;
            println!("connecting PIR + proving ZKP1 (this can take a while)…");
            let result = flow::finish_delegation(
                cli.env,
                cli.static_source.as_deref(),
                &data_dir,
                net,
                wallet_id,
                notes_file,
                sig,
                !*no_wait,
            )?;
            println!("ok — submitted delegation tx {}", result.tx_hash);
            if result.confirmed {
                if let Some(pos) = result.van_leaf_position {
                    println!("confirmed VAN leaf {pos}");
                }
                println!();
                println!("next: nozy-vote cast --choices 1=0,2=1,...  (see `active` for options)");
            } else if *no_wait {
                println!(
                    "skipped wait; poll later with vote-server /tx/{}",
                    result.tx_hash
                );
            } else {
                println!("submitted; confirmation parse incomplete — check vote servers");
            }
            Ok(())
        }
        Commands::Cast {
            choices,
            network,
            wallet_id,
            delegation_tx,
            single_share,
            no_wait,
        } => {
            let net = flow::parse_network(network)?;
            let result = flow::cast_votes(
                cli.env,
                cli.static_source.as_deref(),
                &data_dir,
                net,
                wallet_id,
                choices,
                delegation_tx.as_deref(),
                *single_share,
                !*no_wait,
            )?;
            println!(
                "ok — cast complete for {} proposal(s) (round {})",
                result.proposal_count, result.round_id
            );
            Ok(())
        }
    }
}
