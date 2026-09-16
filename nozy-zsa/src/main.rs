//! Isolated OrchardZSA CLI — does not link Ironwood orchard 0.15.
//!
//! Sprint 1: prove QEDIT deps resolve; then issue/transfer against zsa.methyl.cc.

use clap::{Parser, Subcommand};

const DEFAULT_LWD: &str = "https://zsa.methyl.cc";
const FAUCET: &str = "https://faucet.zsa.methyl.cc";

#[derive(Parser)]
#[command(name = "nozy-zsa", about = "OrchardZSA toolkit (isolated from Ironwood Nozy)")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print pinned endpoints and confirm OrchardZSA types link
    Status {
        #[arg(long)]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Status { json } => {
            // Touch OrchardZSA surface so missing features fail at compile time.
            let _ = orchard::note::AssetBase::zatoshi();
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "tool": "nozy-zsa",
                        "lwd": DEFAULT_LWD,
                        "faucet": FAUCET,
                        "orchard_zsa": "linked",
                        "asset_base": "zatoshi",
                        "note": "issue/transfer not wired yet — deps resolve"
                    })
                );
            } else {
                println!("nozy-zsa status");
                println!("   LWD:         {DEFAULT_LWD}");
                println!("   Faucet:      {FAUCET}");
                println!("   OrchardZSA:  linked (AssetBase::zatoshi ok)");
                println!("   Next:        issue / transfer / compact scan");
            }
        }
    }
}
