use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteKeyResponse {
    pub write_key: Uuid,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Site {
    pub id: Uuid,
    pub name: String,
    pub domain: String,
    pub timezone: String,
    pub allowed_origins: Vec<String>,
    pub retention_days: i32,
    pub write_key: Uuid,
    pub server_write_key: Uuid,
    pub anti_adblock_server: String,
    pub anti_adblock_js_path: String,
    pub anti_adblock_beacon_path: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnsureSiteResponse {
    pub created: bool,
    pub site: Site,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteInput {
    pub name: String,
    pub domain: String,
    #[serde(default = "utc")]
    pub timezone: String,
    #[serde(default)]
    #[serde(alias = "allowed_origins")]
    pub allowed_origins: Vec<String>,
    #[serde(default = "retention")]
    #[serde(alias = "retention_days")]
    pub retention_days: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AntiAdblockInput {
    pub server_type: String,
    pub js_path: String,
    pub beacon_path: String,
}

impl AntiAdblockInput {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !matches!(self.server_type.as_str(), "caddy" | "nginx" | "apache") {
            return Err("unsupported server type");
        }
        if !valid_proxy_path(&self.js_path, true) {
            return Err("invalid JavaScript path");
        }
        if !valid_proxy_path(&self.beacon_path, false) {
            return Err("invalid beacon path");
        }
        if self.js_path == self.beacon_path {
            return Err("proxy paths must be different");
        }
        Ok(())
    }
}

fn valid_proxy_path(value: &str, javascript: bool) -> bool {
    let Some(name) = value.strip_prefix('/') else {
        return false;
    };
    if name.contains('/') || name.len() > 67 {
        return false;
    }
    let stem = if javascript {
        let Some(stem) = name.strip_suffix(".js") else {
            return false;
        };
        stem
    } else {
        name
    };
    let max_stem_len = if javascript { 63 } else { 64 };
    (6..=max_stem_len).contains(&stem.len())
        && stem
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'~' | b'-'))
}
fn utc() -> String {
    "UTC".into()
}
fn retention() -> i32 {
    365
}

#[derive(Debug, Deserialize)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub token: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiTokenInput {
    pub name: String,
    pub expires_in_days: Option<i64>,
    #[serde(default = "default_api_scopes")]
    pub scopes: Vec<String>,
}
fn default_api_scopes() -> Vec<String> {
    vec!["sites:read".into(), "analytics:read".into()]
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ApiTokenSummary {
    pub id: Uuid,
    pub name: String,
    pub token_prefix: String,
    pub scopes: Vec<String>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiTokenCreated {
    pub id: Uuid,
    pub name: String,
    pub token_prefix: String,
    pub scopes: Vec<String>,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectInput {
    #[serde(default = "pageview")]
    pub name: String,
    pub url: String,
    pub referrer: Option<String>,
    pub title: Option<String>,
    pub occurred_at: Option<DateTime<Utc>>,
    #[serde(default = "empty_object")]
    pub properties: Value,
    pub screen_width: Option<u32>,
    pub privacy_control: Option<String>,
    pub tracker_version: Option<String>,
}
fn pageview() -> String {
    "pageview".into()
}
fn empty_object() -> Value {
    Value::Object(Default::default())
}

#[derive(Debug, Deserialize)]
pub struct RangeQuery {
    pub from: NaiveDate,
    pub to: NaiveDate,
}
#[derive(Debug, Deserialize)]
pub struct ReportQuery {
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub limit: Option<i64>,
}
#[derive(Debug, Deserialize)]
pub struct StreamQuery {
    #[serde(alias = "lastEventId")]
    pub last_event_id: Option<i64>,
    pub token: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ReportRow {
    pub value: String,
    pub views: i64,
    pub visitors: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnotationInput {
    pub occurred_on: NaiveDate,
    pub label: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Annotation {
    pub id: Uuid,
    pub site_id: Uuid,
    pub occurred_on: NaiveDate,
    pub label: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FunnelStep {
    pub label: String,
    pub event_name: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunnelInput {
    pub name: String,
    pub steps: Vec<FunnelStep>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportSubscriptionInput {
    pub name: String,
    pub webhook_url: String,
    pub frequency: String,
    #[serde(default)]
    pub anomaly_only: bool,
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ReportSubscription {
    pub id: Uuid,
    pub site_id: Uuid,
    pub name: String,
    pub webhook_url: String,
    pub frequency: String,
    pub anomaly_only: bool,
    pub enabled: bool,
    pub next_run_at: DateTime<Utc>,
    pub last_sent_at: Option<DateTime<Utc>>,
    pub last_status: Option<String>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ReportDelivery {
    pub id: i64,
    pub status: String,
    pub response_status: Option<i32>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SearchConsoleCallbackQuery {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchConsoleReportQuery {
    pub from: NaiveDate,
    pub to: NaiveDate,
    #[serde(default = "search_query_dimension")]
    pub dimension: String,
    pub limit: Option<i64>,
}

fn search_query_dimension() -> String {
    "query".into()
}
#[derive(Debug, Serialize)]
pub struct Metric {
    pub current: i64,
    pub previous: i64,
    pub change_percent: Option<f64>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Overview {
    pub views: Metric,
    pub visitors: Metric,
    pub sessions: Metric,
    pub events: Metric,
    /// Distinct human visitors with an event in the last five minutes.
    pub current_online: i64,
    pub bounce_rate: f64,
    pub avg_duration_seconds: f64,
    pub trend: Vec<TrendPoint>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TrendPoint {
    pub date: NaiveDate,
    pub visitors: i64,
    pub page_views: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CollectionHealth {
    pub accepted_total: i64,
    pub rejected_total: i64,
    pub last_accepted_at: Option<DateTime<Utc>>,
    pub last_rejected_at: Option<DateTime<Utc>>,
    pub last_rejection_code: Option<String>,
    pub last_tracker_version: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalInput {
    pub name: String,
    #[serde(alias = "event_name")]
    pub event_name: String,
    #[serde(alias = "path_pattern")]
    pub path_pattern: Option<String>,
}
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Goal {
    pub id: Uuid,
    pub site_id: Uuid,
    pub name: String,
    pub event_name: String,
    pub path_pattern: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct GoalWithStats {
    pub id: Uuid,
    pub site_id: Uuid,
    pub name: String,
    pub event_name: String,
    pub path_pattern: Option<String>,
    pub created_at: DateTime<Utc>,
    pub conversions: i64,
    pub conversion_rate: f64,
}
