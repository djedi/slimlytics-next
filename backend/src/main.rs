use anyhow::{Context, Result};
use base64::Engine;
use slimlytics_backend::search_console::SearchConsoleConfig;
use slimlytics_backend::{
    app,
    briefs::process_due_reports,
    enrichment::GeoIp,
    maintenance::{prune_expired_events, refresh_daily_rollups},
    AppState,
};
use sqlx::postgres::PgPoolOptions;
use std::{env, net::SocketAddr};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,tower_http=info".into()),
        )
        .init();
    let database_url = env::var("DATABASE_URL").context("DATABASE_URL is required")?;
    let jwt_secret = env::var("JWT_SECRET").context("JWT_SECRET is required")?;
    let identity_secret = env::var("VISITOR_HASH_SECRET")
        .or_else(|_| env::var("IDENTITY_SECRET"))
        .context("VISITOR_HASH_SECRET is required")?;
    let access_token_ttl_seconds: i64 = env::var("ACCESS_TOKEN_TTL_SECONDS")
        .unwrap_or_else(|_| "3600".into())
        .parse()
        .context("ACCESS_TOKEN_TTL_SECONDS must be an integer")?;
    if !(300..=604_800).contains(&access_token_ttl_seconds) {
        anyhow::bail!("ACCESS_TOKEN_TTL_SECONDS must be between 300 and 604800");
    }
    let trust_proxy = match env::var("TRUST_PROXY")
        .unwrap_or_else(|_| "false".into())
        .as_str()
    {
        "true" => true,
        "false" => false,
        _ => anyhow::bail!("TRUST_PROXY must be true or false"),
    };
    if jwt_secret.len() < 32 || identity_secret.len() < 32 {
        anyhow::bail!("secrets must be at least 32 bytes");
    }
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await?;
    sqlx::migrate!("../migrations").run(&pool).await?;
    refresh_daily_rollups(&pool, 3660).await?;
    let maintenance_pool = pool.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            let mut total = 0_u64;
            for _ in 0..100 {
                match prune_expired_events(&maintenance_pool, 10_000).await {
                    Ok(removed) => {
                        total += removed;
                        if removed < 10_000 {
                            break;
                        }
                        tokio::task::yield_now().await;
                    }
                    Err(error) => {
                        tracing::error!(%error, "event retention pruning failed");
                        break;
                    }
                }
            }
            if total > 0 {
                tracing::info!(removed = total, "expired events pruned");
            }
        }
    });
    let rollup_pool = pool.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            if let Err(error) = refresh_daily_rollups(&rollup_pool, 8).await {
                tracing::error!(%error, "daily analytics rollup refresh failed");
            }
        }
    });
    let address: SocketAddr = env::var("SLIMLYTICS_BIND")
        .or_else(|_| env::var("BIND_ADDR"))
        .unwrap_or_else(|_| "0.0.0.0:3001".into())
        .parse()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(%address, "Slimlytics API listening");
    let report_identity_secret = identity_secret.as_bytes().to_vec();
    let mut state = AppState::new(pool.clone(), jwt_secret, identity_secret.into_bytes())
        .with_access_token_ttl(access_token_ttl_seconds)
        .with_trust_proxy(trust_proxy);
    if let Some(path) = env::var("GEOIP_DATABASE_PATH")
        .ok()
        .filter(|value| !value.trim().is_empty())
    {
        state = state.with_geoip(
            GeoIp::open(&path).with_context(|| format!("failed to open GeoIP database {path}"))?,
        );
        tracing::info!(%path, "GeoIP enrichment enabled");
    }
    let optional_env = |name| env::var(name).ok().filter(|value| !value.trim().is_empty());
    let google_client_id = optional_env("GOOGLE_CLIENT_ID");
    let google_client_secret = optional_env("GOOGLE_CLIENT_SECRET");
    let google_redirect_uri = optional_env("GOOGLE_REDIRECT_URI");
    let integration_key = optional_env("INTEGRATION_ENCRYPTION_KEY");
    match (
        google_client_id,
        google_client_secret,
        google_redirect_uri,
        integration_key,
    ) {
        (None, None, None, None) => {}
        (Some(client_id), Some(client_secret), Some(redirect_uri), Some(key)) => {
            let decoded = base64::engine::general_purpose::STANDARD
                .decode(key)
                .context("INTEGRATION_ENCRYPTION_KEY must be base64")?;
            let encryption_key: [u8; 32] = decoded.try_into().map_err(|_| {
                anyhow::anyhow!("INTEGRATION_ENCRYPTION_KEY must decode to 32 bytes")
            })?;
            let return_uri = format!(
                "{}/app",
                env::var("SLIMLYTICS_BASE_URL")
                    .unwrap_or_else(|_| "http://localhost:8080".into())
                    .trim_end_matches('/')
            );
            state = state.with_search_console(SearchConsoleConfig {
                client_id,
                client_secret,
                redirect_uri,
                return_uri,
                encryption_key,
            });
        }
        _ => anyhow::bail!(
            "GOOGLE_CLIENT_ID, GOOGLE_CLIENT_SECRET, GOOGLE_REDIRECT_URI, and \
             INTEGRATION_ENCRYPTION_KEY must be configured together"
        ),
    }
    let report_pool = pool;
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            match process_due_reports(&report_pool, &report_identity_secret).await {
                Ok(count) if count > 0 => tracing::info!(count, "scheduled reports processed"),
                Ok(_) => {}
                Err(error) => tracing::error!(%error, "scheduled report processing failed"),
            }
        }
    });
    axum::serve(
        listener,
        app(state).into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
