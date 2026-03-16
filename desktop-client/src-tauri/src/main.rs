// Prevents additional console window on Windows in release mode
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod error;

use commands::*;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            wallet_exists,
            create_wallet,
            restore_wallet,
            unlock_wallet,
            get_wallet_status,
            
            generate_address,
            
            get_balance,
            sync_wallet,
            
            send_transaction,
            estimate_fee,
            get_transaction_history,
            get_transaction,
            
            get_config,
            set_zebra_url,
            test_zebra_connection,
            
            check_proving_status,
            download_proving_parameters,
            
            get_notes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

