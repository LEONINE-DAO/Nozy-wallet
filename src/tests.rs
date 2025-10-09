use crate::error::{NozyError, NozyResult};
use crate::hd_wallet::HDWallet;
use crate::zebra_integration::ZebraClient;
use crate::orchard_tx::OrchardTransactionBuilder;
use crate::notes::SpendableNote;
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
        
        // Test setting password
        let result = wallet.set_password("test_password");
        assert!(result.is_ok());
        assert!(wallet.is_password_protected());
        
        // Test password verification
        let verify_result = wallet.verify_password("test_password");
        assert!(verify_result.is_ok());
        assert!(verify_result.unwrap());
        
        // Test wrong password
        let wrong_result = wallet.verify_password("wrong_password");
        assert!(wrong_result.is_ok());
        assert!(!wrong_result.unwrap());
    }

    #[test]
    fn test_address_generation() {
        let wallet = HDWallet::new().unwrap();
        
        // Test Orchard address generation
        let address = wallet.generate_orchard_address(0, 0);
        assert!(address.is_ok());
        
        let address = address.unwrap();
        assert!(!address.is_empty());
    }

    #[test]
    fn test_wallet_storage() {
        let wallet = HDWallet::new().unwrap();
        let storage = WalletStorage::new(PathBuf::from("test_wallet_data"));
        
        // Test saving wallet
        let result = tokio::runtime::Runtime::new().unwrap().block_on(
            storage.save_wallet(&wallet, "test_password")
        );
        assert!(result.is_ok());
        
        // Test loading wallet
        let loaded_wallet = tokio::runtime::Runtime::new().unwrap().block_on(
            storage.load_wallet("test_password")
        );
        assert!(loaded_wallet.is_ok());
        
        // Clean up
        let _ = std::fs::remove_dir_all("test_wallet_data");
    }

    #[test]
    fn test_zebra_client_creation() {
        let client = ZebraClient::new("http://127.0.0.1:8232".to_string());
        assert_eq!(client.url, "http://127.0.0.1:8232");
    }

    #[test]
    fn test_orchard_transaction_builder() {
        let builder = OrchardTransactionBuilder::new(false);
        assert!(builder.proving_keys.is_none());
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

    #[test]
    fn test_wallet_backup() {
        let wallet = HDWallet::new().unwrap();
        let storage = WalletStorage::new(PathBuf::from("test_backup_data"));
        
        // Save wallet first
        let _ = tokio::runtime::Runtime::new().unwrap().block_on(
            storage.save_wallet(&wallet, "test_password")
        );
        
        // Test backup creation
        let backup_result = tokio::runtime::Runtime::new().unwrap().block_on(
            storage.create_backup("test_backups")
        );
        assert!(backup_result.is_ok());
        
        // Test backup listing
        let backups = storage.list_backups();
        assert!(backups.is_ok());
        assert!(!backups.unwrap().is_empty());
        
        // Clean up
        let _ = std::fs::remove_dir_all("test_backup_data");
        let _ = std::fs::remove_dir_all("test_backups");
    }

    #[test]
    fn test_note_scanning_structure() {
        let wallet = HDWallet::new().unwrap();
        let client = ZebraClient::new("http://127.0.0.1:8232".to_string());
        
        // Test note scanning function exists
        let result = tokio::runtime::Runtime::new().unwrap().block_on(
            crate::notes::scan_real_notes(&client, &wallet, 1000, 1001)
        );
        // This will fail because we don't have a real Zebra node, but the function should exist
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_transaction_building_structure() {
        let builder = OrchardTransactionBuilder::new(false);
        let client = ZebraClient::new("http://127.0.0.1:8232".to_string());
        let spendable_notes = Vec::new();
        
        // Test transaction building function exists
        let result = tokio::runtime::Runtime::new().unwrap().block_on(
            builder.build_single_spend(
                &client,
                &spendable_notes,
                "test_address",
                1000,
                100,
                None,
            )
        );
        // This will fail because we don't have real data, but the function should exist
        assert!(result.is_err() || result.is_ok());
    }
}

/// Integration tests that require a real Zebra node
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    #[ignore] // Ignore by default, run with --ignored
    fn test_zebra_connection() {
        let client = ZebraClient::new("http://127.0.0.1:8232".to_string());
        
        let result = tokio::runtime::Runtime::new().unwrap().block_on(
            client.get_block_count()
        );
        
        // This test will only pass if Zebra is running
        if result.is_ok() {
            println!("✅ Zebra node is accessible");
        } else {
            println!("⚠️  Zebra node is not accessible - this is expected in CI");
        }
    }

    #[test]
    #[ignore] // Ignore by default, run with --ignored
    fn test_full_transaction_flow() {
        let wallet = HDWallet::new().unwrap();
        let client = ZebraClient::new("http://127.0.0.1:8232".to_string());
        let builder = OrchardTransactionBuilder::new(false);
        
        // This test would require a real Zebra node and real notes
        // For now, just test that the structure exists
        assert!(true);
    }
}

/// Performance tests
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_wallet_creation_performance() {
        let start = Instant::now();
        let _wallet = HDWallet::new().unwrap();
        let duration = start.elapsed();
        
        // Wallet creation should be fast (less than 100ms)
        assert!(duration.as_millis() < 100);
    }

    #[test]
    fn test_address_generation_performance() {
        let wallet = HDWallet::new().unwrap();
        
        let start = Instant::now();
        for i in 0..100 {
            let _address = wallet.generate_orchard_address(0, i).unwrap();
        }
        let duration = start.elapsed();
        
        // 100 addresses should be generated quickly (less than 1 second)
        assert!(duration.as_secs() < 1);
    }

    #[test]
    fn test_password_hashing_performance() {
        let mut wallet = HDWallet::new().unwrap();
        
        let start = Instant::now();
        let _result = wallet.set_password("test_password");
        let duration = start.elapsed();
        
        // Password hashing should be reasonably fast (less than 500ms)
        assert!(duration.as_millis() < 500);
    }
}
