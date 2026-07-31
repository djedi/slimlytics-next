use slimlytics_backend::webhooks::{is_public_ip, validate_webhook_url};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[test]
fn webhook_urls_require_https_without_embedded_credentials() {
    assert!(validate_webhook_url("https://hooks.example.com/slimlytics").is_ok());
    assert!(validate_webhook_url("http://hooks.example.com/slimlytics").is_err());
    assert!(validate_webhook_url("https://user:secret@hooks.example.com/").is_err());
    assert!(validate_webhook_url("https://hooks.example.com/#secret").is_err());
    assert!(validate_webhook_url("https://localhost/hook").is_err());
}

#[test]
fn webhook_delivery_rejects_non_public_networks() {
    for ip in [
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
        IpAddr::V4(Ipv4Addr::new(169, 254, 169, 254)),
        IpAddr::V4(Ipv4Addr::new(100, 64, 0, 1)),
        IpAddr::V6(Ipv6Addr::LOCALHOST),
        "fc00::1".parse().unwrap(),
    ] {
        assert!(!is_public_ip(ip), "{ip} must be blocked");
    }
    assert!(is_public_ip("8.8.8.8".parse().unwrap()));
    assert!(is_public_ip("2606:4700:4700::1111".parse().unwrap()));
}
