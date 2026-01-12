use axum::{
    Router,
    routing::{get, post},
    middleware as axum_middleware,
};
use anyhow::Context;
use dotenvy as dotenv;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod config;
mod db;
mod error;
mod solana;
mod vault;
mod ws;
mod middleware;

use config::Config;
use solana::SolanaClient;
use vault::VaultManager;
use db::{SnapshotService, MfaService};
use middleware::RateLimitLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // trying multiple .env locations since working directory differs between dev and prod
    let _ = dotenv::from_filename_override(".env");
    let _ = dotenv::from_filename_override(concat!(env!("CARGO_MANIFEST_DIR"), "/.env"));
    let _ = dotenv::dotenv_override();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,vault_backend=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Collateral Vault Management System");

    tracing::info!("Loading configuration from environment");
    let config = Config::from_env().context("error with configuration")?;
    tracing::info!("Configuration loaded successfully");

    tracing::info!("Connecting to database...");
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .context("Failed to connect to database")?;

    tracing::info!("Database connected successfully");

    tracing::info!("Initializing Solana client...");
    let solana_client = SolanaClient::new(&config).context("Failed to initialize Solana client")?;
    tracing::info!("Solana client initialized");

    let vault_manager = Arc::new(VaultManager::new(solana_client, db_pool.clone()));

    let snapshot_service = Arc::new(SnapshotService::new(db_pool.clone()));

    let mfa_service = Arc::new(MfaService::new(
        db_pool.clone(),
        "Collateral Vault".to_string()
    ));

    tracing::info!("Initializing rate limiting...");
    let redis_url = std::env::var("REDIS_URL").ok();
    
    let rate_limit_default = Arc::new(
        RateLimitLayer::with_defaults(redis_url.as_deref()).await
    );
    let rate_limit_read = Arc::new(
        RateLimitLayer::read_heavy(redis_url.as_deref()).await
    );
    let rate_limit_write = Arc::new(
        RateLimitLayer::write_heavy(redis_url.as_deref()).await
    );
    let rate_limit_expensive = Arc::new(
        RateLimitLayer::expensive(redis_url.as_deref()).await
    );
    
    tracing::info!("Rate limiting initialized successfully");

    // spawning as background task so server startup isn't blocked by snapshot schedule
    let snapshot_service_clone = snapshot_service.clone();
    tokio::spawn(async move {
        tracing::info!("Starting periodic TVL snapshot task");
        if let Err(e) = snapshot_service_clone.run_periodic_snapshot().await {
            tracing::error!("Snapshot task failed: {}", e);
        }
    });

    let app_state = Arc::new(AppState {
        vault_manager,
        db_pool: db_pool.clone(),
        mfa_service,
    });

    // grouping routes by rate limit tier to avoid repeating the middleware closure pattern everywhere
    let app = Router::new()
        .route("/health", get(api::health::health_check))
        .route("/config/public", get(api::health::public_config))

        .route("/vault/initialize", post(api::vault::build_initialize_unsigned))
        .route("/vault/deposit", post(api::vault::build_deposit_unsigned))
        .route("/vault/withdraw", post(api::vault::build_withdraw_unsigned))
        .route("/vault/sync", post(api::vault::sync_tx))
        .route("/vault/force-sync", post(api::vault::force_sync_vault))
        .route_layer({
            let limiter = rate_limit_write.clone();
            axum_middleware::from_fn(move |headers, req, next| {
                let limiter = limiter.clone();
                async move { limiter.middleware(headers, req, next).await }
            })
        })

        .route("/vault/balance/:user", get(api::vault::get_balance))
        .route("/vault/transactions/:user", get(api::vault::get_transactions))
        .route("/vault/tvl", get(api::vault::get_tvl))
        .route_layer({
            let limiter = rate_limit_read.clone();
            axum_middleware::from_fn(move |headers, req, next| {
                let limiter = limiter.clone();
                async move { limiter.middleware(headers, req, next).await }
            })
        })

        .route("/yield/compound", post(api::r#yield::build_compound_yield_tx))
        .route("/yield/auto-compound", post(api::r#yield::build_auto_compound_tx))
        .route("/yield/configure", post(api::r#yield::build_configure_yield_tx))
        .route("/yield/sync", post(api::r#yield::sync_yield_tx))
        .route_layer({
            let limiter = rate_limit_write.clone();
            axum_middleware::from_fn(move |headers, req, next| {
                let limiter = limiter.clone();
                async move { limiter.middleware(headers, req, next).await }
            })
        })

        .route("/yield/info/:user", get(api::r#yield::get_yield_info))
        .route_layer({
            let limiter = rate_limit_read.clone();
            axum_middleware::from_fn(move |headers, req, next| {
                let limiter = limiter.clone();
                async move { limiter.middleware(headers, req, next).await }
            })
        })

        .route("/analytics/overview", get(api::analytics::get_overview))
        .route("/analytics/distribution", get(api::analytics::get_user_distribution))
        .route("/analytics/utilization", get(api::analytics::get_utilization))
        .route("/analytics/flow", get(api::analytics::get_flow_metrics))
        .route("/analytics/yield", get(api::analytics::get_yield_metrics))
        .route("/analytics/chart/tvl", get(api::analytics::get_tvl_chart))
        .route_layer({
            let limiter = rate_limit_expensive.clone();
            axum_middleware::from_fn(move |headers, req, next| {
                let limiter = limiter.clone();
                async move { limiter.middleware(headers, req, next).await }
            })
        })

        .route("/mfa/setup", post(api::mfa::setup_mfa))
        .route("/mfa/verify-setup", post(api::mfa::verify_and_enable_mfa))
        .route("/mfa/disable", post(api::mfa::disable_mfa))
        .route("/mfa/check", post(api::mfa::check_mfa))
        .route("/mfa/status/:vault_address", get(api::mfa::get_mfa_status))
        .route_layer({
            let limiter = rate_limit_default.clone();
            axum_middleware::from_fn(move |headers, req, next| {
                let limiter = limiter.clone();
                async move { limiter.middleware(headers, req, next).await }
            })
        })

        // WebSocket has its own rate limiting logic in the handler
        .route("/ws", get(ws::handler::ws_handler))
        
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    // in case the configured port is taken, try a few more before giving up
    let mut port = config.port;
    let mut listener = None;

    for _ in 0..10u16 {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        match tokio::net::TcpListener::bind(&addr).await {
            Ok(l) => {
                listener = Some((addr, l));
                break;
            }
            Err(e) => {
                tracing::warn!("Failed to bind to {}: {} (trying next port)", addr, e);
                port = port.saturating_add(1);
            }
        }
    }

    let (addr, listener) = listener.ok_or_else(|| anyhow::anyhow!(
        "Failed to bind to any port in range {}..{}",
        config.port,
        config.port.saturating_add(9)
    ))?;

    tracing::info!("Listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    pub vault_manager: Arc<VaultManager>,
    pub db_pool: sqlx::PgPool,
    pub mfa_service: Arc<MfaService>,
}

