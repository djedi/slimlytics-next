use slimlytics_cli::{
    find_site, normalize_api_url, normalize_domain, save_auth, tracking_setup, Site, StoredAuth,
};
use std::fs;
use uuid::Uuid;

fn site() -> Site {
    Site {
        id: Uuid::parse_str("df222f1c-8d95-4917-872e-98b30115aac8").unwrap(),
        name: "Example".into(),
        domain: "example.com".into(),
        timezone: "UTC".into(),
        allowed_origins: vec!["https://example.com".into()],
        retention_days: 365,
        write_key: Uuid::parse_str("d8f6f152-7a9e-4eb9-a8a1-468db4c0ea33").unwrap(),
        anti_adblock_server: "caddy".into(),
        anti_adblock_js_path: "/456bbb63bb86.js".into(),
        anti_adblock_beacon_path: "/0d31360a3101".into(),
        created_at: "2026-07-29T00:00:00Z".into(),
    }
}

#[test]
fn domains_are_normalized_for_site_creation() {
    assert_eq!(normalize_domain("Example.COM").unwrap(), "example.com");
    assert_eq!(
        normalize_domain("https://example.com/").unwrap(),
        "example.com"
    );
    assert!(normalize_domain("https://example.com/path").is_err());
    assert!(normalize_domain("example.com:8443").is_err());
    assert!(normalize_domain("not a domain").is_err());
}

#[test]
fn a_site_can_be_selected_by_id_or_domain() {
    let sites = vec![site()];
    assert_eq!(find_site(&sites, "example.com").unwrap().id, sites[0].id);
    assert_eq!(
        find_site(&sites, "df222f1c-8d95-4917-872e-98b30115aac8")
            .unwrap()
            .domain,
        "example.com"
    );
    assert!(find_site(&sites, "missing.example").is_err());

    let mut duplicate = site();
    duplicate.id = Uuid::parse_str("a20be759-6926-49e1-ab95-bd2d75f1b883").unwrap();
    assert!(find_site(&[site(), duplicate], "example.com").is_err());
}

#[test]
fn tracking_setup_is_complete_and_ai_friendly() {
    let setup = tracking_setup(&site(), "https://slimlytics.com").unwrap();
    assert_eq!(
        setup.snippet,
        r#"<script async src="/456bbb63bb86.js"></script>"#
    );
    assert!(setup.server_config.contains("handle /456bbb63bb86.js"));
    assert!(setup
        .server_config
        .contains("rewrite /456bbb63bb86.js /p/d8f6f152-7a9e-4eb9-a8a1-468db4c0ea33/0d31360a3101"));
    assert!(setup
        .server_config
        .contains("reverse_proxy https://slimlytics.com"));
    assert!(!setup
        .server_config
        .contains("reverse_proxy https://slimlytics.com/p/"));
    assert!(setup.server_config.contains("header_up -Authorization"));
    assert!(setup
        .server_config
        .contains("/api/collect/d8f6f152-7a9e-4eb9-a8a1-468db4c0ea33"));
    assert_eq!(setup.script_test_url, "https://example.com/456bbb63bb86.js");
    assert_eq!(setup.beacon_test_url, "https://example.com/0d31360a3101");

    let mut unsafe_site = site();
    unsafe_site.anti_adblock_js_path = "/valid.js\nheader injected".into();
    assert!(tracking_setup(&unsafe_site, "https://slimlytics.com").is_err());
}

#[test]
fn auth_file_is_private_and_round_trips() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("auth.json");
    let auth = StoredAuth {
        api_url: "https://slimlytics.com".into(),
        token: "slyt_test-token".into(),
    };
    save_auth(&path, &auth).unwrap();
    let decoded: StoredAuth = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(decoded, auth);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}

#[cfg(unix)]
#[test]
fn auth_file_does_not_follow_symbolic_links() {
    use std::os::unix::fs::symlink;
    let directory = tempfile::tempdir().unwrap();
    let victim = directory.path().join("victim");
    let path = directory.path().join("auth.json");
    fs::write(&victim, "do not overwrite").unwrap();
    symlink(&victim, &path).unwrap();
    let auth = StoredAuth {
        api_url: "https://slimlytics.com".into(),
        token: "slyt_test-token".into(),
    };
    assert!(save_auth(&path, &auth).is_err());
    assert_eq!(fs::read_to_string(victim).unwrap(), "do not overwrite");
}

#[test]
fn plaintext_api_urls_are_limited_to_loopback_development() {
    assert!(normalize_api_url("http://127.0.0.1:8080").is_ok());
    assert!(normalize_api_url("http://localhost:8080").is_ok());
    assert!(normalize_api_url("http://analytics.example.com").is_err());
    assert!(normalize_api_url("https://analytics.example.com").is_ok());
}
