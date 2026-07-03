//! Integration test for wallet restore + save (uses BIP39 test vector, not real user seeds).

use nozy::{paths::get_wallet_data_dir, HDWallet, WalletStorage};

const TEST_MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

#[tokio::test]
async fn restore_save_and_load_roundtrip() {
    let wallet_dir = get_wallet_data_dir();
    let wallet_path = wallet_dir.join("wallet.dat");
    let backup = wallet_path.with_extension("dat.pre-restore-test");
    if wallet_path.exists() {
        std::fs::rename(&wallet_path, &backup).ok();
    }

    let wallet = HDWallet::from_mnemonic(TEST_MNEMONIC).expect("valid test mnemonic");
    let storage = WalletStorage::new(wallet_dir.clone());
    storage
        .save_wallet(&wallet, "test-password-123")
        .await
        .expect("save should succeed");

    let loaded = storage
        .load_wallet("test-password-123")
        .await
        .expect("load should succeed");
    assert_eq!(loaded.get_mnemonic(), TEST_MNEMONIC);

    let _ = std::fs::remove_file(&wallet_path);
    if backup.exists() {
        std::fs::rename(&backup, &wallet_path).ok();
    }
}

/// Run manually with real seed: `$env:NOZY_TEST_MNEMONIC='word1 word2 ...'; cargo test restore_from_env -- --ignored --nocapture`
#[tokio::test]
#[ignore = "manual: set NOZY_TEST_MNEMONIC env var"]
async fn restore_from_env() {
    let mnemonic = std::env::var("NOZY_TEST_MNEMONIC").expect("NOZY_TEST_MNEMONIC not set");
    let password = std::env::var("NOZY_TEST_PASSWORD").unwrap_or_default();
    let words: Vec<&str> = mnemonic.split_whitespace().collect();
    assert!(
        matches!(words.len(), 12 | 15 | 18 | 21 | 24),
        "unexpected word count: {}",
        words.len()
    );

    let wallet = HDWallet::from_mnemonic(&words.join(" ")).expect("from_mnemonic");
    let storage = WalletStorage::with_xdg_dir();
    storage
        .save_wallet(&wallet, &password)
        .await
        .expect("save_wallet");
    let loaded = storage.load_wallet(&password).await.expect("load_wallet");
    assert_eq!(loaded.get_mnemonic(), words.join(" "));
    eprintln!(
        "restore_from_env ok: {} words, password_len={}",
        words.len(),
        password.len()
    );
}
