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
    let session_token = body_json(register.into_body()).await["token"]
        .as_str()
        .unwrap()
        .to_owned();

    let created_token = router
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/account/tokens",
            Some(&session_token),
            json!({"name":"e2e agent","expiresInDays":30}),
        ))
        .await
        .unwrap();
    assert_eq!(created_token.status(), StatusCode::CREATED);
    let created_token = body_json(created_token.into_body()).await;
    let token_id = created_token["id"].as_str().unwrap();
    let token = created_token["token"].as_str().unwrap().to_owned();
    assert!(token.starts_with("slyt_"));
    assert!(created_token["tokenPrefix"].as_str().unwrap().len() >= 9);
    let stored_hash: Vec<u8> = sqlx::query_scalar("SELECT token_hash FROM api_tokens WHERE id=$1")
        .bind(uuid::Uuid::parse_str(token_id).unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(stored_hash.len(), 32);
    assert_ne!(stored_hash, token.as_bytes());

    let token_cannot_mint_tokens = router
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/account/tokens",
            Some(&token),
            json!({"name":"forbidden child token"}),
        ))
        .await
        .unwrap();
    assert_eq!(token_cannot_mint_tokens.status(), StatusCode::UNAUTHORIZED);

    let expired_token = router
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/account/tokens",
            Some(&session_token),
            json!({"name":"expiry test","expiresInDays":1}),
        ))
        .await
        .unwrap();
    assert_eq!(expired_token.status(), StatusCode::CREATED);
    let expired_token = body_json(expired_token.into_body()).await;
    sqlx::query("UPDATE api_tokens SET created_at=now()-interval '2 seconds', expires_at=now()-interval '1 second' WHERE id=$1")
        .bind(uuid::Uuid::parse_str(expired_token["id"].as_str().unwrap()).unwrap())
        .execute(&pool)
        .await
        .unwrap();
    let expired_rejected = router
        .clone()
        .oneshot(json_request(
            "GET",
            "/api/auth/me",
            Some(expired_token["token"].as_str().unwrap()),
            json!({}),
        ))
        .await
        .unwrap();
    assert_eq!(expired_rejected.status(), StatusCode::UNAUTHORIZED);

    let me = router
        .clone()
        .oneshot(json_request("GET", "/api/auth/me", Some(&token), json!({})))
        .await
        .unwrap();
    assert_eq!(me.status(), StatusCode::OK);

    let listed_tokens = router
        .clone()
        .oneshot(json_request(
            "GET",
            "/api/account/tokens",
            Some(&token),
            json!({}),
        ))
        .await
        .unwrap();
    assert_eq!(listed_tokens.status(), StatusCode::OK);
    let listed_tokens = body_json(listed_tokens.into_body()).await;
    assert_eq!(listed_tokens.as_array().unwrap().len(), 1);
    assert!(listed_tokens[0].get("token").is_none());

    let created = router.clone().oneshot(json_request("POST", "/api/sites", Some(&token), json!({"name":"Example","domain":"example.com","timezone":"UTC","allowed_origins":["https://example.com"],"retention_days":30}))).await.unwrap();
    assert_eq!(created.status(), StatusCode::CREATED);
    let site = body_json(created.into_body()).await;
    let site_id = site["id"].as_str().unwrap();
    let site_uuid = uuid::Uuid::parse_str(site_id).unwrap();
    let write_key = site["writeKey"].as_str().unwrap();
    assert_eq!(site["antiAdblockServer"], "caddy");
    assert!(site["antiAdblockJsPath"]
        .as_str()
        .is_some_and(|path| path.starts_with('/') && path.ends_with(".js") && path.len() == 16));
    assert!(site["antiAdblockBeaconPath"]
        .as_str()
        .is_some_and(|path| path.starts_with('/') && path.len() == 13));

    let ensured = router.clone().oneshot(json_request("POST", "/api/sites/ensure", Some(&token), json!({"name":"Ignored on reuse","domain":"EXAMPLE.COM.","timezone":"America/Denver","allowedOrigins":["https://example.com"],"retentionDays":90}))).await.unwrap();
    assert_eq!(ensured.status(), StatusCode::OK);
    let ensured = body_json(ensured.into_body()).await;
    assert_eq!(ensured["created"], false);
    assert_eq!(ensured["site"]["id"], site_id);
    assert_eq!(ensured["site"]["name"], "Example");

    let updated = router
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("/api/sites/{site_id}/anti-adblock"),
            Some(&token),
            json!({
                "serverType":"nginx",
                "jsPath":"/456bbb63bb86.js",
                "beaconPath":"/0d31360a3101"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(updated.status(), StatusCode::OK);
    let updated = body_json(updated.into_body()).await;
    assert_eq!(updated["antiAdblockServer"], "nginx");
    assert_eq!(updated["antiAdblockJsPath"], "/456bbb63bb86.js");
    assert_eq!(updated["antiAdblockBeaconPath"], "/0d31360a3101");

    let mut proxy_test = Request::get(format!("/api/collect/{write_key}"))
        .body(Body::empty())
        .unwrap();
    proxy_test
        .headers_mut()
        .insert(header::ORIGIN, "https://example.com".parse().unwrap());
    let proxy_test = router.clone().oneshot(proxy_test).await.unwrap();
    assert_eq!(proxy_test.status(), StatusCode::OK);
    assert_eq!(
        proxy_test.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN],
        "https://example.com"
    );
    assert_eq!(body_json(proxy_test.into_body()).await["status"], "ok");

    let preflight = Request::builder()
        .method("OPTIONS")
        .uri(format!("/api/e/{write_key}"))
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
        &format!("/api/e/{write_key}"),
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

    let revoked = router
        .clone()
        .oneshot(json_request(
            "DELETE",
            "/api/account/tokens/current",
            Some(&token),
            json!({}),
        ))
        .await
        .unwrap();
    assert_eq!(revoked.status(), StatusCode::NO_CONTENT);
    let rejected = router
        .clone()
        .oneshot(json_request("GET", "/api/auth/me", Some(&token), json!({})))
        .await
        .unwrap();
    assert_eq!(rejected.status(), StatusCode::UNAUTHORIZED);
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
