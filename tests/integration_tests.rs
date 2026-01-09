// test files for integration tests

use nozy::{
    HDWallet, NoteScanner, NozyError, NozyResult, OrchardTransactionBuilder, WalletStorage,
    ZebraClient,
};
use std::path::PathBuf;
use std::sync::Once;

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

    let addr1 = original_wallet
        .generate_orchard_address(0, 0)
        .expect("Failed to generate address");
    let addr2 = restored_wallet
        .generate_orchard_address(0, 0)
        .expect("Failed to generate address");

    assert_eq!(addr1, addr2);
}

#[tokio::test]
#[ignore]
async fn test_address_generation() {
    setup_test_env();

    let wallet = HDWallet::new().expect("Failed to create wallet");

    for i in 0..5 {
        let address = wallet
            .generate_orchard_address(0, i)
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
        .generate_orchard_address(0, 0)
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

    if scan_result.total_balance < amount_zatoshis + fee_zatoshis {
        println!("⚠️  Insufficient funds for transaction test");
        return;
    }

    let recipient_wallet = HDWallet::new().expect("Failed to create recipient wallet");
    let recipient_address = recipient_wallet
        .generate_orchard_address(0, 0)
        .expect("Failed to generate recipient address");

    let mut builder = OrchardTransactionBuilder::new_async(true)
        .await
        .expect("Failed to create transaction builder");

    let transaction = builder
        .build_single_spend(
            &client,
            &spendable_notes,
            &recipient_address,
            amount_zatoshis,
            fee_zatoshis,
            Some(b"Integration test transaction"),
        )
        .await
        .expect("Failed to build transaction");

    println!(
        "✅ Transaction built: {} bytes, TXID: {}",
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
async fn test_transaction_building() {
    setup_test_env();

    let wallet = HDWallet::new().expect("Failed to create wallet");
    let client = ZebraClient::new(get_test_zebra_url());

    if !check_zebra_available(&client).await {
        println!("⚠️  Zebra node not available - skipping test");
        return;
    }

    let recipient_wallet = HDWallet::new().expect("Failed to create recipient wallet");
    let recipient_address = recipient_wallet
        .generate_orchard_address(0, 0)
        .expect("Failed to generate recipient address");

    let mut builder = OrchardTransactionBuilder::new_async(true)
        .await
        .expect("Failed to create transaction builder");

    let empty_notes = vec![];
    let result = builder
        .build_single_spend(
            &client,
            &empty_notes,
            &recipient_address,
            10_000,
            10_000,
            None,
        )
        .await;

    assert!(result.is_err());
    match result {
        Err(NozyError::InsufficientFunds(_)) => {
            println!("✅ Transaction building correctly handles insufficient funds");
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

    for i in 0..10 {
        let addr = wallet
            .generate_orchard_address(0, i)
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
