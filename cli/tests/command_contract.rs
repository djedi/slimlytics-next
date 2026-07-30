use serde_json::Value;
use std::{
    io::{Read, Write},
    net::TcpListener,
    process::Command,
    thread,
};

#[test]
fn site_ensure_reuses_domain_and_emits_agent_contract() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = [0_u8; 8192];
        let length = stream.read(&mut request).unwrap();
        let request = String::from_utf8_lossy(&request[..length]);
        assert!(request.starts_with("POST /api/sites/ensure HTTP/1.1\r\n"));
        assert!(request
            .to_ascii_lowercase()
            .contains("authorization: bearer slyt_test-agent-token\r\n"));
        let body = r#"{"created":false,"site":{"id":"df222f1c-8d95-4917-872e-98b30115aac8","name":"Example","domain":"example.com","timezone":"UTC","allowedOrigins":["https://example.com"],"retentionDays":365,"writeKey":"d8f6f152-7a9e-4eb9-a8a1-468db4c0ea33","antiAdblockServer":"caddy","antiAdblockJsPath":"/456bbb63bb86.js","antiAdblockBeaconPath":"/0d31360a3101","createdAt":"2026-07-29T00:00:00Z"}}"#;
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        )
        .unwrap();
    });

    let output = Command::new(env!("CARGO_BIN_EXE_slimlytics"))
        .args([
            "--json",
            "--api-url",
            &format!("http://{address}"),
            "site",
            "ensure",
            "example.com",
            "--server",
            "caddy",
        ])
        .env("SLIMLYTICS_TOKEN", "slyt_test-agent-token")
        .output()
        .unwrap();
    server.join().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["schemaVersion"], 1);
    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["created"], false);
    assert_eq!(value["data"]["site"]["domain"], "example.com");
    assert!(value["data"]["tracking"]["serverConfig"]
        .as_str()
        .unwrap()
        .contains("rewrite /456bbb63bb86.js /p/"));
    assert_eq!(
        value["data"]["tracking"]["snippet"],
        r#"<script async src="/456bbb63bb86.js"></script>"#
    );
}

#[test]
fn site_ensure_creates_missing_domain_once() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = thread::spawn(move || {
        let (mut create_stream, _) = listener.accept().unwrap();
        let mut request = [0_u8; 8192];
        let length = create_stream.read(&mut request).unwrap();
        let request = String::from_utf8_lossy(&request[..length]);
        assert!(request.starts_with("POST /api/sites/ensure HTTP/1.1\r\n"));
        assert!(request.contains(r#""domain":"new.example.com""#));
        assert!(request.contains(r#""allowedOrigins":["https://new.example.com"]"#));
        let body = r#"{"created":true,"site":{"id":"4ea55444-cf62-489d-a4c3-bc09da805486","name":"new.example.com","domain":"new.example.com","timezone":"UTC","allowedOrigins":["https://new.example.com"],"retentionDays":365,"writeKey":"d8f6f152-7a9e-4eb9-a8a1-468db4c0ea33","antiAdblockServer":"caddy","antiAdblockJsPath":"/456bbb63bb86.js","antiAdblockBeaconPath":"/0d31360a3101","createdAt":"2026-07-29T00:00:00Z"}}"#;
        write!(
            create_stream,
            "HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        )
        .unwrap();
    });

    let output = Command::new(env!("CARGO_BIN_EXE_slimlytics"))
        .args([
            "--json",
            "--api-url",
            &format!("http://{address}"),
            "site",
            "ensure",
            "new.example.com",
        ])
        .env("SLIMLYTICS_TOKEN", "slyt_test-agent-token")
        .output()
        .unwrap();
    server.join().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["data"]["created"], true);
    assert_eq!(value["data"]["site"]["domain"], "new.example.com");
}
