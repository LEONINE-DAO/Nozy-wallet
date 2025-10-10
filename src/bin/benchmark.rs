use nozy::benchmarks::BenchmarkSuite;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 NozyWallet Performance Benchmark Suite");
    println!("=========================================");

    let args: Vec<String> = env::args().collect();
    let mut suite = BenchmarkSuite::new();

    if args.len() > 1 {
        match args[1].as_str() {
            "wallet" => {
                println!("Running wallet-specific benchmarks...");
                suite.benchmark_wallet_creation().await?;
                suite.benchmark_address_generation().await?;
                suite.benchmark_password_hashing().await?;
                suite.benchmark_wallet_storage().await?;
            },
            "proving" => {
                println!("Running proving-specific benchmarks...");
                suite.benchmark_proving_initialization().await?;
                suite.benchmark_proving_parameters_loading().await?;
            },
            "network" => {
                println!("Running network-specific benchmarks...");
                suite.benchmark_zebra_connection().await?;
                suite.benchmark_note_scanning().await?;
            },
            "transaction" => {
                println!("Running transaction-specific benchmarks...");
                suite.benchmark_transaction_building().await?;
            },
            "json" => {
                println!("Running all benchmarks and exporting to JSON...");
                suite.run_all().await?;
                let json = suite.export_json()?;
                std::fs::write("benchmark_results.json", json)?;
                println!("Results exported to benchmark_results.json");
            },
            _ => {
                println!("Unknown benchmark category: {}", args[1]);
                println!("Available categories: wallet, proving, network, transaction, json");
                return Ok(());
            }
        }
    } else {
        // Run all benchmarks
        suite.run_all().await?;
    }

    Ok(())
}
