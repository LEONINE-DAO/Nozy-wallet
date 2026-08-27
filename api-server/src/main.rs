use axum::http::HeaderValue;
use axum::{
    http::{header, Method},
    middleware::from_fn,
    response::{IntoResponse, Json as ResponseJson},
    routing::{delete, get, post},
    Router,
};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

mod handlers;
mod invoice_handlers;
mod ironwood_handlers;
mod keystone_handlers;
mod lwd_handlers;
mod middleware;
mod profile_handlers;
mod sapling_handlers;
mod vote_handlers;
mod zns_handlers;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let api_key = std::env::var("NOZY_API_KEY").ok();
    let rate_limit_requests = std::env::var("NOZY_RATE_LIMIT_REQUESTS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);
    let rate_limit_window = std::env::var("NOZY_RATE_LIMIT_WINDOW")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(60);
    let is_production = std::env::var("NOZY_PRODUCTION").is_ok();

    if api_key.is_some() {
        info!("API key authentication enabled");
    } else {
        warn!("⚠️  API key authentication is DISABLED - set NOZY_API_KEY environment variable to enable");
    }

    info!(
        "Rate limiting: {} requests per {} seconds",
        rate_limit_requests, rate_limit_window
    );
    info!(
        "Environment: {}",
        if is_production {
            "PRODUCTION"
        } else {
            "DEVELOPMENT"
        }
    );

    tokio::task::spawn_blocking(nozy::warm_orchard_proving_key);
    info!("Orchard proving warm-up started in background");

    // HTTPS configuration
    let https_enabled = std::env::var("NOZY_HTTPS_ENABLED").is_ok();
    let https_port: u16 = std::env::var("NOZY_HTTPS_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(443);
    let http_port: u16 = std::env::var("NOZY_HTTP_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);

    // F-03: default loopback. Set NOZY_BIND_ADDR=0.0.0.0 only for intentional LAN/hosted.
    // Must run before middleware captures `api_key`.
    let bind_host = std::env::var("NOZY_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1".to_string());
    let bind_is_all_interfaces = bind_host == "0.0.0.0" || bind_host == "::" || bind_host == "[::]";

    // Locked-down local: NOZY_PRODUCTION requires an API key even on loopback so
    // create/restore/unlock (and all other routes behind api_key_auth) are not open.
    if is_production && api_key.is_none() {
        return Err(anyhow::anyhow!(
            "NOZY_PRODUCTION is set but NOZY_API_KEY is missing. \
             Set NOZY_API_KEY for create/restore/unlock and other authenticated routes, \
             or unset NOZY_PRODUCTION for local development."
        ));
    }

    if bind_is_all_interfaces {
        warn!(
            "Binding to {} — API is reachable on all interfaces. Prefer 127.0.0.1 unless hosted.",
            bind_host
        );
        if api_key.is_none() {
            return Err(anyhow::anyhow!(
                "Refusing to bind {bind_host} without NOZY_API_KEY. \
                 Seed/restore routes must not be LAN-exposed unauthenticated. \
                 Set NOZY_API_KEY, or use NOZY_BIND_ADDR=127.0.0.1 (default)."
            ));
        }
    }

    let app = Router::new()
        .route("/api/wallet/exists", get(handlers::check_wallet_exists))
        .route("/api/wallet/create", post(handlers::create_wallet))
        .route("/api/wallet/restore", post(handlers::restore_wallet))
        .route("/api/wallet/unlock", post(handlers::unlock_wallet))
        .route("/api/wallet/status", get(handlers::get_wallet_status))
        .route(
            "/api/ironwood/status",
            get(ironwood_handlers::get_ironwood_status),
        )
        .route(
            "/api/ironwood/plan",
            post(ironwood_handlers::ironwood_plan_save),
        )
        .route(
            "/api/ironwood/split",
            post(ironwood_handlers::ironwood_split),
        )
        .route(
            "/api/ironwood/migrate",
            post(ironwood_handlers::ironwood_migrate),
        )
        .route(
            "/api/ironwood/broadcast",
            post(ironwood_handlers::ironwood_broadcast),
        )
        .route(
            "/api/sapling/status",
            get(sapling_handlers::get_sapling_status),
        )
        .route("/api/sapling/scan", post(sapling_handlers::scan_sapling))
        .route(
            "/api/sapling/shield",
            post(sapling_handlers::shield_sapling),
        )
        .route("/api/vote/status", get(vote_handlers::vote_status))
        .route("/api/vote/active", get(vote_handlers::vote_active))
        .route(
            "/api/vote/export-notes",
            post(vote_handlers::vote_export_notes),
        )
        .route("/api/vote/prepare", post(vote_handlers::vote_prepare))
        .route("/api/vote/delegate", post(vote_handlers::vote_delegate))
        .route(
            "/api/vote/sign-delegation",
            post(vote_handlers::vote_sign_delegation),
        )
        .route(
            "/api/vote/delegate-finish",
            post(vote_handlers::vote_delegate_finish),
        )
        .route("/api/vote/cast", post(vote_handlers::vote_cast))
        .route("/api/zns/resolve", post(zns_handlers::resolve_zns_name))
        .route("/api/zns/link", get(zns_handlers::get_zns_link))
        .route("/api/zns/link", post(zns_handlers::link_zns_name))
        .route("/api/zns/link", delete(zns_handlers::unlink_zns_name))
        .route("/api/zns/reverse", get(zns_handlers::reverse_zns_lookup))
        .route("/api/profile", get(profile_handlers::get_profile))
        .route("/api/profile", post(profile_handlers::update_profile))
        .route(
            "/api/business/invoices",
            post(invoice_handlers::create_invoice),
        )
        .route(
            "/api/business/invoices",
            get(invoice_handlers::list_invoices),
        )
        .route(
            "/api/business/invoices/{id}",
            get(invoice_handlers::get_invoice),
        )
        .route(
            "/api/business/invoices/{id}/qr",
            get(invoice_handlers::get_invoice_qr),
        )
        .route(
            "/api/business/invoices/{id}/cancel",
            post(invoice_handlers::cancel_invoice),
        )
        .route("/api/pilot/metrics", get(handlers::get_pilot_metrics))
        .route("/api/address/generate", post(handlers::generate_address))
        .route("/api/balance", get(handlers::get_balance))
        .route("/api/sync", post(handlers::sync_wallet))
        .route("/api/transaction/send", post(handlers::send_transaction))
        .route("/api/transaction/fee-estimate", get(handlers::estimate_fee))
        .route(
            "/api/transaction/history",
            get(handlers::get_transaction_history),
        )
        .route("/api/transaction/{txid}", get(handlers::get_transaction))
        .route(
            "/api/transaction/check-confirmations",
            post(handlers::check_transaction_confirmations),
        )
        .route(
            "/api/transaction/speed-up",
            post(handlers::speed_up_transaction),
        )
        .route("/api/address-book", get(handlers::list_address_book))
        .route("/api/address-book", post(handlers::add_address_book_entry))
        .route(
            "/api/address-book/{name}",
            delete(handlers::remove_address_book_entry),
        )
        .route(
            "/api/address-book/search",
            get(handlers::search_address_book),
        )
        .route("/api/notes", get(handlers::get_notes))
        .route("/api/config", get(handlers::get_config))
        .route("/api/config/zebra-url", post(handlers::set_zebra_url))
        .route("/api/config/theme", post(handlers::set_theme))
        .route(
            "/api/config/test-zebra",
            post(handlers::test_zebra_connection),
        )
        .route("/api/chain/block-count", get(handlers::chain_block_count))
        .route("/api/chain/block/{height}", get(handlers::chain_block))
        .route(
            "/api/transaction/broadcast",
            post(handlers::broadcast_raw_transaction),
        )
        .route("/api/proving/status", get(handlers::check_proving_status))
        .route(
            "/api/proving/download",
            post(handlers::download_proving_parameters),
        )
        .route("/api/web/me", get(handlers::web_me))
        .route("/api/web/read-state", get(handlers::web_read_state))
        .route("/api/web/privacy-status", get(handlers::web_privacy_status))
        .route("/api/web/node-status", get(handlers::web_node_status))
        .route("/api/lwd/info", get(lwd_handlers::lwd_info))
        .route("/api/lwd/chain-tip", get(lwd_handlers::lwd_chain_tip))
        .route(
            "/api/lwd/sync/compact",
            post(lwd_handlers::lwd_sync_compact),
        )
        .route(
            "/api/lwd/sync/compact-to-tip",
            post(lwd_handlers::lwd_sync_compact_to_tip),
        )
        .route(
            "/api/keystone/status",
            get(keystone_handlers::keystone_status),
        )
        .route(
            "/api/keystone/enable",
            post(keystone_handlers::keystone_enable),
        )
        .route(
            "/api/keystone/export-ufvk",
            post(keystone_handlers::keystone_export_ufvk),
        )
        .route(
            "/api/keystone/prepare-send",
            post(keystone_handlers::keystone_prepare_send),
        )
        .route(
            "/api/keystone/complete-send",
            post(keystone_handlers::keystone_complete_send),
        )
        .route("/health", get(health_check))
        .layer(axum::middleware::from_fn(
            move |req: axum::extract::Request, next: axum::middleware::Next| {
                let api_key = api_key.clone();
                async move { middleware::api_key_auth(req, next, api_key).await }
            },
        ))
        .layer({
            let limiter = middleware::RateLimiter::new(rate_limit_requests, rate_limit_window);
            axum::middleware::from_fn(
                move |req: axum::extract::Request, next: axum::middleware::Next| {
                    let limiter = limiter.clone();
                    async move { middleware::rate_limit_middleware(req, next, limiter).await }
                },
            )
        })
        .layer(from_fn(middleware::security_headers))
        .layer(from_fn(middleware::request_logging))
        .layer({
            let cors_origins = if is_production {
                std::env::var("NOZY_CORS_ORIGINS")
                    .ok()
                    .map(|s| {
                        s.split(',')
                            .map(|s| s.trim().to_string())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default()
            } else {
                vec![]
            };

            let cors_layer = if is_production && !cors_origins.is_empty() {
                let origins: Vec<HeaderValue> = cors_origins
                    .iter()
                    .filter_map(|s| HeaderValue::from_str(s).ok())
                    .collect();
                CorsLayer::new().allow_origin(AllowOrigin::list(origins))
            } else {
                CorsLayer::new().allow_origin(AllowOrigin::predicate(
                    |origin: &HeaderValue, _request_head: &_| {
                        let origin_str = origin.to_str().unwrap_or("");
                        origin_str.starts_with("http://localhost:")
                            || origin_str.starts_with("http://127.0.0.1:")
                            || origin_str == "http://localhost"
                            || origin_str == "http://127.0.0.1"
                            || origin_str.starts_with("http://10.0.2.2:")
                            || origin_str.starts_with("http://0.0.0.0:")
                    },
                ))
            };

            cors_layer
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS, Method::DELETE])
                .allow_headers([
                    header::CONTENT_TYPE,
                    header::AUTHORIZATION,
                    header::ACCEPT,
                    header::ACCEPT_LANGUAGE,
                    header::ACCEPT_ENCODING,
                    header::CACHE_CONTROL,
                    header::CONNECTION,
                    header::USER_AGENT,
                    header::ORIGIN,
                    header::REFERER,
                ])
                .allow_credentials(true)
        })
        .layer(TraceLayer::new_for_http());

    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        info!("Received shutdown signal, shutting down gracefully...");
    };

    if https_enabled {
        use axum_server::tls_rustls::RustlsConfig;
        use std::net::SocketAddr;

        let cert_path = std::env::var("NOZY_SSL_CERT_PATH")
            .ok()
            .ok_or_else(|| anyhow::anyhow!("NOZY_SSL_CERT_PATH not set for HTTPS mode"))?;
        let key_path = std::env::var("NOZY_SSL_KEY_PATH")
            .ok()
            .ok_or_else(|| anyhow::anyhow!("NOZY_SSL_KEY_PATH not set for HTTPS mode"))?;

        info!("Loading SSL certificate from: {}", cert_path);
        info!("Loading SSL key from: {}", key_path);

        let config = RustlsConfig::from_pem_file(cert_path, key_path)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to load TLS configuration: {e}"))?;

        let ip: std::net::IpAddr = bind_host
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid NOZY_BIND_ADDR '{bind_host}': {e}"))?;
        let addr = SocketAddr::from((ip, https_port));
        info!(
            "API server listening on https://{}:{}",
            bind_host, https_port
        );
        info!("Health check: https://localhost:{}/health", https_port);

        let handle = axum_server::Handle::new();
        let shutdown_handle = handle.clone();

        tokio::spawn(async move {
            shutdown_signal.await;
            info!("Shutting down HTTPS server...");
            shutdown_handle.shutdown();
        });

        axum_server::bind_rustls(addr, config)
            .handle(handle)
            .serve(app.into_make_service())
            .await
            .map_err(|e| {
                tracing::error!("Server error: {}", e);
                anyhow::anyhow!("Server error: {e}")
            })?;
    } else {
        let listener = tokio::net::TcpListener::bind(format!("{bind_host}:{http_port}"))
            .await
            .map_err(|e| {
                tracing::error!("Failed to bind to {}:{}: {}", bind_host, http_port, e);
                anyhow::anyhow!(
                    "Failed to bind to {bind_host}:{http_port}: {e}. Is the port already in use?"
                )
            })?;
        info!("API server listening on http://{}:{}", bind_host, http_port);
        info!("Health check: http://localhost:{}/health", http_port);

        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal)
            .await
            .map_err(|e| {
                tracing::error!("Server error: {}", e);
                anyhow::anyhow!("Server error: {e}")
            })?;
    }

    info!("API server shut down successfully");
    Ok(())
}

async fn health_check() -> impl IntoResponse {
    ResponseJson(serde_json::json!({
        "status": "ok",
        "service": "nozywallet-api",
        "version": nozy::version_info::RELEASE_VERSION,
        "nozy_wallet": nozy::version_info::VERSION_DISPLAY,
        "nozy_release_codename": nozy::version_info::RELEASE_CODENAME
    }))
}
