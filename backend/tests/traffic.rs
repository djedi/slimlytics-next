use slimlytics_backend::traffic::{client_metadata, origin_allowed, traffic_class, RateLimiter};
use std::{net::IpAddr, time::Duration};

#[test]
fn origin_must_exactly_match_allowlist() {
    let allowed = vec!["https://example.com".to_string()];
    assert!(origin_allowed(Some("https://example.com"), &allowed));
    assert!(!origin_allowed(Some("https://evil.example"), &allowed));
    assert!(!origin_allowed(None, &allowed));
}

#[test]
fn classifies_bots_and_internal_addresses() {
    let internal: IpAddr = "10.0.0.1".parse().unwrap();
    assert_eq!(
        traffic_class("Googlebot", "1.1.1.1".parse().unwrap(), &[]),
        "bot"
    );
    assert_eq!(traffic_class("Mozilla", internal, &[internal]), "internal");
    assert_eq!(
        traffic_class("Mozilla", "1.1.1.1".parse().unwrap(), &[]),
        "human"
    );
}

#[test]
fn extracts_coarse_client_metadata() {
    let (browser, os) = client_metadata("Mozilla/5.0 (Macintosh) AppleWebKit Chrome/126");
    assert_eq!(browser, "Chrome");
    assert_eq!(os, "macOS");
}

#[test]
fn rate_limiter_rejects_over_limit() {
    let limiter = RateLimiter::new(2, Duration::from_secs(60));
    assert!(limiter.check("key"));
    assert!(limiter.check("key"));
    assert!(!limiter.check("key"));
}
