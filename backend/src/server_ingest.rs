use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::net::IpAddr;
use url::Url;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerEvent {
    pub idempotency_key: Option<String>,
    pub url: String,
    pub user_agent: String,
    pub client_ip: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub referrer: Option<String>,
    pub method: Option<String>,
    pub status: Option<u16>,
    pub event_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ServerEventBatch {
    pub events: Vec<ServerEvent>,
}

pub struct ValidatedServerEvent {
    pub idempotency_key: Option<String>,
    pub url: Url,
    pub user_agent: String,
    pub client_ip: IpAddr,
    pub occurred_at: DateTime<Utc>,
    pub referrer: Option<String>,
    pub method: Option<String>,
    pub status: Option<u16>,
    pub event_name: String,
}

pub fn validate_server_event(
    input: ServerEvent,
    site_domain: &str,
) -> Result<ValidatedServerEvent, &'static str> {
    let url = Url::parse(&input.url).map_err(|_| "invalid url")?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str() != Some(site_domain) {
        return Err("url must use the site's domain");
    }
    if input.user_agent.is_empty() || input.user_agent.len() > 1024 {
        return Err("invalid userAgent");
    }
    let client_ip = input
        .client_ip
        .parse::<IpAddr>()
        .map_err(|_| "invalid clientIp")?;
    let occurred_at = input.occurred_at.unwrap_or_else(Utc::now);
    let age = Utc::now() - occurred_at;
    if age.num_days() > 7 || age.num_minutes() < -5 {
        return Err("occurredAt outside accepted window");
    }
    let method = input.method.map(|value| value.to_ascii_uppercase());
    if method
        .as_deref()
        .is_some_and(|value| !matches!(value, "GET" | "HEAD"))
    {
        return Err("only GET and HEAD requests can be ingested");
    }
    if input
        .status
        .is_some_and(|value| !(100..=599).contains(&value))
    {
        return Err("invalid status");
    }
    let event_name = input.event_name.unwrap_or_else(|| "pageview".into());
    if event_name.is_empty()
        || event_name.len() > 64
        || !event_name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
    {
        return Err("invalid eventName");
    }
    if input
        .referrer
        .as_ref()
        .is_some_and(|value| value.len() > 2048)
    {
        return Err("invalid referrer");
    }
    if input.idempotency_key.as_ref().is_some_and(|value| {
        !(8..=128).contains(&value.len())
            || !value.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':')
            })
    }) {
        return Err("invalid idempotencyKey");
    }
    Ok(ValidatedServerEvent {
        idempotency_key: input.idempotency_key,
        url,
        user_agent: input.user_agent,
        client_ip,
        occurred_at,
        referrer: input.referrer,
        method,
        status: input.status,
        event_name,
    })
}
