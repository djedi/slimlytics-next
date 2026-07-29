use axum::{
    body::{to_bytes, Body},
    extract::ConnectInfo,
    http::{header, Request, StatusCode},
};
use serde_json::{json, Value};
use slimlytics_backend::maintenance::prune_expired_events;
use slimlytics_backend::{app, AppState};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to a disposable PostgreSQL database"]
async fn auth_site_goal_and_collection_flow() {
    let url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL required");
    let pool = PgPoolOptions::new().connect(&url).await.unwrap();
    sqlx::migrate!("../migrations").run(&pool).await.unwrap();
    let router = app(AppState::new(
        pool.clone(),
        "01234567890123456789012345678901".into(),
        b"abcdefghijklmnopqrstuvwxyz012345".to_vec(),
    ));

    let email = format!("owner+{}@example.com", uuid::Uuid::new_v4());
    let register = router
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/auth/register",
            None,
            json!({"email":email,"password":"long-enough-password"}),
        ))
        .await
        .unwrap();
    assert_eq!(register.status(), StatusCode::CREATED);
    let token = body_json(register.into_body()).await["token"]
        .as_str()
        .unwrap()
        .to_owned();

    let created = router.clone().oneshot(json_request("POST", "/api/sites", Some(&token), json!({"name":"Example","domain":"example.com","timezone":"UTC","allowed_origins":["https://example.com"],"retention_days":30}))).await.unwrap();
    assert_eq!(created.status(), StatusCode::CREATED);
    let site = body_json(created.into_body()).await;
    let site_id = site["id"].as_str().unwrap();
    let site_uuid = uuid::Uuid::parse_str(site_id).unwrap();
    let write_key = site["writeKey"].as_str().unwrap();

    let preflight = Request::builder()
        .method("OPTIONS")
        .uri(format!("/api/collect/{write_key}"))
        .header(header::ORIGIN, "https://example.com")
        .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
        .body(Body::empty())
        .unwrap();
    let preflight = router.clone().oneshot(preflight).await.unwrap();
    assert_eq!(preflight.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        preflight.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN],
        "https://example.com"
    );

    let goal = router
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/sites/{site_id}/goals"),
            Some(&token),
            json!({"name":"Signup","event_name":"signup","path_pattern":"/thanks%"}),
        ))
        .await
        .unwrap();
    assert_eq!(goal.status(), StatusCode::CREATED);

    let mut collect = json_request(
        "POST",
        &format!("/api/collect/{write_key}"),
        None,
        json!({"name":"signup","url":"https://example.com/thanks?utm_source=newsletter&email=private@example.com#secret"}),
    );
    collect
        .headers_mut()
        .insert(header::ORIGIN, "https://example.com".parse().unwrap());
    collect
        .headers_mut()
        .insert(header::USER_AGENT, "Mozilla/5.0".parse().unwrap());
    collect.extensions_mut().insert(ConnectInfo(
        "203.0.113.10:443".parse::<std::net::SocketAddr>().unwrap(),
    ));
    let collected = router.clone().oneshot(collect).await.unwrap();
    assert_eq!(collected.status(), StatusCode::ACCEPTED);
    assert_eq!(
        collected.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN],
        "https://example.com"
    );

    let stored: (String, Option<String>) =
        sqlx::query_as("SELECT url,utm_source FROM events WHERE site_id=$1 LIMIT 1")
            .bind(site_uuid)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(stored.0, "https://example.com/thanks");
    assert_eq!(stored.1.as_deref(), Some("newsletter"));
    let completions: i64 =
        sqlx::query_scalar("SELECT count(*) FROM goal_completions WHERE site_id=$1")
            .bind(site_uuid)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(completions, 1);

    let today = chrono::Utc::now().date_naive();
    for path in [
        format!("/api/sites/{site_id}/overview?from={today}&to={today}"),
        format!("/api/sites/{site_id}/visitors?from={today}&to={today}"),
        format!("/api/sites/{site_id}/events?from={today}&to={today}"),
    ] {
        let response = router
            .clone()
            .oneshot(json_request("GET", &path, Some(&token), json!({})))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK, "{path}");
        let body = body_json(response.into_body()).await;
        assert_ne!(body, json!([]), "{path} should expose the ingested event");
    }

    sqlx::query("UPDATE sites SET retention_days=1 WHERE id=$1")
        .bind(site_uuid)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE events SET occurred_at=now()-interval '2 days' WHERE site_id=$1")
        .bind(site_uuid)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO events(site_id,visitor_id,session_id,event_name,url,path) VALUES($1,'fresh-visitor','fresh-session','pageview','https://example.com/fresh','/fresh')")
        .bind(site_uuid)
        .execute(&pool)
        .await
        .unwrap();
    for suffix in ["a", "b"] {
        sqlx::query("INSERT INTO events(site_id,occurred_at,visitor_id,session_id,event_name,url,path) VALUES($1,now()-interval '3 days',$2,$3,'pageview','https://example.com/old','/old')")
            .bind(site_uuid)
            .bind(format!("old-visitor-{suffix}"))
            .bind(format!("old-session-{suffix}"))
            .execute(&pool)
            .await
            .unwrap();
    }
    let longer_retention_site: uuid::Uuid = sqlx::query_scalar("INSERT INTO sites(name,domain,retention_days) VALUES($1,'retained.example',10) RETURNING id")
        .bind(format!("retained-{}", uuid::Uuid::new_v4()))
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO events(site_id,occurred_at,visitor_id,session_id,event_name,url,path) VALUES($1,now()-interval '2 days','retained-visitor','retained-session','pageview','https://retained.example/','/')")
        .bind(longer_retention_site)
        .execute(&pool)
        .await
        .unwrap();

    assert_eq!(prune_expired_events(&pool, 1).await.unwrap(), 1);
    assert_eq!(prune_expired_events(&pool, 1).await.unwrap(), 1);
    assert_eq!(prune_expired_events(&pool, 1).await.unwrap(), 1);
    assert_eq!(prune_expired_events(&pool, 1).await.unwrap(), 0);
    let remaining: i64 = sqlx::query_scalar("SELECT count(*) FROM events WHERE site_id=$1")
        .bind(site_uuid)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(remaining, 1, "unexpired event must be preserved");
    let retained: i64 = sqlx::query_scalar("SELECT count(*) FROM events WHERE site_id=$1")
        .bind(longer_retention_site)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(retained, 1, "site-specific retention must be honored");
    let dependent_rows: i64 = sqlx::query_scalar("SELECT (SELECT count(*) FROM goal_completions WHERE site_id=$1) + (SELECT count(*) FROM stream_events WHERE site_id=$1)")
        .bind(site_uuid)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(dependent_rows, 0, "dependent rows must cascade on prune");
}

fn json_request(method: &str, uri: &str, token: Option<&str>, body: Value) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    builder.body(Body::from(body.to_string())).unwrap()
}
async fn body_json(body: Body) -> Value {
    serde_json::from_slice(&to_bytes(body, usize::MAX).await.unwrap()).unwrap()
}
