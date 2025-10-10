use crate::zebra_integration::ZebraClient;
use crate::error::NozyResult;

/// Test module for Zebra RPC functionality
pub struct RpcTester {
    client: ZebraClient,
}

impl RpcTester {
    pub fn new(zebra_url: String) -> Self {
        Self {
            client: ZebraClient::new(zebra_url),
        }
    }

    /// Test basic RPC connectivity
    pub async fn test_connectivity(&self) -> NozyResult<()> {
        println!("🔍 Testing Zebra RPC connectivity...");
        
        // Test get_block_count
        match self.client.get_block_count().await {
            Ok(count) => println!("✅ get_block_count: {} blocks", count),
            Err(e) => println!("❌ get_block_count failed: {}", e),
        }

        // Test get_network_info
        match self.client.get_network_info().await {
            Ok(info) => println!("✅ get_network_info: {:?}", info),
            Err(e) => println!("❌ get_network_info failed: {}", e),
        }

        // Test get_mempool_info
        match self.client.get_mempool_info().await {
            Ok(info) => println!("✅ get_mempool_info: {:?}", info),
            Err(e) => println!("❌ get_mempool_info failed: {}", e),
        }

        Ok(())
    }

    /// Test Orchard-specific functionality
    pub async fn test_orchard_functionality(&self) -> NozyResult<()> {
        println!("🌳 Testing Orchard functionality...");
        
        // Get current block height
        let height = self.client.get_best_block_height().await?;
        println!("📊 Current block height: {}", height);

        // Test get_orchard_tree_state
        match self.client.get_orchard_tree_state(height).await {
            Ok(tree_state) => {
                println!("✅ get_orchard_tree_state:");
                println!("   Height: {}", tree_state.height);
                println!("   Anchor: {}", hex::encode(tree_state.anchor));
                println!("   Commitment count: {}", tree_state.commitment_count);
            },
            Err(e) => println!("❌ get_orchard_tree_state failed: {}", e),
        }

        // Test get_note_position
        let test_commitment = [0u8; 32];
        match self.client.get_note_position(&test_commitment).await {
            Ok(position) => println!("✅ get_note_position: {}", position),
            Err(e) => println!("❌ get_note_position failed: {}", e),
        }

        // Test get_authentication_path
        let test_anchor = [0u8; 32];
        match self.client.get_authentication_path(0, &test_anchor).await {
            Ok(auth_path) => println!("✅ get_authentication_path: {} hashes", auth_path.len()),
            Err(e) => println!("❌ get_authentication_path failed: {}", e),
        }

        Ok(())
    }

    /// Test transaction-related functionality
    pub async fn test_transaction_functionality(&self) -> NozyResult<()> {
        println!("💸 Testing transaction functionality...");
        
        // Test get_txout_set_info
        match self.client.get_txout_set_info().await {
            Ok(info) => println!("✅ get_txout_set_info: {:?}", info),
            Err(e) => println!("❌ get_txout_set_info failed: {}", e),
        }

        // Test get_fee_estimate
        match self.client.get_fee_estimate().await {
            Ok(estimate) => println!("✅ get_fee_estimate: {:?}", estimate),
            Err(e) => println!("❌ get_fee_estimate failed: {}", e),
        }

        Ok(())
    }

    /// Run all tests
    pub async fn run_all_tests(&self) -> NozyResult<()> {
        println!("🚀 Running comprehensive Zebra RPC tests...\n");
        
        self.test_connectivity().await?;
        println!();
        
        self.test_orchard_functionality().await?;
        println!();
        
        self.test_transaction_functionality().await?;
        println!();
        
        println!("✅ All RPC tests completed!");
        Ok(())
    }
}
