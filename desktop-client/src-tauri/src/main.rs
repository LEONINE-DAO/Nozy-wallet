// Prevents additional console window on Windows in release mode
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod error;
mod session;

use commands::*;

fn network_from_config() -> zcash_protocol::consensus::NetworkType {
    let config = nozy::load_config();
    if config.network == "testnet" {
        zcash_protocol::consensus::NetworkType::Test
    } else {
        zcash_protocol::consensus::NetworkType::Main
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            use tauri::Manager;

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_size(tauri::LogicalSize::new(1200.0, 800.0));
                let _ = window.center();
                let _ = window.show();
                let _ = window.set_focus();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            wallet_exists,
            create_wallet,
            restore_wallet,
            unlock_wallet,
            lock_wallet,
            change_password,
            get_mnemonic,
            get_private_key,
            get_wallet_status,
            list_wallet_profiles_cmd,
            switch_wallet_profile,
            generate_address,
            get_balance,
            prepare_cosign_request,
            sign_cosign_request,
            complete_cosign_send,
            get_keystone_status,
            set_keystone_enabled,
            export_keystone_ufvk,
            keystone_prepare_send,
            keystone_complete_send,
            get_sync_status,
            get_orchard_pool_stats,
            sync_wallet,
            send_transaction,
            estimate_fee,
            get_transaction_history,
            get_transaction,
            speed_up_transaction,
            check_transaction_confirmations,
            get_config,
            set_zebra_url,
            test_zebra_connection,
            check_proving_status,
            download_proving_parameters,
            get_notes,
            lwd_get_info,
            lwd_chain_tip,
            lwd_sync_compact,
            lwd_sync_compact_to_tip,
            address_book_list,
            address_book_add,
            address_book_remove,
            address_book_get,
            address_book_search,
            export_backup,
            restore_from_backup,
            list_backups,
            sign_message,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
