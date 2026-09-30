//! Payout claim pack — UFVK + mainnet Orchard address for Shielded Labs scans.
//!
//! Round 1 failures on the feature-net thread were mostly the wrong viewing key
//! (desktop node vs mobile delegator) or confusing bonded principal with earned
//! cTAZ. This pack is the paste body for that submission window.

use super::{short_hex, zat_to_ctaz};
use crate::error::{NozyError, NozyResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default)]
pub struct PayoutClaimInput {
    pub feature_net_ufvk: String,
    pub mainnet_orchard: Option<String>,
    pub mobile_ufvk: Option<String>,
    pub height: u32,
    pub cutoff_height: Option<u32>,
    pub earned_zat: u64,
    pub bonded_zat: u64,
    pub active_bonds: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutClaimPack {
    pub feature_net_ufvk: String,
    pub ufvk_fingerprint: String,
    pub mainnet_orchard: Option<String>,
    pub mobile_ufvk: Option<String>,
    pub height: u32,
    pub cutoff_height: Option<u32>,
    pub earned_zat: u64,
    pub bonded_zat: u64,
    pub active_bonds: usize,
    pub complete: bool,
    pub notes: Vec<String>,
    pub paste_body: String,
}

impl PayoutClaimPack {
    pub fn build(input: PayoutClaimInput) -> NozyResult<Self> {
        let feature_net_ufvk = normalize_ufvk(&input.feature_net_ufvk)?;
        let mainnet_orchard = match input.mainnet_orchard {
            Some(addr) if !addr.trim().is_empty() => Some(normalize_mainnet_orchard(&addr)?),
            _ => None,
        };
        let mobile_ufvk = match input.mobile_ufvk {
            Some(u) if !u.trim().is_empty() => {
                let m = normalize_ufvk(&u)?;
                if m == feature_net_ufvk {
                    None
                } else {
                    Some(m)
                }
            }
            _ => None,
        };

        let mut notes = vec![
            "Payouts are pro rata on earned cTAZ from mining/staking, not faucet or bonded principal."
                .into(),
            "Earned here is latest − initial on current bonds — Shielded Labs scans the UFVK for the official period total."
                .into(),
        ];
        if mainnet_orchard.is_none() {
            notes.push(
                "Add a mainnet Orchard unified address (u1…) — that is where ZEC is sent.".into(),
            );
        }
        if mobile_ufvk.is_none() {
            notes.push(
                "If you also staked from a mobile delegator (e.g. ZingoDelegator), include that UFVK too. Round 1 misses were often desktop-only."
                    .into(),
            );
        } else {
            notes.push(
                "Two UFVKs listed — submit both if you used node/GUI and a mobile wallet.".into(),
            );
        }
        if let Some(cut) = input.cutoff_height {
            if cut > input.height {
                notes.push(format!(
                    "Cutoff height {cut} is above current feature-net height {}.",
                    input.height
                ));
            }
        }

        let ufvk_fingerprint = ufvk_fingerprint(&feature_net_ufvk);
        let complete = mainnet_orchard.is_some();
        let mut pack = Self {
            feature_net_ufvk,
            ufvk_fingerprint,
            mainnet_orchard,
            mobile_ufvk,
            height: input.height,
            cutoff_height: input.cutoff_height,
            earned_zat: input.earned_zat,
            bonded_zat: input.bonded_zat,
            active_bonds: input.active_bonds,
            complete,
            notes,
            paste_body: String::new(),
        };
        pack.paste_body = pack.render_paste();
        Ok(pack)
    }

    fn render_paste(&self) -> String {
        let mut lines = vec![
            "Nozy × Crosslink payout claim".into(),
            format!("Feature-net height: {}", self.height),
        ];
        match self.cutoff_height {
            Some(h) => lines.push(format!("Cutoff height: {h}")),
            None => lines
                .push("Cutoff height: (not set — use the height Shielded Labs announced)".into()),
        }
        lines.push(format!(
            "Earned on current bonds (latest − initial): {:.8} cTAZ",
            zat_to_ctaz(self.earned_zat)
        ));
        lines.push(format!(
            "Bonded principal (NOT the payout metric): {:.8} cTAZ",
            zat_to_ctaz(self.bonded_zat)
        ));
        lines.push(format!("Active bonds: {}", self.active_bonds));
        lines.push(String::new());
        lines.push("Feature-net UFVK (desktop / node wallet):".into());
        lines.push(self.feature_net_ufvk.clone());
        lines.push(format!("UFVK fingerprint: {}", self.ufvk_fingerprint));
        lines.push(String::new());
        lines.push("Mainnet Orchard payout address:".into());
        lines.push(
            self.mainnet_orchard
                .clone()
                .unwrap_or_else(|| "(missing — paste a u1… address)".into()),
        );
        if let Some(ref mobile) = self.mobile_ufvk {
            lines.push(String::new());
            lines.push("Second UFVK (mobile / delegator):".into());
            lines.push(mobile.clone());
        }
        lines.push(String::new());
        lines.push("Notes:".into());
        for note in &self.notes {
            lines.push(format!("- {note}"));
        }
        lines.join("\n")
    }
}

pub fn display_payout_claim(pack: &PayoutClaimPack) {
    super::display::print_feature_net_banner();
    println!("{}", pack.paste_body);
}

fn normalize_ufvk(raw: &str) -> NozyResult<String> {
    let s = raw.trim().to_string();
    if s.len() < 16 {
        return Err(NozyError::InvalidInput(
            "Feature-net UFVK is empty or too short — load it from the Crosslink node".into(),
        ));
    }
    Ok(s)
}

fn normalize_mainnet_orchard(raw: &str) -> NozyResult<String> {
    let s = raw.trim().to_string();
    if !looks_like_mainnet_orchard(&s) {
        return Err(NozyError::InvalidInput(
            "Payout address must be a mainnet Orchard unified address (u1…), not feature-net or testnet"
                .into(),
        ));
    }
    Ok(s)
}

pub fn looks_like_mainnet_orchard(addr: &str) -> bool {
    let s = addr.trim();
    s.starts_with("u1") && !s.starts_with("utest") && s.len() >= 78
}

pub fn ufvk_fingerprint(ufvk: &str) -> String {
    short_hex(ufvk.trim(), 16, 8)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> PayoutClaimInput {
        PayoutClaimInput {
            feature_net_ufvk: "uview1abcdefghijklmnopqrstuvwxyz0123456789ABCDEF".into(),
            mainnet_orchard: Some(
                "u1abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789abcd"
                    .into(),
            ),
            mobile_ufvk: None,
            height: 196_617,
            cutoff_height: Some(131_000),
            earned_zat: 18_400_000_000,
            bonded_zat: 380_000_000_000,
            active_bonds: 12,
        }
    }

    #[test]
    fn pack_marks_complete_with_u1_and_ufvk() {
        let pack = PayoutClaimPack::build(input()).unwrap();
        assert!(pack.complete);
        assert!(pack.paste_body.contains("Feature-net UFVK"));
        assert!(pack.paste_body.contains("u1abcdefghijklmnopqrstuvwxyz"));
        assert!(pack.paste_body.contains("184.00000000 cTAZ"));
        assert!(pack.paste_body.contains("NOT the payout metric"));
        assert!(pack.ufvk_fingerprint.contains('…') || pack.ufvk_fingerprint.len() < 40);
    }

    #[test]
    fn rejects_testnet_payout_address() {
        let mut i = input();
        i.mainnet_orchard =
            Some("utest1abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUV".into());
        assert!(PayoutClaimPack::build(i).is_err());
    }

    #[test]
    fn incomplete_without_payout_address() {
        let mut i = input();
        i.mainnet_orchard = None;
        let pack = PayoutClaimPack::build(i).unwrap();
        assert!(!pack.complete);
        assert!(pack.paste_body.contains("missing"));
    }

    #[test]
    fn drops_duplicate_mobile_ufvk() {
        let mut i = input();
        i.mobile_ufvk = Some(i.feature_net_ufvk.clone());
        let pack = PayoutClaimPack::build(i).unwrap();
        assert!(pack.mobile_ufvk.is_none());
    }

    #[test]
    fn keeps_distinct_mobile_ufvk() {
        let mut i = input();
        i.mobile_ufvk = Some("uview1zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz".into());
        let pack = PayoutClaimPack::build(i).unwrap();
        assert!(pack.mobile_ufvk.is_some());
        assert!(pack.paste_body.contains("Second UFVK"));
    }

    #[test]
    fn mainnet_orchard_heuristic() {
        assert!(looks_like_mainnet_orchard(
            "u1abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789abcd"
        ));
        assert!(!looks_like_mainnet_orchard("utest1abc"));
        assert!(!looks_like_mainnet_orchard("t1abc"));
        assert!(!looks_like_mainnet_orchard("u1short"));
    }
}
