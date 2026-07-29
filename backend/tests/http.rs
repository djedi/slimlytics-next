use axum::{body::Body, http::Request};
use slimlytics_backend::{app, AppState};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;

fn state() -> AppState {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://unused:unused@localhost/unused")
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
