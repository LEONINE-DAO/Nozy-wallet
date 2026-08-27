//! NU7 coinholder vote calendar (forum thread #56912, updated end date Sep 14).

use chrono::{DateTime, Utc};

/// Spendable Ironwood snapshot (UTC).
pub const SNAPSHOT_UTC: &str = "2026-08-24T19:00:00Z";
/// Voting opens (UTC).
pub const VOTE_START_UTC: &str = "2026-08-25T00:00:00Z";
/// Voting closes (UTC) — aligned with ZCAP per forum update.
pub const VOTE_END_UTC: &str = "2026-09-14T19:00:00Z";
pub const FORUM_URL: &str = "https://forum.zcashcommunity.com/t/nu7-coinholder-vote/56912";
pub const TALLY_URL: &str = "https://tally.valargroup.org";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VotePhase {
    PreSnapshot,
    PreOpen,
    Open,
    Closed,
}

fn parse_utc(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .expect("compile-time calendar constant")
        .with_timezone(&Utc)
}

pub fn current_phase() -> VotePhase {
    let now = Utc::now();
    let snapshot = parse_utc(SNAPSHOT_UTC);
    let start = parse_utc(VOTE_START_UTC);
    let end = parse_utc(VOTE_END_UTC);
    if now < snapshot {
        VotePhase::PreSnapshot
    } else if now < start {
        VotePhase::PreOpen
    } else if now <= end {
        VotePhase::Open
    } else {
        VotePhase::Closed
    }
}

pub fn phase_message(phase: VotePhase) -> String {
    let snapshot = parse_utc(SNAPSHOT_UTC);
    match phase {
        VotePhase::PreSnapshot => {
            let hours = (snapshot - Utc::now()).num_hours();
            format!(
                "Migrate Orchard → Ironwood before snapshot (~{hours}h left). Weight freezes at snapshot."
            )
        }
        VotePhase::PreOpen => {
            "Snapshot passed. Voting opens soon — ensure Ironwood notes were held at snapshot."
                .into()
        }
        VotePhase::Open => "Voting is OPEN. Delegate, then cast your ballot.".into(),
        VotePhase::Closed => format!("Voting window has ended. Tallies: {TALLY_URL}"),
    }
}

pub fn print_eligibility_banner() {
    println!("=== NozyWallet — NU7 coinholder vote ===");
    println!("eligibility: spendable Ironwood notes at snapshot");
    println!("snapshot:    {SNAPSHOT_UTC}");
    println!("vote window: {VOTE_START_UTC} → {VOTE_END_UTC}");
    println!("poll by:     Valar Group / Project Tachyon (forum thread)");
    println!("forum:       {FORUM_URL}");
    println!();
    println!("{}", phase_message(current_phase()));
}
