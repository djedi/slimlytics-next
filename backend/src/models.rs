use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

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
    pub created_at: DateTime<Utc>,
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
#[derive(Debug, Serialize)]
pub struct Metric {
    pub current: i64,
    pub previous: i64,
    pub change_percent: Option<f64>,
}
#[derive(Debug, Serialize)]
pub struct Overview {
    pub views: Metric,
    pub visitors: Metric,
    pub sessions: Metric,
    pub events: Metric,
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
