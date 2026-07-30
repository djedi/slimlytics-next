use crate::{
    auth::{
        generate_api_token, hash_api_token, hash_password, issue_token, verify_password,
        verify_token,
    },
    error::ApiError,
    identity::derive_ids,
    models::*,
    privacy::sanitize_url,
    traffic::{client_metadata, origin_allowed, traffic_class, RateLimiter},
};
use axum::{
    extract::{ConnectInfo, FromRequestParts, Path, Query, State},
    http::{header, request::Parts, HeaderMap, StatusCode},
    response::{sse::Event, IntoResponse, Redirect, Sse},
    routing::{delete, get, post},
    Json, Router,
};
use chrono::{DateTime, Duration as ChronoDuration, NaiveDate, TimeZone, Utc};
use serde::Serialize;
use serde_json::{json, Value};
use sqlx::{PgPool, Postgres, Transaction};
use std::{
    convert::Infallible,
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::Duration,
};
use tokio::sync::broadcast;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use url::Url;
use uuid::Uuid;

type DateBounds = (DateTime<Utc>, DateTime<Utc>, DateTime<Utc>);
type CsvRow = (
    DateTime<Utc>,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
);

#[derive(Clone, Debug, Serialize)]
pub struct StreamMessage {
    pub id: i64,
    pub site_id: Uuid,
    pub payload: Value,
}

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    jwt_secret: Arc<String>,
    identity_secret: Arc<Vec<u8>>,
    limiter: RateLimiter,
    stream_tx: broadcast::Sender<StreamMessage>,
    internal_ips: Arc<Vec<IpAddr>>,
    access_token_ttl_seconds: i64,
    trust_proxy: bool,
}
impl AppState {
    pub fn new(pool: PgPool, jwt_secret: String, identity_secret: Vec<u8>) -> Self {
        let (stream_tx, _) = broadcast::channel(1024);
        Self {
            pool,
            jwt_secret: Arc::new(jwt_secret),
            identity_secret: Arc::new(identity_secret),
            limiter: RateLimiter::new(120, Duration::from_secs(60)),
            stream_tx,
            internal_ips: Arc::new(Vec::new()),
            access_token_ttl_seconds: 3600,
            trust_proxy: false,
        }
    }
    pub fn with_internal_ips(mut self, ips: Vec<IpAddr>) -> Self {
        self.internal_ips = Arc::new(ips);
        self
    }
    pub fn with_access_token_ttl(mut self, seconds: i64) -> Self {
        self.access_token_ttl_seconds = seconds;
        self
    }
    pub fn with_trust_proxy(mut self, trust_proxy: bool) -> Self {
        self.trust_proxy = trust_proxy;
        self
    }
}

#[derive(Clone, Copy)]
struct CurrentUser(Uuid);
impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = ApiError;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let value = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(ApiError::Unauthorized)?;
        if value.starts_with("slyt_") {
            let user: Option<Uuid> = sqlx::query_scalar(
                "UPDATE api_tokens SET last_used_at=now() WHERE token_hash=$1 AND revoked_at IS NULL AND expires_at>now() RETURNING user_id",
            )
            .bind(hash_api_token(value))
            .fetch_optional(&state.pool)
            .await?;
            return user.map(Self).ok_or(ApiError::Unauthorized);
        }
        verify_token(value, &state.jwt_secret)
            .map(|claims| Self(claims.sub))
            .map_err(|_| ApiError::Unauthorized)
    }
}

#[derive(Clone, Copy)]
struct SessionUser(Uuid);
impl FromRequestParts<AppState> for SessionUser {
    type Rejection = ApiError;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let value = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(ApiError::Unauthorized)?;
        verify_token(value, &state.jwt_secret)
            .map(|claims| Self(claims.sub))
            .map_err(|_| ApiError::Unauthorized)
    }
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/api/openapi.json", get(openapi_document))
        .route("/api/docs", get(scalar_docs))
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route("/api/auth/me", get(me))
        .route(
            "/api/account/tokens",
            get(list_api_tokens).post(create_api_token),
        )
        .route(
            "/api/account/tokens/current",
            delete(revoke_current_api_token),
        )
        .route("/api/account/tokens/{token_id}", delete(revoke_api_token))
        .route("/api/sites", get(list_sites).post(create_site))
        .route("/api/sites/ensure", post(ensure_site))
        .route(
            "/api/sites/{site_id}",
            get(get_site).put(update_site).delete(delete_site),
        )
        .route("/api/sites/{site_id}/rotate-key", post(rotate_key))
        .route(
            "/api/sites/{site_id}/anti-adblock",
            axum::routing::put(update_anti_adblock),
        )
        .route("/api/sites/{site_id}/overview", get(overview))
        .route("/api/sites/{site_id}/reports/{dimension}", get(report))
        .route("/api/sites/{site_id}/visitors", get(list_visitors))
        .route(
            "/api/sites/{site_id}/visitors/{visitor_id}",
            get(visitor_timeline),
        )
        .route("/api/sites/{site_id}/events", get(custom_events))
        .route(
            "/api/sites/{site_id}/goals",
            get(list_goals).post(create_goal),
        )
        .route("/api/sites/{site_id}/goals/{goal_id}", delete(delete_goal))
        .route("/api/sites/{site_id}/export.csv", get(export_csv))
        .route("/api/sites/{site_id}/stream", get(stream))
        .route(
            "/api/collect/{write_key}",
            get(collect_test).post(collect).options(collect_options),
        )
        .route(
            "/api/e/{write_key}",
            get(collect_test).post(collect).options(collect_options),
        )
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status":"ok"})))
}

const OPENAPI_JSON: &str = include_str!("../../docs/openapi.json");
async fn openapi_document() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/vnd.oai.openapi+json")],
        OPENAPI_JSON,
    )
}

async fn scalar_docs() -> Redirect {
    Redirect::temporary("/docs/api")
}

async fn ready(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    sqlx::query("SELECT 1").execute(&state.pool).await?;
    Ok(Json(json!({"status":"ready"})))
}

async fn register(
    State(state): State<AppState>,
    Json(input): Json<Credentials>,
) -> Result<impl IntoResponse, ApiError> {
    validate_credentials(&input)?;
    let hash = hash_password(&input.password).map_err(|_| ApiError::Internal)?;
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO users(email,password_hash) VALUES(lower($1),$2) RETURNING id",
    )
    .bind(input.email.trim())
    .bind(hash)
    .fetch_one(&state.pool)
    .await
    .map_err(map_conflict)?;
    let token = issue_token(id, &state.jwt_secret, state.access_token_ttl_seconds)
        .map_err(|_| ApiError::Internal)?;
    Ok((StatusCode::CREATED, Json(TokenResponse { token })))
}
async fn login(
    State(state): State<AppState>,
    Json(input): Json<Credentials>,
) -> Result<Json<TokenResponse>, ApiError> {
    let row: Option<(Uuid, String)> =
        sqlx::query_as("SELECT id,password_hash FROM users WHERE email=lower($1)")
            .bind(input.email.trim())
            .fetch_optional(&state.pool)
            .await?;
    let (id, hash) = row.ok_or(ApiError::Unauthorized)?;
    if !verify_password(&input.password, &hash).map_err(|_| ApiError::Unauthorized)? {
        return Err(ApiError::Unauthorized);
    }
    Ok(Json(TokenResponse {
        token: issue_token(id, &state.jwt_secret, state.access_token_ttl_seconds)
            .map_err(|_| ApiError::Internal)?,
    }))
}
async fn me(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<Json<Value>, ApiError> {
    let row: (Uuid, String, DateTime<Utc>) =
        sqlx::query_as("SELECT id,email,created_at FROM users WHERE id=$1")
            .bind(user)
            .fetch_optional(&state.pool)
            .await?
            .ok_or(ApiError::Unauthorized)?;
    Ok(Json(
        json!({"id": row.0, "email": row.1, "createdAt": row.2}),
    ))
}

async fn create_api_token(
    State(state): State<AppState>,
    SessionUser(user): SessionUser,
    Json(input): Json<ApiTokenInput>,
) -> Result<impl IntoResponse, ApiError> {
    let name = input.name.trim();
    if name.is_empty() || name.chars().count() > 100 {
        return Err(ApiError::BadRequest(
            "token name must contain between 1 and 100 characters".into(),
        ));
    }
    let days = input.expires_in_days.unwrap_or(365);
    if !(1..=3650).contains(&days) {
        return Err(ApiError::BadRequest(
            "expiresInDays must be between 1 and 3650".into(),
        ));
    }
    let token = generate_api_token();
    let token_prefix: String = token.chars().take(12).collect();
    let expires_at = Utc::now() + ChronoDuration::days(days);
    let row: (Uuid, DateTime<Utc>) = sqlx::query_as(
        "INSERT INTO api_tokens(user_id,name,token_hash,token_prefix,expires_at) VALUES($1,$2,$3,$4,$5) RETURNING id,created_at",
    )
    .bind(user)
    .bind(name)
    .bind(hash_api_token(&token))
    .bind(&token_prefix)
    .bind(expires_at)
    .fetch_one(&state.pool)
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(ApiTokenCreated {
            id: row.0,
            name: name.to_owned(),
            token_prefix,
            token,
            expires_at,
            created_at: row.1,
        }),
    ))
}

async fn list_api_tokens(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<Json<Vec<ApiTokenSummary>>, ApiError> {
    let tokens = sqlx::query_as(
        "SELECT id,name,token_prefix,last_used_at,expires_at,created_at FROM api_tokens WHERE user_id=$1 AND revoked_at IS NULL AND expires_at>now() ORDER BY created_at DESC",
    )
    .bind(user)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(tokens))
}

async fn revoke_api_token(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(token): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let result = sqlx::query(
        "UPDATE api_tokens SET revoked_at=now() WHERE id=$1 AND user_id=$2 AND revoked_at IS NULL",
    )
    .bind(token)
    .bind(user)
    .execute(&state.pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn revoke_current_api_token(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .filter(|value| value.starts_with("slyt_"))
        .ok_or(ApiError::Unauthorized)?;
    let result = sqlx::query(
        "UPDATE api_tokens SET revoked_at=now() WHERE user_id=$1 AND token_hash=$2 AND revoked_at IS NULL",
    )
    .bind(user)
    .bind(hash_api_token(token))
    .execute(&state.pool)
    .await?;
    if result.rows_affected() == 1 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::Unauthorized)
    }
}

fn validate_credentials(value: &Credentials) -> Result<(), ApiError> {
    if !value.email.contains('@') || value.password.len() < 12 {
        return Err(ApiError::BadRequest(
            "valid email and password of at least 12 characters required".into(),
        ));
    }
    Ok(())
}
fn map_conflict(error: sqlx::Error) -> ApiError {
    if matches!(&error, sqlx::Error::Database(db) if db.is_unique_violation()) {
        ApiError::BadRequest("resource already exists".into())
    } else {
        ApiError::Database(error)
    }
}

async fn require_site(pool: &PgPool, user: Uuid, site: Uuid, write: bool) -> Result<(), ApiError> {
    let role: Option<String> = sqlx::query_scalar(
        "SELECT role::text FROM site_memberships WHERE site_id=$1 AND user_id=$2",
    )
    .bind(site)
    .bind(user)
    .fetch_optional(pool)
    .await?;
    match role.as_deref() {
        None => Err(ApiError::NotFound),
        Some("viewer") if write => Err(ApiError::Forbidden),
        Some(_) => Ok(()),
    }
}
async fn list_sites(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<Json<Vec<Site>>, ApiError> {
    Ok(Json(sqlx::query_as("SELECT s.id,s.name,s.domain,s.timezone,s.allowed_origins,s.retention_days,s.write_key,s.anti_adblock_server,s.anti_adblock_js_path,s.anti_adblock_beacon_path,s.created_at FROM sites s JOIN site_memberships m ON m.site_id=s.id WHERE m.user_id=$1 ORDER BY s.created_at").bind(user).fetch_all(&state.pool).await?))
}
async fn create_site(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(mut input): Json<SiteInput>,
) -> Result<impl IntoResponse, ApiError> {
    validate_site(&input)?;
    input.domain = canonical_domain(&input.domain)?;
    let mut tx = state.pool.begin().await?;
    let site: Site = sqlx::query_as("INSERT INTO sites(name,domain,timezone,allowed_origins,retention_days) VALUES($1,$2,$3,$4,$5) RETURNING id,name,domain,timezone,allowed_origins,retention_days,write_key,anti_adblock_server,anti_adblock_js_path,anti_adblock_beacon_path,created_at")
        .bind(input.name)
        .bind(input.domain)
        .bind(input.timezone)
        .bind(input.allowed_origins)
        .bind(input.retention_days)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_conflict)?;
    sqlx::query("INSERT INTO site_memberships(site_id,user_id,role) VALUES($1,$2,'owner')")
        .bind(site.id)
        .bind(user)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(site)))
}

async fn ensure_site(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(mut input): Json<SiteInput>,
) -> Result<impl IntoResponse, ApiError> {
    validate_site(&input)?;
    input.domain = canonical_domain(&input.domain)?;
    let mut tx = state.pool.begin().await?;
    let inserted: Option<Site> = sqlx::query_as(
        "INSERT INTO sites(name,domain,timezone,allowed_origins,retention_days) VALUES($1,$2,$3,$4,$5) ON CONFLICT (lower(domain)) DO NOTHING RETURNING id,name,domain,timezone,allowed_origins,retention_days,write_key,anti_adblock_server,anti_adblock_js_path,anti_adblock_beacon_path,created_at",
    )
    .bind(&input.name)
    .bind(&input.domain)
    .bind(&input.timezone)
    .bind(&input.allowed_origins)
    .bind(input.retention_days)
    .fetch_optional(&mut *tx)
    .await?;
    let (created, site) = if let Some(site) = inserted {
        sqlx::query("INSERT INTO site_memberships(site_id,user_id,role) VALUES($1,$2,'owner')")
            .bind(site.id)
            .bind(user)
            .execute(&mut *tx)
            .await?;
        (true, site)
    } else {
        let site = sqlx::query_as(
            "SELECT s.id,s.name,s.domain,s.timezone,s.allowed_origins,s.retention_days,s.write_key,s.anti_adblock_server,s.anti_adblock_js_path,s.anti_adblock_beacon_path,s.created_at FROM sites s JOIN site_memberships m ON m.site_id=s.id WHERE m.user_id=$1 AND lower(s.domain)=lower($2)",
        )
        .bind(user)
        .bind(&input.domain)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| ApiError::BadRequest("domain is already managed by another account".into()))?;
        (false, site)
    };
    tx.commit().await?;
    let status = if created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    Ok((status, Json(EnsureSiteResponse { created, site })))
}

fn validate_site(input: &SiteInput) -> Result<(), ApiError> {
    if input.name.trim().is_empty()
        || input.domain.trim().is_empty()
        || !(1..=3650).contains(&input.retention_days)
    {
        return Err(ApiError::BadRequest("invalid site".into()));
    }
    canonical_domain(&input.domain)?;
    for origin in &input.allowed_origins {
        let parsed =
            Url::parse(origin).map_err(|_| ApiError::BadRequest("invalid origin".into()))?;
        if !matches!(parsed.scheme(), "http" | "https") || parsed.path() != "/" {
            return Err(ApiError::BadRequest(
                "origins must contain scheme and host only".into(),
            ));
        }
    }
    Ok(())
}

fn canonical_domain(value: &str) -> Result<String, ApiError> {
    let value = value.trim().trim_end_matches('.');
    if value.contains("://") || value.chars().any(char::is_whitespace) {
        return Err(ApiError::BadRequest("invalid domain".into()));
    }
    let parsed = Url::parse(&format!("https://{value}"))
        .map_err(|_| ApiError::BadRequest("invalid domain".into()))?;
    let host = parsed
        .host_str()
        .filter(|_| parsed.port().is_none() && parsed.path() == "/")
        .filter(|host| host.contains('.') || *host == "localhost")
        .ok_or_else(|| ApiError::BadRequest("invalid domain".into()))?;
    Ok(host.to_ascii_lowercase())
}

async fn get_site(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(site): Path<Uuid>,
) -> Result<Json<Site>, ApiError> {
    require_site(&state.pool, user, site, false).await?;
    Ok(Json(fetch_site(&state.pool, site).await?))
}
async fn fetch_site(pool: &PgPool, id: Uuid) -> Result<Site, ApiError> {
    sqlx::query_as("SELECT id,name,domain,timezone,allowed_origins,retention_days,write_key,anti_adblock_server,anti_adblock_js_path,anti_adblock_beacon_path,created_at FROM sites WHERE id=$1").bind(id).fetch_optional(pool).await?.ok_or(ApiError::NotFound)
}
async fn update_site(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(site): Path<Uuid>,
    Json(mut i): Json<SiteInput>,
) -> Result<Json<Site>, ApiError> {
    require_site(&state.pool, user, site, true).await?;
    validate_site(&i)?;
    i.domain = canonical_domain(&i.domain)?;
    sqlx::query("UPDATE sites SET name=$2,domain=$3,timezone=$4,allowed_origins=$5,retention_days=$6,updated_at=now() WHERE id=$1")
        .bind(site)
        .bind(i.name)
        .bind(i.domain)
        .bind(i.timezone)
        .bind(i.allowed_origins)
        .bind(i.retention_days)
        .execute(&state.pool)
        .await
        .map_err(map_conflict)?;
    Ok(Json(fetch_site(&state.pool, site).await?))
}

async fn update_anti_adblock(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(site): Path<Uuid>,
    Json(input): Json<AntiAdblockInput>,
) -> Result<Json<Site>, ApiError> {
    require_site(&state.pool, user, site, true).await?;
    input
        .validate()
        .map_err(|message| ApiError::BadRequest(message.into()))?;
    sqlx::query(
        "UPDATE sites SET anti_adblock_server=$2,anti_adblock_js_path=$3,anti_adblock_beacon_path=$4,updated_at=now() WHERE id=$1",
    )
    .bind(site)
    .bind(input.server_type)
    .bind(input.js_path)
    .bind(input.beacon_path)
    .execute(&state.pool)
    .await?;
    Ok(Json(fetch_site(&state.pool, site).await?))
}

async fn delete_site(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(site): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    require_site(&state.pool, user, site, true).await?;
    sqlx::query("DELETE FROM sites WHERE id=$1")
        .bind(site)
        .execute(&state.pool)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
async fn rotate_key(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(site): Path<Uuid>,
) -> Result<Json<WriteKeyResponse>, ApiError> {
    require_site(&state.pool, user, site, true).await?;
    let key:Uuid=sqlx::query_scalar("UPDATE sites SET write_key=gen_random_uuid(),updated_at=now() WHERE id=$1 RETURNING write_key").bind(site).fetch_one(&state.pool).await?;
    Ok(Json(WriteKeyResponse { write_key: key }))
}

fn client_ip(headers: &HeaderMap, peer: IpAddr, trust_proxy: bool) -> IpAddr {
    if trust_proxy {
        headers
            .get("x-forwarded-for")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(',').next())
            .and_then(|value| value.trim().parse().ok())
            .unwrap_or(peer)
    } else {
        peer
    }
}

async fn collect_test(
    State(state): State<AppState>,
    Path(key): Path<Uuid>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let allowed: Option<Vec<String>> =
        sqlx::query_scalar("SELECT allowed_origins FROM sites WHERE write_key=$1")
            .bind(key)
            .fetch_optional(&state.pool)
            .await?;
    let allowed = allowed.ok_or(ApiError::NotFound)?;
    let origin = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok());
    if origin.is_some() && !origin_allowed(origin, &allowed) {
        return Err(ApiError::Forbidden);
    }
    let mut response_headers = HeaderMap::new();
    response_headers.insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    if let Some(origin) = origin {
        response_headers.insert(
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            origin
                .parse()
                .map_err(|_| ApiError::BadRequest("invalid origin".into()))?,
        );
        response_headers.insert(header::VARY, "Origin".parse().unwrap());
    }
    Ok((response_headers, Json(json!({"status":"ok"}))))
}

#[axum::debug_handler]
async fn collect(
    State(state): State<AppState>,
    Path(key): Path<Uuid>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(input): Json<CollectInput>,
) -> Result<impl IntoResponse, ApiError> {
    let site: Option<(Uuid, Vec<String>)> =
        sqlx::query_as("SELECT id,allowed_origins FROM sites WHERE write_key=$1")
            .bind(key)
            .fetch_optional(&state.pool)
            .await?;
    let (site, allowed) = site.ok_or(ApiError::NotFound)?;
    let origin = headers.get(header::ORIGIN).and_then(|v| v.to_str().ok());
    if !origin_allowed(origin, &allowed) {
        return Err(ApiError::Forbidden);
    }
    let ip = client_ip(&headers, peer.ip(), state.trust_proxy);
    if !state.limiter.check(&format!("{site}:{ip}")) {
        return Err(ApiError::RateLimited);
    }
    let ua = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let at = input.occurred_at.unwrap_or_else(Utc::now);
    if (at - Utc::now()).num_hours().abs() > 24 {
        return Err(ApiError::BadRequest(
            "occurred_at outside accepted window".into(),
        ));
    }
    let clean = sanitize_url(&input.url).map_err(|_| ApiError::BadRequest("invalid url".into()))?;
    let parsed = Url::parse(&input.url).map_err(|_| ApiError::BadRequest("invalid url".into()))?;
    let ids = derive_ids(
        &state.identity_secret,
        &site.to_string(),
        &ip.to_string(),
        ua,
        at,
    );
    let referrer = input
        .referrer
        .as_deref()
        .map(sanitize_url)
        .transpose()
        .map_err(|_| ApiError::BadRequest("invalid referrer".into()))?;
    let ref_host = referrer
        .as_deref()
        .and_then(|v| Url::parse(v).ok())
        .and_then(|v| v.host_str().map(str::to_owned));
    let class = traffic_class(ua, ip, &state.internal_ips);
    let (browser, os) = client_metadata(ua);
    let country = headers
        .get("cf-ipcountry")
        .and_then(|value| value.to_str().ok())
        .filter(|value| value.len() == 2 && value.bytes().all(|byte| byte.is_ascii_alphabetic()))
        .map(str::to_ascii_uppercase);
    if !input.properties.is_object() {
        return Err(ApiError::BadRequest("properties must be an object".into()));
    }
    let device = if input.screen_width.unwrap_or(1024) < 768 {
        "mobile"
    } else {
        "desktop"
    };
    let q = |name: &str| {
        parsed
            .query_pairs()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.into_owned())
    };
    let mut tx = state.pool.begin().await?;
    let event_id:Uuid=sqlx::query_scalar("INSERT INTO events(site_id,occurred_at,visitor_id,session_id,event_name,url,path,referrer,referrer_host,title,country_code,device_type,browser,os,utm_source,utm_medium,utm_campaign,utm_term,utm_content,properties,traffic_class) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21::traffic_class) RETURNING id").bind(site).bind(at).bind(&ids.visitor_id).bind(&ids.session_id).bind(&input.name).bind(&clean).bind(parsed.path()).bind(referrer).bind(ref_host.as_deref()).bind(input.title).bind(country.as_deref()).bind(device).bind(browser).bind(os).bind(q("utm_source")).bind(q("utm_medium")).bind(q("utm_campaign")).bind(q("utm_term")).bind(q("utm_content")).bind(input.properties).bind(class).fetch_one(&mut *tx).await?;
    complete_goals(
        &mut tx,
        site,
        event_id,
        &ids.visitor_id,
        at,
        &input.name,
        parsed.path(),
    )
    .await?;
    let payload = json!({
        "id": event_id,
        "type": input.name,
        "page": parsed.path(),
        "visitorId": ids.visitor_id,
        "country": country,
        "timestamp": at,
        "referrer": ref_host
    });
    let stream_id: i64 = sqlx::query_scalar(
        "INSERT INTO stream_events(site_id,event_id,payload) VALUES($1,$2,$3) RETURNING id",
    )
    .bind(site)
    .bind(event_id)
    .bind(&payload)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    let _ = state.stream_tx.send(StreamMessage {
        id: stream_id,
        site_id: site,
        payload,
    });
    let mut response_headers = HeaderMap::new();
    if let Some(origin) = origin {
        response_headers.insert(
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            origin
                .parse()
                .map_err(|_| ApiError::BadRequest("invalid origin".into()))?,
        );
        response_headers.insert(header::VARY, "Origin".parse().unwrap());
    }
    Ok((response_headers, StatusCode::ACCEPTED))
}

async fn collect_options(
    State(state): State<AppState>,
    Path(key): Path<Uuid>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let allowed: Option<Vec<String>> =
        sqlx::query_scalar("SELECT allowed_origins FROM sites WHERE write_key=$1")
            .bind(key)
            .fetch_optional(&state.pool)
            .await?;
    let allowed = allowed.ok_or(ApiError::NotFound)?;
    let origin = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        .ok_or(ApiError::Forbidden)?;
    if !origin_allowed(Some(origin), &allowed) {
        return Err(ApiError::Forbidden);
    }
    let mut response_headers = HeaderMap::new();
    response_headers.insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        origin
            .parse()
            .map_err(|_| ApiError::BadRequest("invalid origin".into()))?,
    );
    response_headers.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        "POST, OPTIONS".parse().unwrap(),
    );
    response_headers.insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        "content-type".parse().unwrap(),
    );
    response_headers.insert(header::ACCESS_CONTROL_MAX_AGE, "86400".parse().unwrap());
    response_headers.insert(header::VARY, "Origin".parse().unwrap());
    Ok((response_headers, StatusCode::NO_CONTENT))
}

async fn complete_goals(
    tx: &mut Transaction<'_, Postgres>,
    site: Uuid,
    event: Uuid,
    visitor: &str,
    at: DateTime<Utc>,
    name: &str,
    path: &str,
) -> Result<(), ApiError> {
    sqlx::query("INSERT INTO goal_completions(goal_id,event_id,site_id,visitor_id,occurred_at) SELECT id,$2,$1,$3,$4 FROM goals WHERE site_id=$1 AND event_name=$5 AND (path_pattern IS NULL OR $6 LIKE path_pattern) ON CONFLICT DO NOTHING").bind(site).bind(event).bind(visitor).bind(at).bind(name).bind(path).execute(&mut **tx).await?;
    Ok(())
}

fn bounds(from: NaiveDate, to: NaiveDate) -> Result<DateBounds, ApiError> {
    if to < from || (to - from).num_days() > 366 {
        return Err(ApiError::BadRequest("invalid date range".into()));
    }
    let start = Utc.from_utc_datetime(&from.and_hms_opt(0, 0, 0).unwrap());
    let end = Utc.from_utc_datetime(&(to + ChronoDuration::days(1)).and_hms_opt(0, 0, 0).unwrap());
    let prior = start - (end - start);
    Ok((start, end, prior))
}
async fn counts(
    pool: &PgPool,
    site: Uuid,
    a: DateTime<Utc>,
    b: DateTime<Utc>,
) -> Result<(i64, i64, i64, i64), ApiError> {
    Ok(sqlx::query_as("SELECT count(*) FILTER(WHERE event_name='pageview'),count(DISTINCT visitor_id),count(DISTINCT session_id),count(*) FILTER(WHERE event_name<>'pageview') FROM events WHERE site_id=$1 AND occurred_at >= $2 AND occurred_at < $3 AND traffic_class='human'").bind(site).bind(a).bind(b).fetch_one(pool).await?)
}
fn metric(current: i64, previous: i64) -> Metric {
    Metric {
        current,
        previous,
        change_percent: if previous == 0 {
            None
        } else {
            Some((current - previous) as f64 * 100.0 / previous as f64)
        },
    }
}
async fn overview(
    State(s): State<AppState>,
    CurrentUser(u): CurrentUser,
    Path(site): Path<Uuid>,
    Query(q): Query<RangeQuery>,
) -> Result<Json<Overview>, ApiError> {
    require_site(&s.pool, u, site, false).await?;
    let (a, b, p) = bounds(q.from, q.to)?;
    let c = counts(&s.pool, site, a, b).await?;
    let old = counts(&s.pool, site, p, a).await?;
    Ok(Json(Overview {
        views: metric(c.0, old.0),
        visitors: metric(c.1, old.1),
        sessions: metric(c.2, old.2),
        events: metric(c.3, old.3),
    }))
}
async fn report(
    State(s): State<AppState>,
    CurrentUser(u): CurrentUser,
    Path((site, dimension)): Path<(Uuid, String)>,
    Query(q): Query<ReportQuery>,
) -> Result<Json<Vec<ReportRow>>, ApiError> {
    require_site(&s.pool, u, site, false).await?;
    let (a, b, _) = bounds(q.from, q.to)?;
    let column = match dimension.as_str() {
        "pages" => "path",
        "referrers" => "COALESCE(referrer_host, '(direct)')",
        "countries" => "COALESCE(country_code, 'unknown')",
        "devices" => "COALESCE(device_type, 'unknown')",
        "campaigns" => "COALESCE(utm_campaign, '(none)')",
        _ => return Err(ApiError::NotFound),
    };
    let sql=format!("SELECT {column}::text value,count(*) views,count(DISTINCT visitor_id) visitors FROM events WHERE site_id=$1 AND occurred_at >= $2 AND occurred_at < $3 AND traffic_class='human' GROUP BY 1 ORDER BY views DESC LIMIT $4");
    Ok(Json(
        sqlx::query_as(&sql)
            .bind(site)
            .bind(a)
            .bind(b)
            .bind(q.limit.unwrap_or(100).clamp(1, 500))
            .fetch_all(&s.pool)
            .await?,
    ))
}
async fn visitor_timeline(
    State(s): State<AppState>,
    CurrentUser(u): CurrentUser,
    Path((site, visitor)): Path<(Uuid, String)>,
) -> Result<Json<Vec<Value>>, ApiError> {
    require_site(&s.pool, u, site, false).await?;
    let rows:Vec<(Uuid,DateTime<Utc>,String,String,Value)>=sqlx::query_as("SELECT id,occurred_at,event_name,path,properties FROM events WHERE site_id=$1 AND visitor_id=$2 ORDER BY occurred_at DESC LIMIT 500").bind(site).bind(visitor).fetch_all(&s.pool).await?;
    Ok(Json(
        rows.into_iter()
            .map(|r| json!({"id":r.0,"occurred_at":r.1,"name":r.2,"path":r.3,"properties":r.4}))
            .collect(),
    ))
}
async fn list_visitors(
    State(s): State<AppState>,
    CurrentUser(u): CurrentUser,
    Path(site): Path<Uuid>,
    Query(q): Query<RangeQuery>,
) -> Result<Json<Vec<Value>>, ApiError> {
    require_site(&s.pool, u, site, false).await?;
    let (a, b, _) = bounds(q.from, q.to)?;
    type VisitorRow = (
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        String,
        DateTime<Utc>,
        i64,
    );
    let rows: Vec<VisitorRow> = sqlx::query_as(
        "SELECT visitor_id, max(country_code)::text, max(device_type), max(browser), \
         (array_agg(path ORDER BY occurred_at DESC))[1], max(occurred_at), \
         count(DISTINCT session_id) \
         FROM events WHERE site_id=$1 AND occurred_at >= $2 AND occurred_at < $3 \
         AND traffic_class='human' GROUP BY visitor_id ORDER BY max(occurred_at) DESC LIMIT 500",
    )
    .bind(site)
    .bind(a)
    .bind(b)
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(
        rows.into_iter()
            .map(|row| {
                json!({
                    "id": row.0,
                    "country": row.1.unwrap_or_else(|| "Unknown".into()),
                    "device": row.2,
                    "browser": row.3,
                    "page": row.4,
                    "lastSeen": row.5,
                    "sessions": row.6
                })
            })
            .collect(),
    ))
}
async fn custom_events(
    State(s): State<AppState>,
    CurrentUser(u): CurrentUser,
    Path(site): Path<Uuid>,
    Query(q): Query<RangeQuery>,
) -> Result<Json<Vec<Value>>, ApiError> {
    require_site(&s.pool, u, site, false).await?;
    let (a, b, _) = bounds(q.from, q.to)?;
    type EventRow = (
        Uuid,
        DateTime<Utc>,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
    );
    let rows: Vec<EventRow> = sqlx::query_as(
        "SELECT id,occurred_at,event_name,path,visitor_id,country_code::text,referrer_host \
         FROM events WHERE site_id=$1 AND occurred_at >= $2 AND occurred_at < $3 \
         ORDER BY occurred_at DESC LIMIT 500",
    )
    .bind(site)
    .bind(a)
    .bind(b)
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(
        rows.into_iter()
            .map(|row| {
                json!({
                    "id": row.0,
                    "timestamp": row.1,
                    "type": row.2,
                    "page": row.3,
                    "visitorId": row.4,
                    "country": row.5,
                    "referrer": row.6
                })
            })
            .collect(),
    ))
}
async fn list_goals(
    State(s): State<AppState>,
    CurrentUser(u): CurrentUser,
    Path(site): Path<Uuid>,
) -> Result<Json<Vec<Goal>>, ApiError> {
    require_site(&s.pool, u, site, false).await?;
    Ok(Json(sqlx::query_as("SELECT id,site_id,name,event_name,path_pattern,created_at FROM goals WHERE site_id=$1 ORDER BY created_at").bind(site).fetch_all(&s.pool).await?))
}
async fn create_goal(
    State(s): State<AppState>,
    CurrentUser(u): CurrentUser,
    Path(site): Path<Uuid>,
    Json(i): Json<GoalInput>,
) -> Result<impl IntoResponse, ApiError> {
    require_site(&s.pool, u, site, true).await?;
    if i.name.is_empty() || i.event_name.is_empty() {
        return Err(ApiError::BadRequest("name and event_name required".into()));
    }
    let goal:Goal=sqlx::query_as("INSERT INTO goals(site_id,name,event_name,path_pattern) VALUES($1,$2,$3,$4) RETURNING id,site_id,name,event_name,path_pattern,created_at").bind(site).bind(i.name).bind(i.event_name).bind(i.path_pattern).fetch_one(&s.pool).await.map_err(map_conflict)?;
    Ok((StatusCode::CREATED, Json(goal)))
}
async fn delete_goal(
    State(s): State<AppState>,
    CurrentUser(u): CurrentUser,
    Path((site, goal)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    require_site(&s.pool, u, site, true).await?;
    sqlx::query("DELETE FROM goals WHERE id=$1 AND site_id=$2")
        .bind(goal)
        .bind(site)
        .execute(&s.pool)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
async fn export_csv(
    State(s): State<AppState>,
    CurrentUser(u): CurrentUser,
    Path(site): Path<Uuid>,
    Query(q): Query<RangeQuery>,
) -> Result<impl IntoResponse, ApiError> {
    require_site(&s.pool, u, site, false).await?;
    let (a, b, _) = bounds(q.from, q.to)?;
    let rows: Vec<CsvRow> = sqlx::query_as("SELECT occurred_at,event_name,path,visitor_id,referrer_host,utm_campaign FROM events WHERE site_id=$1 AND occurred_at >= $2 AND occurred_at < $3 ORDER BY occurred_at").bind(site).bind(a).bind(b).fetch_all(&s.pool).await?;
    let mut w = csv::Writer::from_writer(vec![]);
    w.write_record([
        "occurred_at",
        "event_name",
        "path",
        "visitor_id",
        "referrer",
        "campaign",
    ])
    .map_err(|_| ApiError::Internal)?;
    for r in rows {
        w.serialize(r).map_err(|_| ApiError::Internal)?
    }
    let data = w.into_inner().map_err(|_| ApiError::Internal)?;
    Ok((
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=events.csv",
            ),
        ],
        data,
    ))
}
async fn stream(
    State(s): State<AppState>,
    Path(site): Path<Uuid>,
    headers: HeaderMap,
    Query(q): Query<StreamQuery>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let token = q.token.as_deref().ok_or(ApiError::Unauthorized)?;
    let u = verify_token(token, &s.jwt_secret)
        .map_err(|_| ApiError::Unauthorized)?
        .sub;
    require_site(&s.pool, u, site, false).await?;
    let last = q
        .last_event_id
        .or_else(|| {
            headers
                .get("last-event-id")
                .and_then(|v| v.to_str().ok())?
                .parse()
                .ok()
        })
        .unwrap_or(0);
    let replay: Vec<(i64, Value)> = sqlx::query_as(
        "SELECT id,payload FROM stream_events WHERE site_id=$1 AND id>$2 ORDER BY id LIMIT 1001",
    )
    .bind(site)
    .bind(last)
    .fetch_all(&s.pool)
    .await?;
    let resync = replay.len() > 1000;
    let replay = if resync { Vec::new() } else { replay };
    let rx = s.stream_tx.subscribe();
    let initial = tokio_stream::iter(replay.into_iter().map(|(id, payload)| {
        Ok(Event::default()
            .id(id.to_string())
            .event("event")
            .json_data(payload)
            .unwrap())
    }));
    let marker = tokio_stream::iter(resync.then(|| {
        Ok(Event::default()
            .event("resync")
            .data("replay window exceeded"))
    }));
    let live = BroadcastStream::new(rx).filter_map(move |item| match item {
        Ok(m) if m.site_id == site => Some(Ok(Event::default()
            .id(m.id.to_string())
            .event("event")
            .json_data(m.payload)
            .unwrap())),
        Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(_)) => {
            Some(Ok(Event::default()
                .event("resync")
                .data("subscriber lagged")))
        }
        _ => None,
    });
    Ok(Sse::new(initial.chain(marker).chain(live)).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("heartbeat"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forwarded_ip_is_ignored_unless_proxy_is_trusted() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "198.51.100.9, 10.0.0.2".parse().unwrap());
        let peer: IpAddr = "203.0.113.7".parse().unwrap();

        assert_eq!(client_ip(&headers, peer, false), peer);
        assert_eq!(
            client_ip(&headers, peer, true),
            "198.51.100.9".parse::<IpAddr>().unwrap()
        );
    }

    #[test]
    fn malformed_forwarded_ip_falls_back_to_peer() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "not-an-ip".parse().unwrap());
        let peer: IpAddr = "203.0.113.7".parse().unwrap();

        assert_eq!(client_ip(&headers, peer, true), peer);
    }
}
