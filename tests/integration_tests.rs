// test files for integration tests

use nozy::{
    HDWallet, NoteScanner, NozyError, OrchardTransactionBuilder, WalletStorage, ZebraClient,
};
use std::path::PathBuf;
use std::sync::Once;
use zcash_protocol::consensus::NetworkType;

static INIT: Once = Once::new();

fn setup_test_env() {
    INIT.call_once(|| {
        std::env::set_var("NOZY_TEST_NETWORK", "testnet");
    });
}

fn get_test_zebra_url() -> String {
    std::env::var("ZEBRA_RPC_URL").unwrap_or_else(|_| "http://127.0.0.1:8232".to_string())
}

fn create_test_storage() -> (WalletStorage, PathBuf) {
    let test_dir = PathBuf::from(format!("test_integration_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&test_dir); // Clean up if exists
    std::fs::create_dir_all(&test_dir).expect("Failed to create test directory");
    (WalletStorage::new(test_dir.clone()), test_dir)
}

fn cleanup_test_dir(dir: &PathBuf) {
    let _ = std::fs::remove_dir_all(dir);
}

async fn check_zebra_available(client: &ZebraClient) -> bool {
    client.test_connection().await.is_ok()
}

#[tokio::test]
#[ignore]
async fn test_wallet_create_and_save() {
    setup_test_env();

    let wallet = HDWallet::new().expect("Failed to create wallet");
    assert!(!wallet.get_mnemonic().is_empty());

    let (storage, test_dir) = create_test_storage();
    let password = "test_password_123";

    storage
        .save_wallet(&wallet, password)
        .await
        .expect("Failed to save wallet");

    let loaded_wallet = storage
        .load_wallet(password)
        .await
        .expect("Failed to load wallet");

    assert_eq!(wallet.get_mnemonic(), loaded_wallet.get_mnemonic());

    cleanup_test_dir(&test_dir);
}

#[tokio::test]
#[ignore]
async fn test_wallet_restore_from_mnemonic() {
    setup_test_env();

    let original_wallet = HDWallet::new().expect("Failed to create wallet");
    let mnemonic = original_wallet.get_mnemonic();

    let restored_wallet =
        HDWallet::from_mnemonic(&mnemonic).expect("Failed to restore wallet from mnemonic");

    assert_eq!(
        original_wallet.get_mnemonic(),
        restored_wallet.get_mnemonic()
    );

    let network = NetworkType::Main;
    let addr1 = original_wallet
        .generate_orchard_address(0, 0, network)
        .expect("Failed to generate address");
    let addr2 = restored_wallet
        .generate_orchard_address(0, 0, network)
        .expect("Failed to generate address");

    assert_eq!(addr1, addr2);
}

#[tokio::test]
#[ignore]
async fn test_address_generation() {
    setup_test_env();

    let wallet = HDWallet::new().expect("Failed to create wallet");

    let network = NetworkType::Test;
    for i in 0..5 {
        let address = wallet
            .generate_orchard_address(0, i, network)
            .expect("Failed to generate address");

        assert!(address.starts_with("u1") || address.starts_with("utest1"));
        assert!(address.len() > 50);
    }
}

#[tokio::test]
#[ignore]
async fn test_zebra_connection() {
    setup_test_env();

    let client = ZebraClient::new(get_test_zebra_url());

    if !check_zebra_available(&client).await {
        println!("⚠️  Zebra node not available - skipping test");
        return;
    }

    let block_count = client
        .get_block_count()
        .await
        .expect("Failed to get block count");

    assert!(block_count > 0);
    println!("✅ Connected to Zebra, block height: {}", block_count);
}

#[tokio::test]
#[ignore]
async fn test_end_to_end_flow_create_scan_send() {
    setup_test_env();

    let wallet = HDWallet::new().expect("Failed to create wallet");
    let address = wallet
        .generate_orchard_address(0, 0, NetworkType::Test)
        .expect("Failed to generate address");

    println!("✅ Created wallet, address: {}", address);

    let client = ZebraClient::new(get_test_zebra_url());

    if !check_zebra_available(&client).await {
        println!("⚠️  Zebra node not available - skipping end-to-end test");
        return;
    }

    let tip_height = client
        .get_block_count()
        .await
        .expect("Failed to get block count");
    println!("✅ Current block height: {}", tip_height);

    let start_height = tip_height.saturating_sub(100);
    let mut scanner = NoteScanner::new(wallet, client.clone());

    let (scan_result, spendable_notes) = scanner
        .scan_notes(Some(start_height), Some(tip_height))
        .await
        .expect("Failed to scan notes");

    println!(
        "✅ Scanned {} notes, {} spendable, balance: {:.8} ZEC",
        scan_result.notes.len(),
        spendable_notes.len(),
        scan_result.total_balance as f64 / 100_000_000.0
    );

    if spendable_notes.is_empty() {
        println!("⚠️  No spendable notes - skipping transaction building");
        return;
    }

    let amount_zatoshis = 10_000;
    let fee_zatoshis = 10_000;

    let orchard_balance = scan_result.total_balance;
    if orchard_balance < amount_zatoshis + fee_zatoshis {
        println!("⚠️  Insufficient funds for transaction test");
        return;
    }

    let recipient_wallet = HDWallet::new().expect("Failed to create recipient wallet");
    let recipient_address = recipient_wallet
        .generate_orchard_address(0, 0, NetworkType::Test)
        .expect("Failed to generate recipient address");

    let builder = OrchardTransactionBuilder::new_async(true)
        .await
        .expect("Failed to create transaction builder");

    use nozy::orchard_tx::ZebraJsonRpcOrchardWitnessProvider;

    let transaction = builder
        .build_single_spend(
            &client,
            &ZebraJsonRpcOrchardWitnessProvider,
            &spendable_notes,
            &recipient_address,
            amount_zatoshis,
            fee_zatoshis,
            Some(b"Integration test transaction"),
            nozy::PILOT_EXPIRY_DELTA_BLOCKS,
        )
        .await
        .expect("Failed to build transaction");

    println!(
        "✅ Transaction built: {} bytes (ZIP-225 v5 raw), txid {}",
        transaction.raw_transaction.len(),
        transaction.txid
    );
}

#[tokio::test]
#[ignore]
async fn test_note_scanning() {
    setup_test_env();

    let wallet = HDWallet::new().expect("Failed to create wallet");
    let client = ZebraClient::new(get_test_zebra_url());

    if !check_zebra_available(&client).await {
        println!("⚠️  Zebra node not available - skipping test");
        return;
    }

    let tip_height = client
        .get_block_count()
        .await
        .expect("Failed to get block count");

    let start_height = tip_height.saturating_sub(10);
    let mut scanner = NoteScanner::new(wallet, client);

    let (result, spendable) = scanner
        .scan_notes(Some(start_height), Some(tip_height))
        .await
        .expect("Failed to scan notes");

    println!(
        "✅ Scanned {} blocks, found {} notes, {} spendable",
        tip_height - start_height,
        result.notes.len(),
        spendable.len()
    );

    assert_eq!(result.notes.len(), result.unspent_count);
    assert!(spendable.len() <= result.notes.len());
}

#[tokio::test]
#[ignore]
async fn test_sync_follows_zebra_tip() {
    setup_test_env();

    let wallet = HDWallet::new().expect("Failed to create wallet");
    let client = ZebraClient::new(get_test_zebra_url());

    if !check_zebra_available(&client).await {
        println!("⚠️  Zebra node not available - skipping tip-follow test");
        return;
    }

    let tip1 = client
        .get_block_count()
        .await
        .expect("Failed to get initial block count");
    println!("🧪 Initial Zebra tip height: {}", tip1);

    let start1 = tip1.saturating_sub(50);
    let mut scanner1 = NoteScanner::new(wallet.clone(), client.clone());
    let (_result1, _spendable1) = scanner1
        .scan_notes(Some(start1), Some(tip1))
        .await
        .expect("First scan (up to tip1) failed");

    // Give Zebra time to advance; this test is best-effort and may no-op if the node is idle.
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    let tip2 = client
        .get_block_count()
        .await
        .expect("Failed to get second block count");
    println!("🧪 Second Zebra tip height: {}", tip2);

    if tip2 <= tip1 {
        println!(
            "ℹ️  Zebra did not advance (tip2 <= tip1); tip-follow behavior cannot be asserted here."
        );
        return;
    }

    let start2 = tip1.saturating_add(1);
    let mut scanner2 = NoteScanner::new(wallet, client);
    let (_result2, _spendable2) = scanner2
        .scan_notes(Some(start2), Some(tip2))
        .await
        .expect("Second scan (up to tip2) failed");

    println!(
        "✅ Sync followed Zebra tip: first pass ended at {}, second pass ended at {}",
        tip1, tip2
    );
}

fn get_test_lightwalletd_url() -> String {
    std::env::var("LIGHTWALLETD_GRPC").unwrap_or_else(|_| "http://127.0.0.1:9067".to_string())
}

async fn check_lightwalletd_available(url: &str) -> bool {
    match zeaking::lwd::connect_lightwalletd(url).await {
        Ok(mut client) => zeaking::lwd::chain_tip_height(&mut client).await.is_ok(),
        Err(_) => false,
    }
}

#[tokio::test]
#[ignore]
async fn test_lwd_compact_sync_follows_tip() {
    setup_test_env();

    let lwd_url = get_test_lightwalletd_url();
    if !check_lightwalletd_available(&lwd_url).await {
        println!("⚠️  lightwalletd not available at {} - skipping LWD tip-follow test", lwd_url);
        return;
    }

    let db_path = std::env::temp_dir().join(format!(
        "nozy_lwd_tip_follow_test_{}.sqlite",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&db_path);

    let mut client = zeaking::lwd::connect_lightwalletd(&lwd_url)
        .await
        .expect("connect lightwalletd");
    let tip1 = zeaking::lwd::chain_tip_height(&mut client)
        .await
        .expect("chain tip 1");
    println!("🧪 Initial LWD tip height: {}", tip1);

    let store = zeaking::lwd::LwdCompactStore::open(&db_path).expect("open compact store");
    let window = 32u64;
    let start_floor = tip1.saturating_sub(window).max(1);
    let stats1 = zeaking::lwd::sync_compact_to_tip_with_options(
        &mut client,
        &store,
        zeaking::lwd::SyncCompactToTipOptions {
            start_floor: Some(start_floor),
            ..Default::default()
        },
    )
    .await
    .expect("first compact-to-tip failed");
    assert!(stats1.range_end <= tip1);
    println!(
        "🧪 First compact sync: range {}-{} tip={} written={}",
        stats1.range_start_effective,
        stats1.range_end,
        stats1.chain_tip,
        stats1.blocks_written
    );

    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    let mut client2 = zeaking::lwd::connect_lightwalletd(&lwd_url)
        .await
        .expect("reconnect lightwalletd");
    let tip2 = zeaking::lwd::chain_tip_height(&mut client2)
        .await
        .expect("chain tip 2");
    println!("🧪 Second LWD tip height: {}", tip2);

    if tip2 <= tip1 {
        println!(
            "ℹ️  LWD did not advance (tip2 <= tip1); tip-follow behavior cannot be asserted here."
        );
        let _ = std::fs::remove_file(&db_path);
        return;
    }

    let stats2 = zeaking::lwd::sync_compact_to_tip_with_options(
        &mut client2,
        &store,
        zeaking::lwd::SyncCompactToTipOptions::default(),
    )
    .await
    .expect("second compact-to-tip failed");

    assert!(
        stats2.range_end >= tip2 || stats2.already_at_tip,
        "second sync should reach tip2={} (range_end={}, already_at_tip={})",
        tip2,
        stats2.range_end,
        stats2.already_at_tip
    );

    let max_h = store.max_compact_height().expect("max height").unwrap_or(0);
    assert!(max_h >= stats1.range_end.min(tip1));

    println!(
        "✅ LWD compact sync followed tip: first ended at {}, second reached {}",
        tip1, tip2
    );

    let _ = std::fs::remove_file(&db_path);
}

#[tokio::test]
#[ignore]
async fn test_transaction_building() {
    setup_test_env();

    let client = ZebraClient::new(get_test_zebra_url());

    if !check_zebra_available(&client).await {
        println!("⚠️  Zebra node not available - skipping test");
        return;
    }

    let recipient_wallet = HDWallet::new().expect("Failed to create recipient wallet");
    let recipient_address = recipient_wallet
        .generate_orchard_address(0, 0, NetworkType::Test)
        .expect("Failed to generate recipient address");

    let builder = OrchardTransactionBuilder::new_async(true)
        .await
        .expect("Failed to create transaction builder");

    let empty_notes = vec![];
    use nozy::orchard_tx::ZebraJsonRpcOrchardWitnessProvider;

    let result = builder
        .build_single_spend(
            &client,
            &ZebraJsonRpcOrchardWitnessProvider,
            &empty_notes,
            &recipient_address,
            10_000,
            10_000,
            None,
            nozy::PILOT_EXPIRY_DELTA_BLOCKS,
        )
        .await;

    assert!(result.is_err());
    match result {
        Err(NozyError::InsufficientFunds(_)) | Err(NozyError::InvalidOperation(_)) => {
            println!("✅ Transaction building correctly rejects empty / insufficient input");
        }
        Err(e) => {
            println!("⚠️  Unexpected error: {:?}", e);
        }
        Ok(_) => {
            panic!("Expected error for empty notes");
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_wallet_storage_with_password() {
    setup_test_env();

    let wallet = HDWallet::new().expect("Failed to create wallet");
    let (storage, test_dir) = create_test_storage();

    let password = "secure_test_password_123";
    let mut wallet = wallet;
    wallet
        .set_password(password)
        .expect("Failed to set password");

    storage
        .save_wallet(&wallet, password)
        .await
        .expect("Failed to save wallet");

    let wrong_result = storage.load_wallet("wrong_password").await;
    assert!(wrong_result.is_err());

    let loaded = storage
        .load_wallet(password)
        .await
        .expect("Failed to load wallet with correct password");

    assert_eq!(wallet.get_mnemonic(), loaded.get_mnemonic());

    cleanup_test_dir(&test_dir);
}

#[tokio::test]
#[ignore]
async fn test_multiple_address_generation() {
    setup_test_env();

    let wallet = HDWallet::new().expect("Failed to create wallet");
    let mut addresses = Vec::new();

    let network = NetworkType::Main;
    for i in 0..10 {
        let addr = wallet
            .generate_orchard_address(0, i, network)
            .expect("Failed to generate address");
        addresses.push(addr);
    }

    for i in 0..addresses.len() {
        for j in (i + 1)..addresses.len() {
            assert_ne!(
                addresses[i], addresses[j],
                "Addresses at index {} and {} should be different",
                i, j
            );
        }
    }

    println!("✅ Generated {} unique addresses", addresses.len());
}
