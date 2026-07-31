use axum::http::HeaderMap;
use slimlytics_backend::enrichment::location_from_headers;

#[test]
fn ignores_edge_location_headers_without_proxy_trust() {
    let mut headers = HeaderMap::new();
    headers.insert("cf-ipcountry", "US".parse().unwrap());
    headers.insert("cf-region-code", "CO".parse().unwrap());
    headers.insert("cf-ipcity", "Denver".parse().unwrap());
    headers.insert("cf-ipcontinent", "NA".parse().unwrap());

    assert_eq!(location_from_headers(&headers, false), None);
}

#[test]
fn reads_and_validates_trusted_edge_location_headers() {
    let mut headers = HeaderMap::new();
    headers.insert("cf-ipcountry", "us".parse().unwrap());
    headers.insert("cf-region-code", "CO".parse().unwrap());
    headers.insert("cf-ipcity", "Denver".parse().unwrap());
    headers.insert("cf-ipcontinent", "NA".parse().unwrap());

    let location = location_from_headers(&headers, true).unwrap();
    assert_eq!(location.country_code.as_deref(), Some("US"));
    assert_eq!(location.region.as_deref(), Some("CO"));
    assert_eq!(location.city.as_deref(), Some("Denver"));
    assert_eq!(location.continent.as_deref(), Some("NA"));
}

#[test]
fn drops_invalid_edge_location_values() {
    let mut headers = HeaderMap::new();
    headers.insert("cf-ipcountry", "USA".parse().unwrap());
    headers.insert("cf-region-code", "<script>".parse().unwrap());
    headers.insert("cf-ipcity", "x".repeat(129).parse().unwrap());

    assert_eq!(location_from_headers(&headers, true), None);
}
