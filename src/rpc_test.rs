use crate::error::NozyResult;
use crate::zebra_integration::ZebraClient;

pub struct RpcTester {
    client: ZebraClient,
}

impl RpcTester {
    pub fn new(zebra_url: String) -> Self {
        Self {
            client: ZebraClient::new(zebra_url),
        }
    }

    pub async fn test_connectivity(&self) -> NozyResult<()> {
        println!("🔍 Testing Zebra RPC connectivity...");

        match self.client.get_block_count().await {
            Ok(count) => println!("✅ get_block_count: {} blocks", count),
            Err(e) => println!("❌ get_block_count failed: {}", e),
        }

        match self.client.get_network_info().await {
            Ok(info) => println!("✅ get_network_info: {:?}", info),
            Err(e) => println!("❌ get_network_info failed: {}", e),
        }

        match self.client.get_mempool_info().await {
            Ok(info) => println!("✅ get_mempool_info: {:?}", info),
            Err(e) => println!("❌ get_mempool_info failed: {}", e),
        }

        Ok(())
    }

    pub async fn test_orchard_functionality(&self) -> NozyResult<()> {
        println!("🌳 Testing Orchard functionality...");

        let height = self.client.get_best_block_height().await?;
        println!("📊 Current block height: {}", height);

        match self.client.get_orchard_tree_state(height).await {
            Ok(tree_state) => {
                println!("✅ get_orchard_tree_state (z_gettreestate):");
                println!("   Height: {}", tree_state.height);
                println!("   Anchor: {}", hex::encode(tree_state.anchor));
                println!("   Commitment count: {}", tree_state.commitment_count);
            }
            Err(e) => println!("❌ get_orchard_tree_state failed: {}", e),
        }

        match self
            .client
            .z_get_subtrees_by_index("orchard", 0, Some(2))
            .await
        {
            Ok(sub) => {
                println!(
                    "✅ z_getsubtreesbyindex(orchard): pool={} subtrees={}",
                    sub.pool,
                    sub.subtrees.len()
                );
            }
            Err(e) => println!("❌ z_getsubtreesbyindex failed: {}", e),
        }

        Ok(())
    }

    pub async fn test_transaction_functionality(&self) -> NozyResult<()> {
        println!("💸 Testing transaction functionality...");

        match self.client.get_txout_set_info().await {
            Ok(info) => println!("✅ get_txout_set_info: {:?}", info),
            Err(e) => println!("❌ get_txout_set_info failed: {}", e),
        }

        match self.client.get_fee_estimate().await {
            Ok(estimate) => println!("✅ get_fee_estimate: {:?}", estimate),
            Err(e) => println!("❌ get_fee_estimate failed: {}", e),
        }

        Ok(())
    }

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
