use slimlytics_backend::search_console::{decrypt_token, encrypt_token, preferred_property};

#[test]
fn encrypts_refresh_tokens_with_authenticated_encryption() {
    let key = [7_u8; 32];
    let encrypted = encrypt_token(&key, "refresh-secret").unwrap();

    assert_ne!(encrypted, "refresh-secret");
    assert_eq!(decrypt_token(&key, &encrypted).unwrap(), "refresh-secret");

    let mut tampered = encrypted;
    tampered.push('x');
    assert!(decrypt_token(&key, &tampered).is_err());
}

#[test]
fn prefers_domain_property_then_https_prefix() {
    let properties = vec![
        "https://example.com/".to_string(),
        "sc-domain:example.com".to_string(),
        "https://other.example/".to_string(),
    ];
    assert_eq!(
        preferred_property("example.com", &properties).as_deref(),
        Some("sc-domain:example.com")
    );
    assert_eq!(
        preferred_property("other.example", &properties).as_deref(),
        Some("https://other.example/")
    );
}
