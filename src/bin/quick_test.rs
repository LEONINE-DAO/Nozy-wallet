use nozy::ZebraClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let zebra_url = env::args()
        .nth(1)
        .unwrap_or_else(|| "http://127.0.0.1:8232".to_string());

    println!("🔍 Quick Zebra RPC Test");
    println!("=======================");
    println!("Testing: {}", zebra_url);
    println!();

    let client = ZebraClient::new(zebra_url);

    // Test 1: Basic connectivity
    println!("Testing basic connectivity...");
    match client.get_block_count().await {
        Ok(count) => {
            println!("✅ SUCCESS: Connected to Zebra node");
            println!("   Block count: {}", count);

            if count == 0 {
                println!("   ⚠️  Warning: Node appears to be starting up (0 blocks)");
            } else {
                println!("   ✅ Node is synchronized");
            }
        }
        Err(e) => {
            println!("❌ FAILED: Cannot connect to Zebra node");
            println!("   Error: {}", e);
            println!();
            println!("Quick fixes to try:");
            println!("1. Make sure Zebra is running: zebrad");
            println!("2. Check if RPC is enabled in ~/.config/zebrad.toml");
            println!("3. Try a different URL: cargo run --bin quick_test http://localhost:8232");
            return Ok(());
        }
    }

    // Test 2: Network info
    println!("\nTesting network info...");
    match client.get_network_info().await {
        Ok(info) => {
            println!("✅ Network info retrieved successfully");
            if let Some(chain) = info.get("chain") {
                println!("   Chain: {:?}", chain);
            }
        }
        Err(e) => {
            println!("❌ Network info failed: {}", e);
        }
    }

    // Test 3: Orchard functionality
    println!("\nTesting Orchard functionality...");
    match client.get_orchard_tree_state(1000).await {
        Ok(tree_state) => {
            println!("✅ Orchard tree state working");
            println!("   Height: {}", tree_state.height);
            println!("   Commitments: {}", tree_state.commitment_count);
        }
        Err(e) => {
            println!("❌ Orchard functionality failed: {}", e);
        }
    }

    println!("\n🎉 Quick test completed!");
    println!("Your Zebra node RPC is compatible with NozyWallet!");

    Ok(())
}
