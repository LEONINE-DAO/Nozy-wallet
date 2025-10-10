use nozy::RpcTester;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 NozyWallet RPC Test Suite");
    println!("============================\n");

    // Get Zebra URL from command line or use default
    let zebra_url = env::args()
        .nth(1)
        .unwrap_or_else(|| "http://127.0.0.1:8232".to_string());

    println!("🔗 Connecting to Zebra node at: {}", zebra_url);
    println!();

    // Create RPC tester
    let tester = RpcTester::new(zebra_url);

    // Run all tests
    match tester.run_all_tests().await {
        Ok(_) => {
            println!("\n🎉 All tests passed successfully!");
            println!("Your Zebra RPC integration is working correctly.");
        },
        Err(e) => {
            println!("\n❌ Tests failed: {}", e);
            println!("\nTroubleshooting tips:");
            println!("1. Make sure Zebra node is running");
            println!("2. Check that RPC is enabled in zebrad.toml:");
            println!("   [rpc]");
            println!("   listen_addr = \"127.0.0.1:8232\"");
            println!("3. Verify the node is synchronized");
            std::process::exit(1);
        }
    }

    Ok(())
}
