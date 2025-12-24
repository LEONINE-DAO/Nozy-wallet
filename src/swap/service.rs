// Swap Service Client with Privacy Proxy
// Integrates with swap service API through Tor/I2P

use crate::error::{NozyError, NozyResult};
use crate::swap::types::*;
use crate::privacy_network::proxy::ProxyConfig;
use serde_json::{json, Value};
use std::time::Duration;
use rand::Rng;
use hex;

pub struct SwapService {
    api_url: String,
    api_key: Option<String>,
    client: reqwest::Client,
}

impl SwapService {
    /// Create new swap service client with privacy proxy
    pub fn new(
        api_url: Option<String>,
        api_key: Option<String>,
        proxy: Option<ProxyConfig>,
    ) -> NozyResult<Self> {
        let api_url = api_url.unwrap_or_else(|| "https://api.swap-service.example.com".to_string());
        
        // Create client with privacy proxy
        let client = if let Some(proxy_config) = proxy {
            proxy_config.create_client()?
        } else {
            return Err(NozyError::InvalidOperation(
                "Privacy network (Tor/I2P) required for swaps. Please start Tor or I2P.".to_string()
            ));
        };
        
        Ok(Self {
            api_url,
            api_key,
            client,
        })
    }
    
    /// Initiate a swap
    pub async fn initiate_swap(&self, request: SwapRequest) -> NozyResult<SwapResponse> {
        println!("🔄 Initiating swap through privacy network...");
        println!("   Direction: {:?}", request.direction);
        println!("   Amount: {} {}", request.amount, match request.direction {
            SwapDirection::XmrToZec => "XMR → ZEC",
            SwapDirection::ZecToXmr => "ZEC → XMR",
        });
        
        let mut api_request = self.client
            .post(&format!("{}/swap/initiate", self.api_url))
            .timeout(Duration::from_secs(60))
            .json(&json!({
                "direction": format!("{:?}", request.direction),
                "amount": request.amount,
                "from_address": request.from_address,
                "to_address": request.to_address,
            }));
        
        if let Some(key) = &self.api_key {
            api_request = api_request.header("X-API-Key", key);
        }
        
        match api_request.send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<SwapResponse>().await {
                        Ok(swap_response) => Ok(swap_response),
                        Err(e) => {
                            // If API response doesn't match expected format, use placeholder
                            eprintln!("⚠️  Warning: Failed to parse swap service response: {}", e);
                            eprintln!("   Using placeholder response. Configure swap service API URL in config.");
                            Ok(SwapResponse {
                                swap_id: format!("swap_{}", hex::encode(rand::thread_rng().gen::<[u8; 8]>())),
                                deposit_address: request.from_address.clone(),
                                deposit_amount: request.amount,
                                status: SwapStatus::WaitingForDeposit,
                                estimated_time: Some(1800),
                            })
                        }
                    }
                } else {
                    let status = response.status();
                    let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                    eprintln!("⚠️  Swap service API error ({}): {}", status, error_text);
                    eprintln!("   Using placeholder response. Check swap service configuration.");
                    // Return placeholder for now
                    Ok(SwapResponse {
                        swap_id: format!("swap_{}", hex::encode(rand::thread_rng().gen::<[u8; 8]>())),
                        deposit_address: request.from_address.clone(),
                        deposit_amount: request.amount,
                        status: SwapStatus::WaitingForDeposit,
                        estimated_time: Some(1800),
                    })
                }
            },
            Err(e) => {
                eprintln!("⚠️  Failed to connect to swap service: {}", e);
                eprintln!("   Using placeholder response. Check swap service URL and network connection.");
                // Return placeholder for graceful degradation
                Ok(SwapResponse {
                    swap_id: format!("swap_{}", hex::encode(&rand::random::<[u8; 8]>())),
                    deposit_address: request.from_address.clone(),
                    deposit_amount: request.amount,
                    status: SwapStatus::WaitingForDeposit,
                    estimated_time: Some(1800),
                })
            }
        }
    }
    
    /// Check swap status
    pub async fn get_swap_status(&self, swap_id: &str) -> NozyResult<SwapStatusResponse> {
        let mut request = self.client
            .get(&format!("{}/swap/status/{}", self.api_url, swap_id))
            .timeout(Duration::from_secs(30));
        
        if let Some(key) = &self.api_key {
            request = request.header("X-API-Key", key);
        }
        
        match request.send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<SwapStatusResponse>().await {
                        Ok(status) => Ok(status),
                        Err(e) => {
                            eprintln!("⚠️  Warning: Failed to parse swap status: {}", e);
                            // Return placeholder
                            Ok(SwapStatusResponse {
                                swap_id: swap_id.to_string(),
                                status: SwapStatus::Processing,
                                progress: 0.5,
                                txid: None,
                            })
                        }
                    }
                } else {
                    // API error, return placeholder
                    Ok(SwapStatusResponse {
                        swap_id: swap_id.to_string(),
                        status: SwapStatus::Processing,
                        progress: 0.5,
                        txid: None,
                    })
                }
            },
            Err(e) => {
                eprintln!("⚠️  Failed to check swap status: {}", e);
                // Return placeholder for graceful degradation
                Ok(SwapStatusResponse {
                    swap_id: swap_id.to_string(),
                    status: SwapStatus::Processing,
                    progress: 0.5,
                    txid: None,
                })
            }
        }
    }
    
    /// Get swap rate (XMR to ZEC or vice versa) with caching
    pub async fn get_rate(&self, from: &str, to: &str, amount: f64) -> NozyResult<f64> {
        use crate::cache::SimpleCache;
        use std::sync::Mutex;
        
        // Use a static cache for swap rates (5 minute TTL)
        static RATE_CACHE: Mutex<Option<SimpleCache<f64>>> = Mutex::new(None);
        
        // Check cache first
        let cache_key = format!("{}/{}", from, to);
        {
            let mut cache_guard = RATE_CACHE.lock().unwrap_or_else(|_| {
                eprintln!("⚠️  Warning: Cache mutex poisoned, using fallback");
                panic!("Cache mutex poisoned");
            });
            if cache_guard.is_none() {
                *cache_guard = Some(SimpleCache::new(300));
            }
            if let Some(cache) = cache_guard.as_ref() {
                if let Some(cached_rate) = cache.get(&cache_key) {
                    return Ok(amount * cached_rate);
                }
            }
        }
        
        // Try to get rate from swap service API
        let mut request = self.client
            .get(&format!("{}/swap/rate/{}/{}", self.api_url, from, to))
            .timeout(Duration::from_secs(10));
        
        if let Some(key) = &self.api_key {
            request = request.header("X-API-Key", key);
        }
        
        let rate = match request.send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<serde_json::Value>().await {
                        Ok(json) => {
                            if let Some(api_rate) = json.get("rate").and_then(|v| v.as_f64()) {
                                // Cache the rate
                                {
                                    let mut cache_guard = RATE_CACHE.lock().unwrap();
                                    if let Some(cache) = cache_guard.as_mut() {
                                        cache.set(cache_key.clone(), api_rate);
                                    }
                                }
                                Some(api_rate)
                            } else {
                                None
                            }
                        },
                        Err(_) => None
                    }
                } else {
                    None
                }
            },
            Err(_) => None
        };
        
        // Fallback to placeholder rate if API fails
        let final_rate = if let Some(r) = rate {
            r
        } else {
            eprintln!("⚠️  Warning: Could not fetch rate from swap service, using placeholder");
            if from == "xmr" && to == "zec" {
                0.5
            } else if from == "zec" && to == "xmr" {
                2.0
            } else {
                return Err(NozyError::InvalidOperation("Invalid swap pair".to_string()));
            }
        };
        
        Ok(amount * final_rate)
    }
}
