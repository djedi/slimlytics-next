use slimlytics_backend::server_ingest::{validate_server_event, ServerEvent};

fn event() -> ServerEvent {
    ServerEvent {
        idempotency_key: Some("edge-request-123".into()),
        url: "https://example.com/article?utm_source=agent".into(),
        user_agent: "GPTBot/1.0".into(),
        client_ip: "203.0.113.20".into(),
        occurred_at: None,
        referrer: None,
        method: Some("GET".into()),
        status: Some(200),
        event_name: None,
    }
}

#[test]
fn accepts_bounded_server_request_metadata() {
    let validated = validate_server_event(event(), "example.com").unwrap();
    assert_eq!(validated.url.host_str(), Some("example.com"));
    assert_eq!(validated.event_name, "pageview");
    assert_eq!(validated.method.as_deref(), Some("GET"));
}

#[test]
fn rejects_cross_domain_private_or_malformed_metadata() {
    let mut input = event();
    input.url = "https://attacker.example/path".into();
    assert!(validate_server_event(input, "example.com").is_err());

    let mut input = event();
    input.client_ip = "not-an-ip".into();
    assert!(validate_server_event(input, "example.com").is_err());

    let mut input = event();
    input.method = Some("TRACE".into());
    assert!(validate_server_event(input, "example.com").is_err());

    let mut input = event();
    input.status = Some(999);
    assert!(validate_server_event(input, "example.com").is_err());
}
