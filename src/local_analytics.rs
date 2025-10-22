use crate::error::{NozyError, NozyResult};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Privacy-focused local analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAnalytics {
    pub commands_executed: HashMap<String, u64>,
    pub errors_encountered: HashMap<String, u64>,
    pub performance_metrics: Vec<PerformanceMetric>,
    
    pub session_count: u64,
    pub first_use: u64,
    pub last_use: u64,
    
    pub features_used: HashMap<String, u64>,
    
    pub platform: String,
    pub rust_version: String,
    pub wallet_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    pub operation: String,
    pub duration_ms: u64,
    pub memory_usage_kb: u64,
    pub success: bool,
    pub timestamp: u64,
}

impl LocalAnalytics {
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        Self {
            commands_executed: HashMap::new(),
            errors_encountered: HashMap::new(),
            performance_metrics: Vec::new(),
            session_count: 0,
            first_use: now,
            last_use: now,
            features_used: HashMap::new(),
            platform: Self::get_platform_info(),
            rust_version: Self::get_rust_version(),
            wallet_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
    
    pub fn track_command(&mut self, command: &str) {
        *self.commands_executed.entry(command.to_string()).or_insert(0) += 1;
        self.update_last_use();
    }
    
    pub fn track_error(&mut self, error_type: &str) {
        *self.errors_encountered.entry(error_type.to_string()).or_insert(0) += 1;
    }
    
    pub fn track_feature(&mut self, feature: &str) {
        *self.features_used.entry(feature.to_string()).or_insert(0) += 1;
    }
    
    pub fn track_performance(&mut self, operation: &str, duration_ms: u64, memory_kb: u64, success: bool) {
        let metric = PerformanceMetric {
            operation: operation.to_string(),
            duration_ms,
            memory_usage_kb: memory_kb,
            success,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        self.performance_metrics.push(metric);
        
        if self.performance_metrics.len() > 1000 {
            self.performance_metrics.drain(0..100);
        }
    }
    
    pub fn start_session(&mut self) {
        self.session_count += 1;
        self.update_last_use();
    }
    
    pub fn save_to_file(&self, path: &PathBuf) -> NozyResult<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| NozyError::Storage(format!("Failed to serialize analytics: {}", e)))?;
        
        fs::write(path, json)
            .map_err(|e| NozyError::Storage(format!("Failed to write analytics file: {}", e)))?;
        
        Ok(())
    }
    
    pub fn load_from_file(path: &PathBuf) -> NozyResult<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }
        
        let content = fs::read_to_string(path)
            .map_err(|e| NozyError::Storage(format!("Failed to read analytics file: {}", e)))?;
        
        let analytics: Self = serde_json::from_str(&content)
            .map_err(|e| NozyError::Storage(format!("Failed to parse analytics: {}", e)))?;
        
        Ok(analytics)
    }
    
    pub fn generate_summary(&self) -> String {
        let mut summary = String::new();
        
        summary.push_str("📊 NozyWallet Local Analytics Summary\n");
        summary.push_str("=====================================\n\n");
        
        summary.push_str(&format!("Sessions: {}\n", self.session_count));
        summary.push_str(&format!("First Use: {}\n", self.format_timestamp(self.first_use)));
        summary.push_str(&format!("Last Use: {}\n", self.format_timestamp(self.last_use)));
        summary.push_str(&format!("Platform: {}\n", self.platform));
        summary.push_str(&format!("Rust Version: {}\n", self.rust_version));
        summary.push_str(&format!("Wallet Version: {}\n\n", self.wallet_version));
        
        summary.push_str("Most Used Commands:\n");
        let mut commands: Vec<_> = self.commands_executed.iter().collect();
        commands.sort_by(|a, b| b.1.cmp(a.1));
        for (cmd, count) in commands.iter().take(5) {
            summary.push_str(&format!("  {}: {} times\n", cmd, count));
        }
        
        summary.push_str("\nFeature Usage:\n");
        let mut features: Vec<_> = self.features_used.iter().collect();
        features.sort_by(|a, b| b.1.cmp(a.1));
        for (feature, count) in features.iter().take(5) {
            summary.push_str(&format!("  {}: {} times\n", feature, count));
        }
        
        if !self.errors_encountered.is_empty() {
            summary.push_str("\nCommon Errors:\n");
            let mut errors: Vec<_> = self.errors_encountered.iter().collect();
            errors.sort_by(|a, b| b.1.cmp(a.1));
            for (error, count) in errors.iter().take(3) {
                summary.push_str(&format!("  {}: {} times\n", error, count));
            }
        }
        
        summary.push_str("\n💡 This data stays on your device - no external collection!\n");
        
        summary
    }
    
    pub fn export_anonymized(&self) -> NozyResult<String> {
        let anonymized = AnonymizedData {
            total_sessions: self.session_count,
            most_used_commands: self.get_top_commands(5),
            feature_adoption: self.get_top_features(5),
            error_frequency: self.get_top_errors(3),
            platform_distribution: self.platform.clone(),
            performance_avg: self.calculate_avg_performance(),
        };
        
        serde_json::to_string_pretty(&anonymized)
            .map_err(|e| NozyError::Storage(format!("Failed to serialize anonymized data: {}", e)))
    }
    
    fn update_last_use(&mut self) {
        self.last_use = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
    
    fn format_timestamp(&self, timestamp: u64) -> String {
        format!("{}", timestamp)
    }
    
    fn get_platform_info() -> String {
        #[cfg(target_os = "windows")]
        return "Windows".to_string();
        #[cfg(target_os = "macos")]
        return "macOS".to_string();
        #[cfg(target_os = "linux")]
        return "Linux".to_string();
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        return "Unknown".to_string();
    }
    
    fn get_rust_version() -> String {
        std::env::var("RUSTC_SEMVER").unwrap_or_else(|_| "unknown".to_string())
    }
    
    fn get_top_commands(&self, limit: usize) -> Vec<(String, u64)> {
        let mut commands: Vec<_> = self.commands_executed.iter().collect();
        commands.sort_by(|a, b| b.1.cmp(a.1));
        commands.into_iter().take(limit).map(|(k, v)| (k.clone(), *v)).collect()
    }
    
    fn get_top_features(&self, limit: usize) -> Vec<(String, u64)> {
        let mut features: Vec<_> = self.features_used.iter().collect();
        features.sort_by(|a, b| b.1.cmp(a.1));
        features.into_iter().take(limit).map(|(k, v)| (k.clone(), *v)).collect()
    }
    
    fn get_top_errors(&self, limit: usize) -> Vec<(String, u64)> {
        let mut errors: Vec<_> = self.errors_encountered.iter().collect();
        errors.sort_by(|a, b| b.1.cmp(a.1));
        errors.into_iter().take(limit).map(|(k, v)| (k.clone(), *v)).collect()
    }
    
    fn calculate_avg_performance(&self) -> f64 {
        if self.performance_metrics.is_empty() {
            return 0.0;
        }
        
        let total_duration: u64 = self.performance_metrics.iter().map(|m| m.duration_ms).sum();
        total_duration as f64 / self.performance_metrics.len() as f64
    }
}

/// Anonymized data structure for optional development insights
#[derive(Debug, Serialize, Deserialize)]
struct AnonymizedData {
    total_sessions: u64,
    most_used_commands: Vec<(String, u64)>,
    feature_adoption: Vec<(String, u64)>,
    error_frequency: Vec<(String, u64)>,
    platform_distribution: String,
    performance_avg: f64,
}

impl Default for LocalAnalytics {
    fn default() -> Self {
        Self::new()
    }
}
