use anyhow::{Context, Result};
use slimlytics_backend::{app, maintenance::prune_expired_events, AppState};
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
    let address: SocketAddr = env::var("SLIMLYTICS_BIND")
        .or_else(|_| env::var("BIND_ADDR"))
        .unwrap_or_else(|_| "0.0.0.0:3001".into())
        .parse()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(%address, "Slimlytics API listening");
    axum::serve(
        listener,
        app(
            AppState::new(pool, jwt_secret, identity_secret.into_bytes())
                .with_access_token_ttl(access_token_ttl_seconds)
                .with_trust_proxy(trust_proxy),
        )
        .into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
