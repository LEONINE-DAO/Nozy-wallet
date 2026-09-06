//! Crosslink TFL / staking JSON-RPC helpers.

use super::types::{
    FinalizedTip, GuardianSnapshot, RawStakingPositions, RosterEntry, StakingAction,
    StakingPositions, WalletSyncStatus,
};
use super::{normalize_finalizer_hex, reverse_bond_pk_hex, staking_day_at};
use crate::config::{BackendKind, WalletConfig};
use crate::error::{NozyError, NozyResult};
use crate::zebra_integration::ZebraClient;
use serde_json::Value;

pub fn build_crosslink_client(config: &WalletConfig) -> NozyResult<ZebraClient> {
    let url = if !config.crosslink_url.is_empty() {
        config.crosslink_url.clone()
    } else if matches!(config.backend, BackendKind::Crosslink) {
        config.zebra_url.clone()
    } else {
        return Err(NozyError::Config(
            "Crosslink RPC URL not set. Point Nozy at a Season 1 node:\n  \
             nozy config --use-crosslink --set-crosslink-url http://127.0.0.1:8232\n  \
             (feature-net only — keep mainnet wallets on BackendKind::Zebra)"
                .into(),
        ));
    };

    Ok(ZebraClient::new_with_backend(url, BackendKind::Crosslink))
}

pub async fn is_tfl_activated(client: &ZebraClient) -> NozyResult<Option<bool>> {
    match client
        .call_rpc::<bool>("is_tfl_activated", Value::Array(vec![]))
        .await
    {
        Ok(v) => Ok(Some(v)),
        Err(_) => Ok(None),
    }
}

pub async fn finality_tip(client: &ZebraClient) -> NozyResult<Option<FinalizedTip>> {
    match client
        .call_rpc::<Value>("get_tfl_final_block_height_and_hash", Value::Array(vec![]))
        .await
    {
        Ok(v) => Ok(Some(parse_finalized_tip(&v))),
        Err(_) => Ok(None),
    }
}

fn parse_finalized_tip(v: &Value) -> FinalizedTip {
    let height = v
        .get("height")
        .or_else(|| v.get("block_height"))
        .and_then(|x| x.as_u64())
        .map(|h| h as u32);
    let hash = v
        .get("hash")
        .or_else(|| v.get("block_hash"))
        .and_then(normalize_hash_value);
    FinalizedTip { height, hash }
}

/// Tip hashes may arrive as hex strings or 32-byte JSON arrays (internal order).
pub fn normalize_hash_value(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => {
            let s = s.trim().trim_start_matches("0x").to_ascii_lowercase();
            if s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit()) {
                Some(s)
            } else {
                None
            }
        }
        Value::Array(arr) if arr.len() == 32 => {
            let mut bytes = Vec::with_capacity(32);
            for item in arr {
                let b = item.as_u64().and_then(|n| u8::try_from(n).ok())?;
                bytes.push(b);
            }
            bytes.reverse();
            Some(hex::encode(bytes))
        }
        _ => None,
    }
}

pub async fn staking_positions(client: &ZebraClient) -> NozyResult<StakingPositions> {
    let raw: RawStakingPositions = client
        .call_rpc("wallet_staking_positions", Value::Array(vec![]))
        .await?;
    Ok(raw.into())
}

/// Node wallet unified full viewing key — required for Season 1 ZEC payout submissions.
pub async fn wallet_ufvk(client: &ZebraClient) -> NozyResult<String> {
    let value: Value = client
        .call_rpc("get_wallet_ufvk", Value::Array(vec![]))
        .await?;
    parse_wallet_ufvk(&value).ok_or_else(|| {
        NozyError::InvalidOperation(
            "Crosslink node returned an unexpected get_wallet_ufvk shape".into(),
        )
    })
}

fn parse_wallet_ufvk(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => {
            let key = s.trim();
            if key.is_empty() {
                None
            } else {
                Some(key.to_string())
            }
        }
        Value::Object(obj) => obj
            .get("ufvk")
            .or_else(|| obj.get("ufk"))
            .or_else(|| obj.get("viewing_key"))
            .and_then(parse_wallet_ufvk),
        _ => None,
    }
}

/// Headless wallet spendable balance + staked totals (monolith PR #51 / newer builds).
pub async fn wallet_sync_status(client: &ZebraClient) -> Option<WalletSyncStatus> {
    let value: Value = client
        .call_rpc("get_wallet_sync_status", Value::Array(vec![]))
        .await
        .ok()?;
    parse_wallet_sync_status(&value)
}

fn parse_wallet_sync_status(value: &Value) -> Option<WalletSyncStatus> {
    let obj = value.as_object()?;
    Some(WalletSyncStatus {
        sync_height: obj.get("sync_height").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        tip_height: obj.get("tip_height").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        user_shielded_spendable_zats: zats_field(obj, "user_shielded_spendable_zats").unwrap_or(0),
        user_shielded_pending_zats: zats_field(obj, "user_shielded_pending_zats").unwrap_or(0),
        user_unshielded_zats: zats_field(obj, "user_unshielded_zats").unwrap_or(0),
        staked_zats: zats_field(obj, "staked_zats").unwrap_or(0),
        withdrawable_zats: zats_field(obj, "withdrawable_zats").unwrap_or(0),
    })
}

fn zats_field(obj: &serde_json::Map<String, Value>, key: &str) -> Option<u64> {
    let v = obj.get(key)?;
    v.as_u64()
        .or_else(|| v.as_i64().map(|n| n.max(0) as u64))
        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
}

pub async fn roster(client: &ZebraClient, zats: bool) -> NozyResult<Vec<RosterEntry>> {
    let method = if zats {
        "get_tfl_roster_zats"
    } else {
        "get_tfl_roster_zec"
    };
    let value: Value = client.call_rpc(method, Value::Array(vec![])).await?;
    Ok(parse_roster(&value))
}

fn parse_roster(value: &Value) -> Vec<RosterEntry> {
    let mut entries = Vec::new();

    // Common shapes: { "roster": [ { "id"/"pk"/"finalizer", "stake"/"amount"/... } ] }
    // or a bare array, or a map of finalizer -> stake.
    if let Some(arr) = value
        .get("roster")
        .and_then(|r| r.as_array())
        .or_else(|| value.as_array())
    {
        for item in arr {
            if let Some(e) = roster_entry_from_value(item) {
                entries.push(e);
            }
        }
    } else if let Some(map) = value.as_object() {
        for (k, v) in map {
            if k == "roster" || k == "total" {
                continue;
            }
            let stake = stake_from_value(v).unwrap_or(0);
            entries.push(RosterEntry {
                finalizer: k.clone(),
                stake_zat: stake,
                share: 0.0,
            });
        }
    }

    let total: u64 = entries.iter().map(|e| e.stake_zat).sum();
    if total > 0 {
        for e in &mut entries {
            e.share = e.stake_zat as f64 / total as f64;
        }
    }
    entries.sort_by(|a, b| b.stake_zat.cmp(&a.stake_zat));
    entries
}

fn roster_entry_from_value(item: &Value) -> Option<RosterEntry> {
    // Season 1 monolith: [ "64-hex-finalizer", stake_ctaz_float ]
    if let Some(pair) = item.as_array() {
        if pair.len() >= 2 {
            let finalizer = pair[0].as_str()?.trim().to_ascii_lowercase();
            let stake = stake_from_value(&pair[1])?;
            return Some(RosterEntry {
                finalizer,
                stake_zat: stake,
                share: 0.0,
            });
        }
        return None;
    }
    if let Some(obj) = item.as_object() {
        let finalizer = obj
            .get("finalizer")
            .or_else(|| obj.get("pub_key"))
            .or_else(|| obj.get("id"))
            .or_else(|| obj.get("pk"))
            .or_else(|| obj.get("pubkey"))
            .or_else(|| obj.get("key"))
            .and_then(|v| match v {
                Value::String(s) => Some(s.clone()),
                Value::Array(_) => normalize_hash_value(v),
                _ => None,
            })?;
        let stake = obj
            .get("stake")
            .or_else(|| obj.get("amount"))
            .or_else(|| obj.get("zats"))
            .or_else(|| obj.get("zec"))
            .or_else(|| obj.get("voting_power"))
            .and_then(stake_from_value)
            .unwrap_or(0);
        return Some(RosterEntry {
            finalizer,
            stake_zat: stake,
            share: 0.0,
        });
    }
    None
}

fn stake_from_value(v: &Value) -> Option<u64> {
    match v {
        Value::Number(n) => {
            if let Some(u) = n.as_u64() {
                Some(u)
            } else {
                n.as_f64().map(|f| {
                    // Roster-in-ZEC often returns floats; treat values < 1e6 as ZEC.
                    if f.abs() < 1_000_000.0 {
                        (f * 100_000_000.0).round() as u64
                    } else {
                        f.round() as u64
                    }
                })
            }
        }
        Value::String(s) => s.parse::<u64>().ok().or_else(|| {
            s.parse::<f64>().ok().map(|f| {
                if f.abs() < 1_000_000.0 {
                    (f * 100_000_000.0).round() as u64
                } else {
                    f.round() as u64
                }
            })
        }),
        Value::Object(o) => o
            .get("zats")
            .or_else(|| o.get("amount"))
            .and_then(stake_from_value),
        _ => None,
    }
}

pub async fn block_finality(client: &ZebraClient, hash: &str) -> NozyResult<Value> {
    client
        .call_rpc(
            "get_tfl_block_finality_from_hash",
            Value::Array(vec![Value::String(hash.to_string())]),
        )
        .await
}

pub async fn tx_finality(client: &ZebraClient, txid: &str) -> NozyResult<Value> {
    client
        .call_rpc(
            "get_tfl_tx_finality_from_hash",
            Value::Array(vec![Value::String(txid.to_string())]),
        )
        .await
}

pub async fn bond_info(client: &ZebraClient, pk_from_positions: &str) -> NozyResult<Value> {
    let native = reverse_bond_pk_hex(pk_from_positions)?;
    client
        .call_rpc("getbondinfo", Value::Array(vec![Value::String(native)]))
        .await
}

pub async fn staking_action(client: &ZebraClient, action: &StakingAction) -> NozyResult<Value> {
    // Retarget / actions use pk as returned by wallet_staking_positions (not reversed).
    if let StakingAction::CreateNewDelegationBond {
        target_finalizer, ..
    } = action
    {
        normalize_finalizer_hex(target_finalizer)?;
    }
    if let StakingAction::RetargetDelegationBond {
        target_finalizer, ..
    } = action
    {
        normalize_finalizer_hex(target_finalizer)?;
    }

    client
        .call_rpc(
            "wallet_staking_action",
            Value::Array(vec![action.to_rpc_param()]),
        )
        .await
}

pub async fn fetch_guardian_snapshot(client: &ZebraClient) -> NozyResult<GuardianSnapshot> {
    let height = client.get_block_count().await?;
    let staking_day = staking_day_at(height);
    let tfl_activated = is_tfl_activated(client).await?;
    let finalized_tip = finality_tip(client).await?;
    let positions = staking_positions(client).await.unwrap_or_default();

    let (finalizer_count, pos_height) = match client
        .call_rpc::<Value>("get_tfl_recency_status", Value::Array(vec![]))
        .await
    {
        Ok(v) => {
            let count = v
                .get("finalizer_statuses")
                .and_then(|x| x.as_array())
                .map(|a| a.len());
            let pos = v
                .get("my_height")
                .and_then(|x| x.as_u64())
                .map(|h| h as u32);
            (count, pos)
        }
        Err(_) => (None, None),
    };

    let next_action = GuardianSnapshot::compute_next_action(&positions, &staking_day);
    let wallet = wallet_sync_status(client).await;

    Ok(GuardianSnapshot {
        rpc_url: client.rpc_url().to_string(),
        height,
        staking_day,
        tfl_activated,
        finalized_tip,
        positions,
        finalizer_count,
        pos_height,
        next_action,
        privacy_notes: vec![],
        wallet,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_roster_zec_tuple_array() {
        let raw = json!([
            [
                "abcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcd",
                123.5
            ],
            [
                "defdefdefdefdefdefdefdefdefdefdefdefdefdefdefdefdefdefdefdefdefdef",
                10.0
            ]
        ]);
        let entries = parse_roster(&raw);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].finalizer.len(), 64);
        assert_eq!(entries[0].stake_zat, 12_350_000_000);
    }

    #[test]
    fn parse_roster_zats_objects() {
        let raw = json!([
            {"pub_key": "abcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcd", "voting_power": 1000000000}
        ]);
        let entries = parse_roster(&raw);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].stake_zat, 1_000_000_000);
    }

    #[test]
    fn parse_wallet_ufvk_string_or_object() {
        assert_eq!(
            parse_wallet_ufvk(&json!("uviewtest1abc")).as_deref(),
            Some("uviewtest1abc")
        );
        assert_eq!(
            parse_wallet_ufvk(&json!({"ufvk": "uviewtest1xyz"})).as_deref(),
            Some("uviewtest1xyz")
        );
        assert!(parse_wallet_ufvk(&json!("")).is_none());
    }

    #[test]
    fn parse_wallet_sync_status_object() {
        let raw = json!({
            "sync_height": 455098,
            "tip_height": 455100,
            "user_shielded_spendable_zats": 2600000000u64,
            "user_shielded_pending_zats": 0,
            "user_unshielded_zats": 0,
            "staked_zats": 500000000000u64,
            "withdrawable_zats": 900000000u64
        });
        let s = parse_wallet_sync_status(&raw).unwrap();
        assert_eq!(s.available_to_stake_zats(), 2_600_000_000);
        assert!(s.wallet_synced());
    }
}
