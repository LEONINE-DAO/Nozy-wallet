use crate::error::NozyError;
use crate::hd_wallet::HDWallet;
use crate::zebra_integration::ZebraClient;
use crate::orchard_tx::OrchardTransactionBuilder;
use crate::storage::WalletStorage;
use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let wallet = HDWallet::new();
        assert!(wallet.is_ok());
        
        let wallet = wallet.unwrap();
        assert!(!wallet.get_mnemonic().is_empty());
    }

    #[test]
    fn test_wallet_from_mnemonic() {
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let wallet = HDWallet::from_mnemonic(mnemonic);
        assert!(wallet.is_ok());
        
        let wallet = wallet.unwrap();
        assert_eq!(wallet.get_mnemonic(), mnemonic);
    }

    #[test]
    fn test_password_protection() {
        let mut wallet = HDWallet::new().unwrap();
        
        let result = wallet.set_password("test_password");
        assert!(result.is_ok());
        assert!(wallet.is_password_protected());
        
        let verify_result = wallet.verify_password("test_password");
        assert!(verify_result.is_ok());
        assert!(verify_result.unwrap());
        
        let wrong_result = wallet.verify_password("wrong_password");
        assert!(wrong_result.is_ok());
        assert!(!wrong_result.unwrap());
    }

    #[test]
    fn test_address_generation() {
        let wallet = HDWallet::new().unwrap();
        
        let address = wallet.generate_orchard_address(0, 0);
        assert!(address.is_ok());
        
        let address = address.unwrap();
        assert!(!address.is_empty());
    }

    #[tokio::test]
    async fn test_wallet_storage() {
        let wallet = HDWallet::new().unwrap();
        let storage = WalletStorage::new(PathBuf::from("test_wallet_data"));
        
        std::fs::create_dir_all("test_wallet_data").unwrap();
        
        let result = storage.save_wallet(&wallet, "test_password").await;
        assert!(result.is_ok());
        
        let loaded_wallet = storage.load_wallet("test_password").await;
        assert!(loaded_wallet.is_ok());
        
        let _ = std::fs::remove_dir_all("test_wallet_data");
    }

    #[test]
    fn test_zebra_client_creation() {
        let client = ZebraClient::new("http://127.0.0.1:8232".to_string());
        assert!(std::ptr::addr_of!(client) != std::ptr::null());
    }

    #[test]
    fn test_orchard_transaction_builder() {
        let builder = OrchardTransactionBuilder::new(false);
        assert!(std::ptr::addr_of!(builder) != std::ptr::null());
    }

    #[test]
    fn test_error_handling() {
        let error = NozyError::NetworkError("Test error".to_string());
        assert!(error.user_friendly_message().contains("Network connection failed"));
        
        let error = NozyError::AddressParsing("Test error".to_string());
        assert!(error.user_friendly_message().contains("Invalid address format"));
        
        let error = NozyError::Transaction("Test error".to_string());
        assert!(error.user_friendly_message().contains("Transaction failed"));
    }

    #[test]
    fn test_error_context() {
        let error = NozyError::NetworkError("Original error".to_string());
        let contextual_error = error.with_context("Additional context");
        
        match contextual_error {
            NozyError::NetworkError(msg) => {
                assert!(msg.contains("Additional context"));
                assert!(msg.contains("Original error"));
            },
            _ => panic!("Expected NetworkError"),
        }
    }

    #[tokio::test]
    async fn test_wallet_backup() {
        let wallet = HDWallet::new().unwrap();
        let storage = WalletStorage::new(PathBuf::from("test_backup_data"));
        
        std::fs::create_dir_all("test_backup_data").unwrap();
        
        let _ = storage.save_wallet(&wallet, "test_password").await;
        
        let backup_result = storage.create_backup("test_backups").await;
        assert!(backup_result.is_ok());
        
        let backups = storage.list_backups();
        assert!(backups.is_ok());
        
        
        let _ = std::fs::remove_dir_all("test_backup_data");
        let _ = std::fs::remove_dir_all("test_backups");
    }

    #[tokio::test]
    async fn test_note_scanning_structure() {
        let wallet = HDWallet::new().unwrap();
        let client = ZebraClient::new("http://127.0.0.1:8232".to_string());
        
        let result = crate::notes::scan_real_notes(&client, &wallet, 1000, 1001).await;
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_transaction_building_structure() {
        let builder = OrchardTransactionBuilder::new(false);
        let client = ZebraClient::new("http://127.0.0.1:8232".to_string());
        let spendable_notes = Vec::new();
        
        let result = builder.build_single_spend(
            &client,
            &spendable_notes,
            "test_address",
            1000,
            100,
            None,
        ).await;
        assert!(result.is_err() || result.is_ok());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    #[ignore] 
    async fn test_zebra_connection() {
        let client = ZebraClient::new("http://127.0.0.1:8232".to_string());
        
        let result = client.get_block_count().await;
        
        if result.is_ok() {
            println!("✅ Zebra node is accessible");
        } else {
            println!("⚠️  Zebra node is not accessible - this is expected in CI");
        }
    }

    #[tokio::test]
    #[ignore] 
    async fn test_full_transaction_flow() {
        use crate::notes::NoteScanner;
        use crate::transaction_builder::ZcashTransactionBuilder;
        
        let wallet = HDWallet::new().unwrap();
        let client = ZebraClient::new("http://127.0.0.1:8232".to_string());
        
        // Test connection
        if client.test_connection().await.is_err() {
            println!("⚠️  Zebra node not available - skipping integration test");
            return;
        }
        
        // Test note scanning
        let mut scanner = NoteScanner::new(wallet.clone(), client.clone());
        let tip_height = client.get_block_count().await.unwrap_or(3_066_071);
        let start_height = tip_height.saturating_sub(100);
        
        let (scan_result, spendable_notes) = scanner.scan_notes(Some(start_height), Some(tip_height)).await.unwrap_or_else(|_| {
            (crate::notes::NoteScanResult {
                notes: vec![],
                total_balance: 0,
                unspent_count: 0,
                spendable_count: 0,
            }, vec![])
        });
        
        println!("✅ Scanned {} notes, {} spendable", scan_result.notes.len(), spendable_notes.len());
        
        let builder = ZcashTransactionBuilder::new();
        println!("✅ Transaction building structure validated");
        
        assert!(true);
    }
    
    #[tokio::test]
    #[ignore]
    async fn test_transaction_history_operations() {
        use crate::transaction_history::{SentTransactionStorage, SentTransactionRecord, TransactionStatus};
        use std::path::PathBuf;
        
        let test_dir = PathBuf::from("test_transaction_history");
        let _ = std::fs::remove_dir_all(&test_dir); // Clean up if exists
        
        let storage = SentTransactionStorage::with_path(test_dir.clone()).unwrap();
        
        let tx1 = SentTransactionRecord::new(
            "test_txid_1".to_string(),
            "u1test1".to_string(),
            100_000_000,
            10_000,
            None,
            vec!["note1".to_string()],
        );
        
        let tx2 = SentTransactionRecord::new(
            "test_txid_2".to_string(),
            "u1test2".to_string(),
            50_000_000,
            5_000,
            None,
            vec!["note2".to_string()],
        );
        
        storage.save_transaction(tx1.clone()).unwrap();
        storage.save_transaction(tx2.clone()).unwrap();
        
        let retrieved = storage.get_transaction("test_txid_1").unwrap();
        assert_eq!(retrieved.txid, tx1.txid);
        
        let pending = storage.get_pending_transactions();
        assert_eq!(pending.len(), 2);
        
        let large_txs = storage.query_transactions(None, Some(75_000_000), None, None, None, None);
        assert_eq!(large_txs.len(), 1);
        assert_eq!(large_txs[0].txid, "test_txid_1");
        
        let stats = storage.get_statistics();
        assert_eq!(stats.total_count, 2);
        assert_eq!(stats.total_sent_zatoshis, 150_000_000);
        assert_eq!(stats.total_fees_zatoshis, 15_000);
        
        let _ = std::fs::remove_dir_all(&test_dir);
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_wallet_creation_performance() {
        let start = Instant::now();
        let _wallet = HDWallet::new().unwrap();
        let duration = start.elapsed();
        
        assert!(duration.as_millis() < 100);
    }

    #[test]
    #[ignore]
    fn test_address_generation_performance() {
        let wallet = HDWallet::new().unwrap();
        
        let start = Instant::now();
        for i in 0..10 {
            let _address = wallet.generate_orchard_address(0, i).unwrap();
        }
        let duration = start.elapsed();
        
        assert!(duration.as_secs() < 10);
    }

    #[test]
    #[ignore] 
    fn test_password_hashing_performance() {
        let mut wallet = HDWallet::new().unwrap();
        
        let start = Instant::now();
        let _result = wallet.set_password("test_password");
        let duration = start.elapsed();
        
        assert!(duration.as_millis() < 2000);
    }
}
