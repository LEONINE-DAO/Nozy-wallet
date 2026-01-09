use axum::{
    extract::Request,
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json as ResponseJson, Response},
};
use std::time::Instant;
use tracing::{info, warn};

pub async fn security_headers(request: Request, next: Next) -> Response {
    let response = next.run(request).await;

    let mut headers = HeaderMap::new();

    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("X-Frame-Options", HeaderValue::from_static("DENY"));
    headers.insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block"),
    );
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    headers.insert(
        "Content-Security-Policy",
        HeaderValue::from_static("default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self'"),
    );

    headers.insert(
        "Permissions-Policy",
        HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
    );

    let mut response = response;
    response.headers_mut().extend(headers);
    response
}

pub async fn request_logging(request: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path();

    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .or_else(|| request.headers().get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    info!(
        method = %method,
        path = %path,
        client_ip = %client_ip,
        "Incoming request"
    );

    let response = next.run(request).await;
    let duration = start.elapsed();
    let status = response.status();

    if status.is_success() {
        info!(
            method = %method,
            path = %path,
            status = %status.as_u16(),
            duration_ms = duration.as_millis(),
            "Request completed"
        );
    } else {
        warn!(
            method = %method,
            path = %path,
            status = %status.as_u16(),
            duration_ms = duration.as_millis(),
            "Request failed"
        );
    }

    response
}

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

#[derive(Clone)]
struct RateLimitEntry {
    count: u32,
    reset_at: SystemTime,
}

#[derive(Clone)]
pub struct RateLimiter {
    limits: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    max_requests: u32,
    window_seconds: u64,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_seconds: u64) -> Self {
        Self {
            limits: Arc::new(RwLock::new(HashMap::new())),
            max_requests,
            window_seconds,
        }
    }

    pub async fn check_rate_limit(&self, client_ip: &str) -> Result<(u32, u32), StatusCode> {
        let mut limits = self.limits.write().await;
        let now = SystemTime::now();

        limits.retain(|_, entry| entry.reset_at > now);

        let entry = limits
            .entry(client_ip.to_string())
            .or_insert_with(|| RateLimitEntry {
                count: 0,
                reset_at: now + Duration::from_secs(self.window_seconds),
            });

        if entry.reset_at <= now {
            entry.count = 0;
            entry.reset_at = now + Duration::from_secs(self.window_seconds);
        }

        if entry.count >= self.max_requests {
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }

        entry.count += 1;
        let remaining = self.max_requests.saturating_sub(entry.count);
        Ok((self.max_requests, remaining))
    }
}

pub async fn rate_limit_middleware(request: Request, next: Next, limiter: RateLimiter) -> Response {
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .or_else(|| request.headers().get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    match limiter.check_rate_limit(&client_ip).await {
        Ok((limit, remaining)) => {
            let mut response = next.run(request).await;
            response.headers_mut().insert(
                "X-RateLimit-Limit",
                HeaderValue::from_str(&limit.to_string())
                    .unwrap_or(HeaderValue::from_static("100")),
            );
            response.headers_mut().insert(
                "X-RateLimit-Remaining",
                HeaderValue::from_str(&remaining.to_string())
                    .unwrap_or(HeaderValue::from_static("0")),
            );
            response
        }
        Err(status) => {
            warn!(client_ip = %client_ip, "Rate limit exceeded");
            let mut response = (
                status,
                ResponseJson(serde_json::json!({
                    "error": "Rate limit exceeded. Please try again later."
                })),
            )
                .into_response();
            response
                .headers_mut()
                .insert("X-RateLimit-Limit", HeaderValue::from_static("100"));
            response
                .headers_mut()
                .insert("X-RateLimit-Remaining", HeaderValue::from_static("0"));
            response
        }
    }
}

pub async fn api_key_auth(
    request: Request,
    next: Next,
    expected_api_key: Option<String>,
) -> Response {
    let Some(ref api_key) = expected_api_key else {
        return next.run(request).await;
    };

    let provided_key = request
        .headers()
        .get("X-API-Key")
        .or_else(|| request.headers().get("Authorization"))
        .and_then(|h| h.to_str().ok())
        .map(|s| {
            if s.starts_with("Bearer ") {
                s.strip_prefix("Bearer ").unwrap_or(s).to_string()
            } else {
                s.to_string()
            }
        });

    match provided_key {
        Some(key) if key == *api_key => next.run(request).await,
        Some(_) => {
            warn!("Invalid API key provided");
            (
                StatusCode::UNAUTHORIZED,
                ResponseJson(serde_json::json!({
                    "error": "Invalid API key"
                })),
            )
                .into_response()
        }
        None => {
            warn!("Missing API key");
            (
                StatusCode::UNAUTHORIZED,
                ResponseJson(serde_json::json!({
                    "error": "API key required. Provide it in X-API-Key header or Authorization: Bearer <key>"
                })),
            )
                .into_response()
        }
    }
}
