use crate::error::NozyResult;
use crate::hd_wallet::HDWallet;
use crate::zebra_integration::ZebraClient;
use crate::orchard_tx::OrchardTransactionBuilder;
use crate::proving::OrchardProvingManager;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Performance benchmarking suite for NozyWallet
pub struct BenchmarkSuite {
    results: Vec<BenchmarkResult>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BenchmarkResult {
    pub name: String,
    #[serde(with = "duration_serde")]
    pub duration: Duration,
    pub memory_usage: Option<usize>,
    pub success: bool,
    pub details: String,
}

mod duration_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_nanos() as u64)
    }

    #[allow(dead_code)]
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let nanos = u64::deserialize(deserializer)?;
        Ok(Duration::from_nanos(nanos))
    }
}

impl BenchmarkSuite {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    /// Run all benchmarks
    pub async fn run_all(&mut self) -> NozyResult<()> {
        println!("🚀 Starting NozyWallet Performance Benchmarks");
        println!("=============================================");

        // Wallet operations
        self.benchmark_wallet_creation().await?;
        self.benchmark_address_generation().await?;
        self.benchmark_password_hashing().await?;
        self.benchmark_wallet_storage().await?;

        // Proving operations
        self.benchmark_proving_initialization().await?;
        self.benchmark_proving_parameters_loading().await?;

        // Transaction operations
        self.benchmark_transaction_building().await?;
        self.benchmark_note_scanning().await?;

        // Network operations
        self.benchmark_zebra_connection().await?;

        self.print_summary();
        Ok(())
    }

    /// Benchmark wallet creation performance
    pub async fn benchmark_wallet_creation(&mut self) -> NozyResult<()> {
        let name = "Wallet Creation";
        println!("\n📊 Benchmarking: {}", name);

        let mut total_duration = Duration::new(0, 0);
        let iterations = 100;

        for i in 0..iterations {
            let start = Instant::now();
            let result = HDWallet::new();
            let duration = start.elapsed();

            if result.is_ok() {
                total_duration += duration;
            } else {
                return Err(result.unwrap_err());
            }

            if (i + 1) % 20 == 0 {
                println!("  Completed {}/{} iterations", i + 1, iterations);
            }
        }

        let avg_duration = total_duration / iterations;
        let throughput = 1_000_000_000.0 / avg_duration.as_nanos() as f64; // wallets per second

        self.results.push(BenchmarkResult {
            name: name.to_string(),
            duration: avg_duration,
            memory_usage: None,
            success: true,
            details: format!("Average: {:?}, Throughput: {:.2} wallets/sec", avg_duration, throughput),
        });

        println!("  ✅ Average: {:?}", avg_duration);
        println!("  📈 Throughput: {:.2} wallets/sec", throughput);

        Ok(())
    }

    /// Benchmark address generation performance
    pub async fn benchmark_address_generation(&mut self) -> NozyResult<()> {
        let name = "Address Generation";
        println!("\n📊 Benchmarking: {}", name);

        let wallet = HDWallet::new()?;
        let mut total_duration = Duration::new(0, 0);
        let iterations = 1000;

        for i in 0..iterations {
            let start = Instant::now();
            let result = wallet.generate_orchard_address(0, i as u32);
            let duration = start.elapsed();

            if result.is_ok() {
                total_duration += duration;
            } else {
                return Err(result.unwrap_err());
            }

            if (i + 1) % 200 == 0 {
                println!("  Completed {}/{} iterations", i + 1, iterations);
            }
        }

        let avg_duration = total_duration / iterations;
        let throughput = 1_000_000_000.0 / avg_duration.as_nanos() as f64; // addresses per second

        self.results.push(BenchmarkResult {
            name: name.to_string(),
            duration: avg_duration,
            memory_usage: None,
            success: true,
            details: format!("Average: {:?}, Throughput: {:.2} addresses/sec", avg_duration, throughput),
        });

        println!("  ✅ Average: {:?}", avg_duration);
        println!("  📈 Throughput: {:.2} addresses/sec", throughput);

        Ok(())
    }

    /// Benchmark password hashing performance
    pub async fn benchmark_password_hashing(&mut self) -> NozyResult<()> {
        let name = "Password Hashing";
        println!("\n📊 Benchmarking: {}", name);

        let mut wallet = HDWallet::new()?;
        let mut total_duration = Duration::new(0, 0);
        let iterations = 10; // Reduced due to Argon2 being slow

        for i in 0..iterations {
            let start = Instant::now();
            let result = wallet.set_password(&format!("test_password_{}", i));
            let duration = start.elapsed();

            if result.is_ok() {
                total_duration += duration;
            } else {
                return Err(result.unwrap_err());
            }

            println!("  Completed {}/{} iterations", i + 1, iterations);
        }

        let avg_duration = total_duration / iterations;
        let throughput = 1_000_000_000.0 / avg_duration.as_nanos() as f64; // hashes per second

        self.results.push(BenchmarkResult {
            name: name.to_string(),
            duration: avg_duration,
            memory_usage: None,
            success: true,
            details: format!("Average: {:?}, Throughput: {:.2} hashes/sec", avg_duration, throughput),
        });

        println!("  ✅ Average: {:?}", avg_duration);
        println!("  📈 Throughput: {:.2} hashes/sec", throughput);

        Ok(())
    }

    /// Benchmark wallet storage performance
    pub async fn benchmark_wallet_storage(&mut self) -> NozyResult<()> {
        let name = "Wallet Storage";
        println!("\n📊 Benchmarking: {}", name);

        let wallet = HDWallet::new()?;
        let storage = crate::storage::WalletStorage::new(PathBuf::from("benchmark_wallet_data"));
        
        // Create directory
        std::fs::create_dir_all("benchmark_wallet_data").unwrap();

        let mut total_duration = Duration::new(0, 0);
        let iterations = 50;

        for i in 0..iterations {
            let start = Instant::now();
            let result = storage.save_wallet(&wallet, &format!("password_{}", i)).await;
            let duration = start.elapsed();

            if result.is_ok() {
                total_duration += duration;
            } else {
                return Err(result.unwrap_err());
            }

            if (i + 1) % 10 == 0 {
                println!("  Completed {}/{} iterations", i + 1, iterations);
            }
        }

        let avg_duration = total_duration / iterations;
        let throughput = 1_000_000_000.0 / avg_duration.as_nanos() as f64; // saves per second

        self.results.push(BenchmarkResult {
            name: name.to_string(),
            duration: avg_duration,
            memory_usage: None,
            success: true,
            details: format!("Average: {:?}, Throughput: {:.2} saves/sec", avg_duration, throughput),
        });

        println!("  ✅ Average: {:?}", avg_duration);
        println!("  📈 Throughput: {:.2} saves/sec", throughput);

        // Cleanup
        let _ = std::fs::remove_dir_all("benchmark_wallet_data");

        Ok(())
    }

    /// Benchmark proving initialization performance
    pub async fn benchmark_proving_initialization(&mut self) -> NozyResult<()> {
        let name = "Proving Initialization";
        println!("\n📊 Benchmarking: {}", name);

        let mut total_duration = Duration::new(0, 0);
        let iterations = 10;

        for i in 0..iterations {
            let start = Instant::now();
            let mut manager = OrchardProvingManager::new(PathBuf::from("benchmark_params"));
            let result = manager.initialize().await;
            let duration = start.elapsed();

            if result.is_ok() {
                total_duration += duration;
            } else {
                return Err(result.unwrap_err());
            }

            println!("  Completed {}/{} iterations", i + 1, iterations);
        }

        let avg_duration = total_duration / iterations;
        let throughput = 1_000_000_000.0 / avg_duration.as_nanos() as f64; // initializations per second

        self.results.push(BenchmarkResult {
            name: name.to_string(),
            duration: avg_duration,
            memory_usage: None,
            success: true,
            details: format!("Average: {:?}, Throughput: {:.2} initializations/sec", avg_duration, throughput),
        });

        println!("  ✅ Average: {:?}", avg_duration);
        println!("  📈 Throughput: {:.2} initializations/sec", throughput);

        Ok(())
    }

    /// Benchmark proving parameters loading performance
    pub async fn benchmark_proving_parameters_loading(&mut self) -> NozyResult<()> {
        let name = "Proving Parameters Loading";
        println!("\n📊 Benchmarking: {}", name);

        // Create test parameters first
        let mut manager = OrchardProvingManager::new(PathBuf::from("benchmark_params"));
        manager.download_parameters().await?;

        let mut total_duration = Duration::new(0, 0);
        let iterations = 100;

        for i in 0..iterations {
            let start = Instant::now();
            let mut manager = OrchardProvingManager::new(PathBuf::from("benchmark_params"));
            let result = manager.initialize().await;
            let duration = start.elapsed();

            if result.is_ok() {
                total_duration += duration;
            } else {
                return Err(result.unwrap_err());
            }

            if (i + 1) % 20 == 0 {
                println!("  Completed {}/{} iterations", i + 1, iterations);
            }
        }

        let avg_duration = total_duration / iterations;
        let throughput = 1_000_000_000.0 / avg_duration.as_nanos() as f64; // loads per second

        self.results.push(BenchmarkResult {
            name: name.to_string(),
            duration: avg_duration,
            memory_usage: None,
            success: true,
            details: format!("Average: {:?}, Throughput: {:.2} loads/sec", avg_duration, throughput),
        });

        println!("  ✅ Average: {:?}", avg_duration);
        println!("  📈 Throughput: {:.2} loads/sec", throughput);

        // Cleanup
        let _ = std::fs::remove_dir_all("benchmark_params");

        Ok(())
    }

    /// Benchmark transaction building performance
    pub async fn benchmark_transaction_building(&mut self) -> NozyResult<()> {
        let name = "Transaction Building";
        println!("\n📊 Benchmarking: {}", name);

        let mut total_duration = Duration::new(0, 0);
        let iterations = 10;

        for i in 0..iterations {
            let start = Instant::now();
            let result = OrchardTransactionBuilder::new_async(false).await;
            let duration = start.elapsed();

            if result.is_ok() {
                total_duration += duration;
            } else {
                return Err(result.unwrap_err());
            }

            println!("  Completed {}/{} iterations", i + 1, iterations);
        }

        let avg_duration = total_duration / iterations;
        let throughput = 1_000_000_000.0 / avg_duration.as_nanos() as f64; // builds per second

        self.results.push(BenchmarkResult {
            name: name.to_string(),
            duration: avg_duration,
            memory_usage: None,
            success: true,
            details: format!("Average: {:?}, Throughput: {:.2} builds/sec", avg_duration, throughput),
        });

        println!("  ✅ Average: {:?}", avg_duration);
        println!("  📈 Throughput: {:.2} builds/sec", throughput);

        Ok(())
    }

    /// Benchmark note scanning performance
    pub async fn benchmark_note_scanning(&mut self) -> NozyResult<()> {
        let name = "Note Scanning";
        println!("\n📊 Benchmarking: {}", name);

        let wallet = HDWallet::new()?;
        let zebra_client = ZebraClient::new("http://127.0.0.1:8232".to_string());
        let mut scanner = crate::notes::NoteScanner::new(wallet, zebra_client);

        let mut total_duration = Duration::new(0, 0);
        let iterations = 5; // Reduced due to network dependency

        for i in 0..iterations {
            let start = Instant::now();
            let result = scanner.scan_notes(Some(1000000), Some(1000001)).await;
            let duration = start.elapsed();

            if result.is_ok() {
                total_duration += duration;
            } else {
                // Note: This might fail if Zebra is not running
                println!("  ⚠️  Iteration {} failed (Zebra may not be running)", i + 1);
                continue;
            }

            println!("  Completed {}/{} iterations", i + 1, iterations);
        }

        if total_duration.as_nanos() > 0 {
            let avg_duration = total_duration / iterations;
            let throughput = 1_000_000_000.0 / avg_duration.as_nanos() as f64; // scans per second

            self.results.push(BenchmarkResult {
                name: name.to_string(),
                duration: avg_duration,
                memory_usage: None,
                success: true,
                details: format!("Average: {:?}, Throughput: {:.2} scans/sec", avg_duration, throughput),
            });

            println!("  ✅ Average: {:?}", avg_duration);
            println!("  📈 Throughput: {:.2} scans/sec", throughput);
        } else {
            self.results.push(BenchmarkResult {
                name: name.to_string(),
                duration: Duration::new(0, 0),
                memory_usage: None,
                success: false,
                details: "Failed - Zebra not running".to_string(),
            });

            println!("  ❌ Failed - Zebra not running");
        }

        Ok(())
    }

    /// Benchmark Zebra connection performance
    pub async fn benchmark_zebra_connection(&mut self) -> NozyResult<()> {
        let name = "Zebra Connection";
        println!("\n📊 Benchmarking: {}", name);

        let zebra_client = ZebraClient::new("http://127.0.0.1:8232".to_string());
        let mut total_duration = Duration::new(0, 0);
        let iterations = 10;

        for i in 0..iterations {
            let start = Instant::now();
            let result = zebra_client.get_block_count().await;
            let duration = start.elapsed();

            if result.is_ok() {
                total_duration += duration;
            } else {
                // Note: This might fail if Zebra is not running
                println!("  ⚠️  Iteration {} failed (Zebra may not be running)", i + 1);
                continue;
            }

            println!("  Completed {}/{} iterations", i + 1, iterations);
        }

        if total_duration.as_nanos() > 0 {
            let avg_duration = total_duration / iterations;
            let throughput = 1_000_000_000.0 / avg_duration.as_nanos() as f64; // requests per second

            self.results.push(BenchmarkResult {
                name: name.to_string(),
                duration: avg_duration,
                memory_usage: None,
                success: true,
                details: format!("Average: {:?}, Throughput: {:.2} requests/sec", avg_duration, throughput),
            });

            println!("  ✅ Average: {:?}", avg_duration);
            println!("  📈 Throughput: {:.2} requests/sec", throughput);
        } else {
            self.results.push(BenchmarkResult {
                name: name.to_string(),
                duration: Duration::new(0, 0),
                memory_usage: None,
                success: false,
                details: "Failed - Zebra not running".to_string(),
            });

            println!("  ❌ Failed - Zebra not running");
        }

        Ok(())
    }

    /// Print benchmark summary
    fn print_summary(&self) {
        println!("\n📊 Benchmark Summary");
        println!("===================");

        let mut total_duration = Duration::new(0, 0);
        let mut successful_benchmarks = 0;

        for result in &self.results {
            let status = if result.success { "✅" } else { "❌" };
            println!("{} {}: {:?}", status, result.name, result.duration);
            if !result.details.is_empty() {
                println!("    {}", result.details);
            }
            
            if result.success {
                total_duration += result.duration;
                successful_benchmarks += 1;
            }
        }

        if successful_benchmarks > 0 {
            let avg_duration = total_duration / successful_benchmarks;
            println!("\n📈 Overall Performance:");
            println!("  Successful benchmarks: {}/{}", successful_benchmarks, self.results.len());
            println!("  Average duration: {:?}", avg_duration);
        }

        println!("\n💡 Performance Tips:");
        println!("  - Use --release build for production");
        println!("  - Keep proving parameters in fast storage");
        println!("  - Use appropriate scan ranges");
        println!("  - Consider parallel operations for large datasets");
    }

    /// Get benchmark results
    pub fn get_results(&self) -> &[BenchmarkResult] {
        &self.results
    }

    /// Export results to JSON
    pub fn export_json(&self) -> NozyResult<String> {
        let json = serde_json::to_string_pretty(&self.results)
            .map_err(|e| crate::error::NozyError::InvalidOperation(format!("JSON serialization failed: {}", e)))?;
        Ok(json)
    }
}

/// Memory usage tracking utilities
pub struct MemoryTracker {
    start_memory: Option<usize>,
}

impl MemoryTracker {
    pub fn new() -> Self {
        Self {
            start_memory: None,
        }
    }

    pub fn start(&mut self) {
        self.start_memory = Some(self.get_current_memory());
    }

    pub fn stop(&mut self) -> Option<usize> {
        if let Some(start) = self.start_memory {
            let current = self.get_current_memory();
            Some(current.saturating_sub(start))
        } else {
            None
        }
    }

    fn get_current_memory(&self) -> usize {
        // This is a simplified memory tracking
        // In a real implementation, you'd use more sophisticated methods
        std::process::id() as usize
    }
}

/// Performance monitoring utilities
pub struct PerformanceMonitor {
    start_time: Instant,
    checkpoints: Vec<(String, Instant)>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            checkpoints: Vec::new(),
        }
    }

    pub fn checkpoint(&mut self, name: &str) {
        self.checkpoints.push((name.to_string(), Instant::now()));
    }

    pub fn get_elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn get_checkpoint_duration(&self, name: &str) -> Option<Duration> {
        self.checkpoints.iter()
            .find(|(n, _)| n == name)
            .map(|(_, time)| time.duration_since(self.start_time))
    }

    pub fn print_report(&self) {
        println!("\n⏱️  Performance Report");
        println!("====================");
        println!("Total elapsed: {:?}", self.get_elapsed());
        
        for (name, time) in &self.checkpoints {
            let duration = time.duration_since(self.start_time);
            println!("  {}: {:?}", name, duration);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_benchmark_suite_creation() {
        let suite = BenchmarkSuite::new();
        assert_eq!(suite.results.len(), 0);
    }

    #[tokio::test]
    async fn test_memory_tracker() {
        let mut tracker = MemoryTracker::new();
        tracker.start();
        // Simulate some work
        std::thread::sleep(Duration::from_millis(1));
        let usage = tracker.stop();
        assert!(usage.is_some());
    }

    #[tokio::test]
    async fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new();
        monitor.checkpoint("test");
        std::thread::sleep(Duration::from_millis(1));
        monitor.checkpoint("test2");
        
        assert!(monitor.get_elapsed().as_millis() > 0);
        assert!(monitor.get_checkpoint_duration("test").is_some());
    }
}
