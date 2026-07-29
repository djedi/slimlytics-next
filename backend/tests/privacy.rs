use slimlytics_backend::privacy::sanitize_url;

#[test]
fn strips_query_and_fragment_but_keeps_path() {
    assert_eq!(
        sanitize_url("https://example.com/docs/page?email=a%40b.com&utm_source=x#private").unwrap(),
        "https://example.com/docs/page"
    );
}

#[test]
fn rejects_non_http_urls() {
    assert!(sanitize_url("javascript:alert(1)").is_err());
}
