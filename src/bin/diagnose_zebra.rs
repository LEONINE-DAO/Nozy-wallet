use nozy::ZebraClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Zebra Node RPC Diagnostic Tool");
    println!("==================================\n");

    // Get Zebra URL from command line or use default
    let zebra_url = env::args()
        .nth(1)
        .unwrap_or_else(|| "http://127.0.0.1:8232".to_string());

    println!("🔗 Testing connection to: {}", zebra_url);
    println!();

    let client = ZebraClient::new(zebra_url.clone());

    // Test 1: Basic connectivity
    println!("1️⃣ Testing basic connectivity...");
    match test_basic_connectivity(&client).await {
        Ok(_) => println!("✅ Basic connectivity: PASSED"),
        Err(e) => {
            println!("❌ Basic connectivity: FAILED - {}", e);
            print_troubleshooting_tips();
            return Ok(());
        }
    }

    // Test 2: RPC method availability
    println!("\n2️⃣ Testing RPC method availability...");
    test_rpc_methods(&client).await;

    // Test 3: Response format validation
    println!("\n3️⃣ Testing response format validation...");
    test_response_formats(&client).await;

    // Test 4: Orchard-specific functionality
    println!("\n4️⃣ Testing Orchard-specific functionality...");
    test_orchard_functionality(&client).await;

    println!("\n🎉 Diagnostic complete!");
    Ok(())
}

async fn test_basic_connectivity(client: &ZebraClient) -> Result<(), Box<dyn std::error::Error>> {
    // Test with a simple RPC call
    let response = client.get_block_count().await?;
    println!("   📊 Block count: {}", response);

    if response == 0 {
        println!("   ⚠️  Warning: Block count is 0 - node may not be synchronized");
    }

    Ok(())
}

async fn test_rpc_methods(client: &ZebraClient) {
    let methods = vec![
        ("getblockcount", "Basic block count"),
        ("getnetworkinfo", "Network information"),
        ("getmempoolinfo", "Mempool information"),
        ("gettxoutsetinfo", "UTXO set information"),
        ("getrawtransaction", "Raw transaction data"),
        ("decoderawtransaction", "Transaction decoding"),
        ("getblocktemplate", "Block template"),
        ("estimatefee", "Fee estimation"),
    ];

    for (method, description) in methods {
        match test_single_rpc_method(client, method).await {
            Ok(_) => println!("   ✅ {}: Available", description),
            Err(e) => println!("   ❌ {}: Not available - {}", description, e),
        }
    }
}

async fn test_single_rpc_method(
    client: &ZebraClient,
    method: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Test different methods based on their expected parameters
    match method {
        "getblockcount" => {
            let _ = client.get_block_count().await?;
        }
        "getnetworkinfo" => {
            let _ = client.get_network_info().await?;
        }
        "getmempoolinfo" => {
            let _ = client.get_mempool_info().await?;
        }
        "gettxoutsetinfo" => {
            let _ = client.get_txout_set_info().await?;
        }
        "getrawtransaction" => {
            // This would need a real txid, so we'll skip for now
            return Err("Requires real transaction ID".into());
        }
        "decoderawtransaction" => {
            // This would need a real raw transaction, so we'll skip for now
            return Err("Requires real raw transaction".into());
        }
        "getblocktemplate" => {
            let _ = client.get_block_template().await?;
        }
        "estimatefee" => {
            let _ = client.get_fee_estimate().await?;
        }
        _ => {
            return Err("Unknown method".into());
        }
    }

    Ok(())
}

async fn test_response_formats(client: &ZebraClient) {
    println!("   🔍 Testing response format compatibility...");

    // Test getblockcount response format
    match client.get_block_count().await {
        Ok(count) => {
            println!("   ✅ getblockcount: Returns u32 ({})", count);
        }
        Err(e) => {
            println!("   ❌ getblockcount: Format error - {}", e);
        }
    }

    // Test getnetworkinfo response format
    match client.get_network_info().await {
        Ok(info) => {
            println!(
                "   ✅ getnetworkinfo: Returns HashMap with {} fields",
                info.len()
            );
            if let Some(chain) = info.get("chain") {
                println!("      Chain: {:?}", chain);
            }
        }
        Err(e) => {
            println!("   ❌ getnetworkinfo: Format error - {}", e);
        }
    }
}

async fn test_orchard_functionality(client: &ZebraClient) {
    println!("   🌳 Testing Orchard-specific methods...");

    // Test get_orchard_tree_state
    match client.get_orchard_tree_state(1000).await {
        Ok(tree_state) => {
            println!("   ✅ get_orchard_tree_state: Working");
            println!("      Height: {}", tree_state.height);
            println!("      Anchor: {}", hex::encode(tree_state.anchor));
            println!("      Commitments: {}", tree_state.commitment_count);
        }
        Err(e) => {
            println!("   ❌ get_orchard_tree_state: Error - {}", e);
        }
    }

    // Test get_note_position
    let test_commitment = [0u8; 32];
    match client.get_note_position(&test_commitment).await {
        Ok(position) => {
            println!("   ✅ get_note_position: Working (position: {})", position);
        }
        Err(e) => {
            println!("   ❌ get_note_position: Error - {}", e);
        }
    }

    // Test get_authentication_path
    let test_anchor = [0u8; 32];
    match client.get_authentication_path(0, &test_anchor).await {
        Ok(auth_path) => {
            println!(
                "   ✅ get_authentication_path: Working ({} hashes)",
                auth_path.len()
            );
        }
        Err(e) => {
            println!("   ❌ get_authentication_path: Error - {}", e);
        }
    }
}

fn print_troubleshooting_tips() {
    println!("\n🔧 Troubleshooting Tips:");
    println!("=========================");
    println!("1. Check if Zebra node is running:");
    println!("   ps aux | grep zebrad");
    println!();
    println!("2. Check Zebra configuration (~/.config/zebrad.toml):");
    println!("   [rpc]");
    println!("   listen_addr = \"127.0.0.1:8232\"");
    println!();
    println!("3. Test direct RPC connection:");
    println!("   curl -H 'content-type: application/json' \\");
    println!("        --data-binary '{{\"jsonrpc\": \"2.0\", \"method\": \"getblockcount\", \"params\": [], \"id\":1}}' \\");
    println!("        http://127.0.0.1:8232");
    println!();
    println!("4. Check if port 8232 is open:");
    println!("   netstat -tlnp | grep 8232");
    println!();
    println!("5. Check Zebra logs for errors:");
    println!("   tail -f ~/.cache/zebrad/debug.log");
    println!();
    println!("6. Verify Zebra version compatibility:");
    println!("   zebrad --version");
}
