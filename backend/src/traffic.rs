use dashmap::DashMap;
use std::{
    net::IpAddr,
    time::{Duration, Instant},
};
use woothee::parser::Parser;

pub fn traffic_class(user_agent: &str, ip: IpAddr, internal: &[IpAddr]) -> &'static str {
    if internal.contains(&ip) {
        return "internal";
    }
    if automation_metadata(user_agent).is_some() {
        return "bot";
    }
    let ua = user_agent.to_ascii_lowercase();
    if ["bot", "spider", "crawler", "headless", "preview"]
        .iter()
        .any(|v| ua.contains(v))
    {
        "bot"
    } else {
        "human"
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AutomationMetadata {
    pub name: &'static str,
    pub category: &'static str,
}

pub fn automation_metadata(user_agent: &str) -> Option<AutomationMetadata> {
    let agents = [
        ("OAI-SearchBot", "OAI-SearchBot", "ai-crawler"),
        ("ChatGPT-User", "ChatGPT-User", "ai-crawler"),
        ("GPTBot", "GPTBot", "ai-crawler"),
        ("ClaudeBot", "ClaudeBot", "ai-crawler"),
        ("Claude-User", "Claude-User", "ai-crawler"),
        ("PerplexityBot", "PerplexityBot", "ai-crawler"),
        ("Perplexity-User", "Perplexity-User", "ai-crawler"),
        ("Google-Extended", "Google-Extended", "ai-crawler"),
        ("Bytespider", "Bytespider", "ai-crawler"),
        ("Googlebot", "Googlebot", "crawler"),
        ("Bingbot", "Bingbot", "crawler"),
    ];
    agents
        .into_iter()
        .find(|(needle, _, _)| user_agent.contains(needle))
        .map(|(_, name, category)| AutomationMetadata { name, category })
}

#[derive(Clone)]
pub struct RateLimiter {
    hits: std::sync::Arc<DashMap<String, (Instant, u32)>>,
    limit: u32,
    window: Duration,
}
impl RateLimiter {
    pub fn new(limit: u32, window: Duration) -> Self {
        Self {
            hits: Default::default(),
            limit,
            window,
        }
    }
    pub fn check(&self, key: &str) -> bool {
        let now = Instant::now();
        let mut entry = self.hits.entry(key.to_owned()).or_insert((now, 0));
        if now.duration_since(entry.0) >= self.window {
            *entry = (now, 0);
        }
        entry.1 += 1;
        entry.1 <= self.limit
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientMetadata {
    pub browser: String,
    pub browser_version: Option<String>,
    pub os: String,
    pub os_version: Option<String>,
    pub device_type: &'static str,
}

pub fn client_metadata(user_agent: &str) -> ClientMetadata {
    let parsed = Parser::new().parse(user_agent);
    let value = |raw: &str| (raw != "UNKNOWN" && !raw.is_empty()).then(|| raw.to_owned());
    ClientMetadata {
        browser: parsed
            .as_ref()
            .and_then(|result| value(result.name))
            .unwrap_or_else(|| "Other".into()),
        browser_version: parsed.as_ref().and_then(|result| value(result.version)),
        os: parsed
            .as_ref()
            .and_then(|result| value(result.os))
            .unwrap_or_else(|| "Other".into()),
        os_version: parsed
            .as_ref()
            .and_then(|result| value(result.os_version.as_ref())),
        device_type: match parsed.as_ref().map(|result| result.category) {
            Some("smartphone" | "mobilephone") => "mobile",
            Some("tablet") => "tablet",
            _ => "desktop",
        },
    }
}

pub fn origin_allowed(origin: Option<&str>, allowed: &[String]) -> bool {
    origin.is_some_and(|value| allowed.iter().any(|allowed| allowed == value))
}

/// Allow collection when Origin matches, or when Origin is absent but Referer is an allowlisted origin.
/// Some mobile browsers omit Origin on same-origin beacon/fetch while still sending Referer.
pub fn collection_origin_allowed(
    origin: Option<&str>,
    referer: Option<&str>,
    allowed: &[String],
) -> bool {
    if origin_allowed(origin, allowed) {
        return true;
    }
    if origin.is_some() {
        return false;
    }
    referer
        .and_then(|value| url::Url::parse(value).ok())
        .and_then(|url| {
            let host = url.host_str()?;
            Some(format!("{}://{}", url.scheme(), host))
        })
        .is_some_and(|ref_origin| allowed.iter().any(|item| item == &ref_origin))
}
