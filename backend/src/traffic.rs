use dashmap::DashMap;
use std::{
    net::IpAddr,
    time::{Duration, Instant},
};

pub fn traffic_class(user_agent: &str, ip: IpAddr, internal: &[IpAddr]) -> &'static str {
    if internal.contains(&ip) {
        return "internal";
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

pub fn client_metadata(user_agent: &str) -> (&'static str, &'static str) {
    let browser = if user_agent.contains("Firefox/") {
        "Firefox"
    } else if user_agent.contains("Edg/") {
        "Edge"
    } else if user_agent.contains("Chrome/") {
        "Chrome"
    } else if user_agent.contains("Safari/") {
        "Safari"
    } else {
        "Other"
    };
    let os = if user_agent.contains("Android") {
        "Android"
    } else if user_agent.contains("iPhone") || user_agent.contains("iPad") {
        "iOS"
    } else if user_agent.contains("Macintosh") {
        "macOS"
    } else if user_agent.contains("Windows") {
        "Windows"
    } else if user_agent.contains("Linux") {
        "Linux"
    } else {
        "Other"
    };
    (browser, os)
}

pub fn origin_allowed(origin: Option<&str>, allowed: &[String]) -> bool {
    origin.is_some_and(|value| allowed.iter().any(|allowed| allowed == value))
}
