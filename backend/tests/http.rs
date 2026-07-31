use axum::{
    body::{to_bytes, Body},
    http::Request,
};
use slimlytics_backend::{app, AppState};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;

fn state() -> AppState {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://unused:***@localhost/unused")
        .unwrap();
    AppState::new(
        pool,
        "test-secret-at-least-32-characters".into(),
        b"identity-secret".to_vec(),
    )
}

#[tokio::test]
async fn liveness_does_not_depend_on_database() {
    let response = app(state())
        .oneshot(Request::get("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn protected_routes_require_bearer_token() {
    let response = app(state())
        .oneshot(Request::get("/api/sites").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn neutral_collection_alias_is_routed() {
    let response = app(state())
        .oneshot(
            Request::post("/api/e/not-a-uuid")
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn collection_proxy_test_is_routed() {
    let response = app(state())
        .oneshot(
            Request::get("/api/collect/not-a-uuid")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn anti_adblock_configuration_requires_authentication() {
    let response = app(state())
        .oneshot(
            Request::put("/api/sites/00000000-0000-4000-8000-000000000000/anti-adblock")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"serverType":"caddy","jsPath":"/456bbb63bb86.js","beaconPath":"/0d31360a3101"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn api_token_management_requires_a_session() {
    let response = app(state())
        .oneshot(
            Request::post("/api/account/tokens")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"agent"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn openapi_document_covers_every_public_backend_route() {
    let response = app(state())
        .oneshot(
            Request::get("/api/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(
        response.headers()["content-type"],
        "application/vnd.oai.openapi+json"
    );
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let document: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(document["openapi"], "3.1.0");
    assert_eq!(document["info"]["title"], "Slimlytics API");

    let required_operations = [
        ("/health", "get"),
        ("/ready", "get"),
        ("/api/openapi.json", "get"),
        ("/api/docs", "get"),
        ("/api/auth/register", "post"),
        ("/api/auth/login", "post"),
        ("/api/auth/me", "get"),
        ("/api/account/tokens", "get"),
        ("/api/account/tokens", "post"),
        ("/api/account/tokens/current", "delete"),
        ("/api/account/tokens/{tokenId}", "delete"),
        ("/api/sites", "get"),
        ("/api/sites", "post"),
        ("/api/sites/ensure", "post"),
        ("/api/sites/{siteId}", "get"),
        ("/api/sites/{siteId}", "put"),
        ("/api/sites/{siteId}", "delete"),
        ("/api/sites/{siteId}/rotate-key", "post"),
        ("/api/sites/{siteId}/rotate-server-key", "post"),
        ("/api/sites/{siteId}/anti-adblock", "put"),
        ("/api/sites/{siteId}/collection-health", "get"),
        ("/api/sites/{siteId}/overview", "get"),
        ("/api/sites/{siteId}/reports/{dimension}", "get"),
        ("/api/sites/{siteId}/insights/journeys", "get"),
        ("/api/sites/{siteId}/insights/attribution", "get"),
        ("/api/sites/{siteId}/insights/anomalies", "get"),
        ("/api/sites/{siteId}/annotations", "get"),
        ("/api/sites/{siteId}/annotations", "post"),
        ("/api/sites/{siteId}/annotations/{annotationId}", "delete"),
        ("/api/sites/{siteId}/funnels", "get"),
        ("/api/sites/{siteId}/funnels", "post"),
        ("/api/sites/{siteId}/funnels/{funnelId}", "delete"),
        ("/api/sites/{siteId}/funnels/{funnelId}/report", "get"),
        ("/api/sites/{siteId}/report-subscriptions", "get"),
        ("/api/sites/{siteId}/report-subscriptions", "post"),
        (
            "/api/sites/{siteId}/report-subscriptions/{subscriptionId}",
            "put",
        ),
        (
            "/api/sites/{siteId}/report-subscriptions/{subscriptionId}",
            "delete",
        ),
        (
            "/api/sites/{siteId}/report-subscriptions/{subscriptionId}/deliver",
            "post",
        ),
        (
            "/api/sites/{siteId}/report-subscriptions/{subscriptionId}/deliveries",
            "get",
        ),
        ("/api/sites/{siteId}/integrations/search-console", "get"),
        ("/api/sites/{siteId}/integrations/search-console", "delete"),
        (
            "/api/sites/{siteId}/integrations/search-console/connect",
            "post",
        ),
        (
            "/api/sites/{siteId}/integrations/search-console/sync",
            "post",
        ),
        ("/api/sites/{siteId}/reports/search-console", "get"),
        ("/api/integrations/search-console/callback", "get"),
        ("/api/mcp", "post"),
        ("/api/sites/{siteId}/visitors", "get"),
        ("/api/sites/{siteId}/visitors/{visitorId}", "get"),
        ("/api/sites/{siteId}/events", "get"),
        ("/api/sites/{siteId}/goals", "get"),
        ("/api/sites/{siteId}/goals", "post"),
        ("/api/sites/{siteId}/goals/{goalId}", "delete"),
        ("/api/sites/{siteId}/export.csv", "get"),
        ("/api/sites/{siteId}/stream", "get"),
        ("/api/collect/{writeKey}", "post"),
        ("/api/collect/{writeKey}", "options"),
        ("/api/e/{writeKey}", "post"),
        ("/api/e/{writeKey}", "options"),
        ("/api/ingest", "post"),
    ];
    for (path, method) in required_operations {
        assert!(
            document["paths"][path].get(method).is_some(),
            "OpenAPI is missing {method} {path}"
        );
    }
    for item in document["paths"].as_object().unwrap().values() {
        for operation in item.as_object().unwrap().values() {
            if let Some(responses) = operation.get("responses") {
                assert!(responses.get("409").is_none(), "API emits 400, not 409");
            }
        }
    }
    assert_eq!(
        document["paths"]["/api/sites/{siteId}/rotate-key"]["post"]["responses"]["200"]["content"]
            ["application/json"]["schema"]["$ref"],
        "#/components/schemas/WriteKeyResponse"
    );
    assert_eq!(
        document["paths"]["/api/sites/{siteId}/visitors/{visitorId}"]["get"]["responses"]["200"]
            ["content"]["application/json"]["schema"]["type"],
        "array"
    );
    let tracker = &document["paths"]["/p/{writeKey}/{beaconName}"]["get"]["responses"];
    assert!(tracker.get("304").is_some());
    assert!(tracker.get("404").is_none());
    assert!(tracker["200"]["content"].get("text/javascript").is_some());
    assert!(tracker["400"]["content"].get("text/plain").is_some());
    assert_eq!(
        document["components"]["securitySchemes"]["bearerAuth"]["type"],
        "http"
    );
    assert_eq!(
        document["paths"]["/health"]["get"]["responses"]
            .as_object()
            .unwrap()
            .keys()
            .collect::<Vec<_>>(),
        vec!["200"]
    );
    assert_eq!(
        document["components"]["schemas"]["Visitor"]["additionalProperties"],
        false
    );
    assert_eq!(
        document["components"]["schemas"]["Event"]["additionalProperties"],
        false
    );
    let get_site_parameters = document["paths"]["/api/sites/{siteId}"]["get"]["parameters"]
        .as_array()
        .unwrap();
    let site_id = get_site_parameters
        .iter()
        .find(|parameter| parameter["name"] == "siteId")
        .unwrap();
    assert_eq!(site_id["schema"]["format"], "uuid");
    let collect_responses = &document["paths"]["/api/collect/{writeKey}"]["post"]["responses"];
    assert!(collect_responses["400"]["content"]
        .get("text/plain")
        .is_some());
    assert!(collect_responses["415"]["content"]
        .get("text/plain")
        .is_some());
    assert!(collect_responses["422"]["content"]
        .get("text/plain")
        .is_some());
    for path in ["/api/collect/{writeKey}", "/api/e/{writeKey}"] {
        assert!(
            document["paths"][path]["get"]["responses"]["403"]["content"]
                .get("application/json")
                .is_some()
        );
    }
}

#[tokio::test]
async fn scalar_ui_redirects_to_the_locally_bundled_reference() {
    let response = app(state())
        .oneshot(Request::get("/api/docs").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), 307);
    assert_eq!(response.headers()["location"], "/docs/api");
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(!html.contains("cdn.jsdelivr.net"));
}
