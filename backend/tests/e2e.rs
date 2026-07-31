use axum::{
    body::{to_bytes, Body},
    extract::ConnectInfo,
    http::{header, Request, StatusCode},
};
use serde_json::{json, Value};
use slimlytics_backend::briefs::{build_marketing_brief, process_due_reports};
use slimlytics_backend::maintenance::{prune_expired_events, refresh_daily_rollups};
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
            json!({
                "name":"e2e agent",
                "expiresInDays":30,
                "scopes":[
                    "sites:read","sites:write","analytics:read","analytics:write",
                    "integrations:read","integrations:write"
                ]
            }),
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
    let server_write_key = site["serverWriteKey"].as_str().unwrap();
    uuid::Uuid::parse_str(server_write_key).unwrap();
    let user_uuid: uuid::Uuid = sqlx::query_scalar("SELECT id FROM users WHERE email=$1")
        .bind(&email)
        .fetch_one(&pool)
        .await
        .unwrap();
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

    let mut private_collect = json_request(
        "POST",
        &format!("/api/e/{write_key}"),
        None,
        json!({
            "name":"signup",
            "url":"https://example.com/private",
            "privacyControl":"gpc",
            "trackerVersion":"1.0.0",
            "properties":{"email":"private@example.com","plan":"pro"}
        }),
    );
    private_collect
        .headers_mut()
        .insert(header::ORIGIN, "https://example.com".parse().unwrap());
    private_collect
        .headers_mut()
        .insert(header::USER_AGENT, "Mozilla/5.0".parse().unwrap());
    private_collect.extensions_mut().insert(ConnectInfo(
        "203.0.113.11:443".parse::<std::net::SocketAddr>().unwrap(),
    ));
    let private_collected = router.clone().oneshot(private_collect).await.unwrap();
    assert_eq!(private_collected.status(), StatusCode::ACCEPTED);
    let private_event: (serde_json::Value, String, Option<String>) = sqlx::query_as(
        "SELECT properties,privacy_mode,tracker_version FROM events WHERE site_id=$1 AND path='/private'",
    )
    .bind(site_uuid)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(private_event.0, json!({}));
    assert_eq!(private_event.1, "gpc");
    assert_eq!(private_event.2.as_deref(), Some("1.0.0"));

    let health = router
        .clone()
        .oneshot(json_request(
            "GET",
            &format!("/api/sites/{site_id}/collection-health"),
            Some(&token),
            json!({}),
        ))
        .await
        .unwrap();
    assert_eq!(health.status(), StatusCode::OK);
    let health = body_json(health.into_body()).await;
    assert_eq!(health["acceptedTotal"], 2);
    assert_eq!(health["rejectedTotal"], 0);
    assert_eq!(health["lastTrackerVersion"], "1.0.0");

    let today = chrono::Utc::now().date_naive();
    let overview_response = router
        .clone()
        .oneshot(json_request(
            "GET",
            &format!("/api/sites/{site_id}/overview?from={today}&to={today}"),
            Some(&token),
            json!({}),
        ))
        .await
        .unwrap();
    assert_eq!(overview_response.status(), StatusCode::OK);
    let overview_body = body_json(overview_response.into_body()).await;
    assert_eq!(overview_body["views"]["current"], 0);
    assert_eq!(overview_body["events"]["current"], 2);
    assert!(overview_body["bounceRate"].is_number());
    assert!(overview_body["avgDurationSeconds"].is_number());
    assert_eq!(overview_body["trend"].as_array().unwrap().len(), 1);

    let goals_response = router
        .clone()
        .oneshot(json_request(
            "GET",
            &format!("/api/sites/{site_id}/goals"),
            Some(&token),
            json!({}),
        ))
        .await
        .unwrap();
    let goals_body = body_json(goals_response.into_body()).await;
    assert_eq!(goals_body[0]["conversions"], 1);
    assert!(goals_body[0]["conversionRate"].is_number());
    let completions: i64 =
        sqlx::query_scalar("SELECT count(*) FROM goal_completions WHERE site_id=$1")
            .bind(site_uuid)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(completions, 1);

    let annotation_path = format!("/api/sites/{site_id}/annotations");
    let mut first_annotation = json_request(
        "POST",
        &annotation_path,
        Some(&token),
        json!({"occurredOn":today,"label":"Campaign launched"}),
    );
    first_annotation
        .headers_mut()
        .insert("idempotency-key", "e2e-campaign-launch".parse().unwrap());
    let first_annotation = router.clone().oneshot(first_annotation).await.unwrap();
    assert_eq!(first_annotation.status(), StatusCode::CREATED);
    let first_annotation = body_json(first_annotation.into_body()).await;
    let mut repeated_annotation = json_request(
        "POST",
        &annotation_path,
        Some(&token),
        json!({"occurredOn":today,"label":"Campaign launched"}),
    );
    repeated_annotation
        .headers_mut()
        .insert("idempotency-key", "e2e-campaign-launch".parse().unwrap());
    let repeated_annotation = router.clone().oneshot(repeated_annotation).await.unwrap();
    assert_eq!(repeated_annotation.status(), StatusCode::CREATED);
    assert_eq!(
        body_json(repeated_annotation.into_body()).await["id"],
        first_annotation["id"]
    );

    let funnel_path = format!("/api/sites/{site_id}/funnels");
    let funnel_input = json!({
        "name":"Signup funnel",
        "steps":[
            {"label":"Visit","path":"/thanks"},
            {"label":"Signup","eventName":"signup"}
        ]
    });
    let mut first_funnel = json_request("POST", &funnel_path, Some(&token), funnel_input.clone());
    first_funnel
        .headers_mut()
        .insert("idempotency-key", "e2e-signup-funnel".parse().unwrap());
    let first_funnel = router.clone().oneshot(first_funnel).await.unwrap();
    assert_eq!(first_funnel.status(), StatusCode::CREATED);
    let first_funnel = body_json(first_funnel.into_body()).await;
    let mut repeated_funnel = json_request("POST", &funnel_path, Some(&token), funnel_input);
    repeated_funnel
        .headers_mut()
        .insert("idempotency-key", "e2e-signup-funnel".parse().unwrap());
    let repeated_funnel = router.clone().oneshot(repeated_funnel).await.unwrap();
    assert_eq!(repeated_funnel.status(), StatusCode::CREATED);
    assert_eq!(
        body_json(repeated_funnel.into_body()).await["id"],
        first_funnel["id"]
    );
    let funnel_report = router
        .clone()
        .oneshot(json_request(
            "GET",
            &format!(
                "/api/sites/{site_id}/funnels/{}/report?from={today}&to={today}",
                first_funnel["id"].as_str().unwrap()
            ),
            Some(&token),
            json!({}),
        ))
        .await
        .unwrap();
    assert_eq!(funnel_report.status(), StatusCode::OK);
    assert_eq!(
        body_json(funnel_report.into_body()).await["steps"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    sqlx::query("INSERT INTO search_console_integrations(site_id,refresh_token_encrypted,connected_by) VALUES($1,'encrypted',$2)")
        .bind(site_uuid)
        .bind(user_uuid)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO search_console_metrics(site_id,metric_date,clicks,impressions,ctr,position) VALUES($1,$2,1,2,0.5,3)")
        .bind(site_uuid)
        .bind(today)
        .execute(&pool)
        .await
        .unwrap();
    let disconnected = router
        .clone()
        .oneshot(json_request(
            "DELETE",
            &format!("/api/sites/{site_id}/integrations/search-console"),
            Some(&token),
            json!({}),
        ))
        .await
        .unwrap();
    assert_eq!(disconnected.status(), StatusCode::NO_CONTENT);
    let search_console_rows: i64 = sqlx::query_scalar(
        "SELECT
           (SELECT count(*) FROM search_console_integrations WHERE site_id=$1) +
           (SELECT count(*) FROM search_console_metrics WHERE site_id=$1)",
    )
    .bind(site_uuid)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(search_console_rows, 0);

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

    let yesterday = today.pred_opt().unwrap();
    sqlx::query("INSERT INTO events(site_id,occurred_at,visitor_id,session_id,event_name,url,path) VALUES($1,$2,'brief-visitor','brief-session','pageview','https://example.com/brief','/brief')")
        .bind(site_uuid)
        .bind(yesterday.and_hms_opt(12, 0, 0).unwrap().and_utc())
        .execute(&pool)
        .await
        .unwrap();
    assert!(refresh_daily_rollups(&pool, 8).await.unwrap() >= 1);
    let brief = build_marketing_brief(&pool, site_uuid, 1).await.unwrap();
    assert_eq!(brief["dataThrough"], yesterday.to_string());
    assert_eq!(brief["metrics"]["pageViews"]["value"], 1);
    assert_eq!(brief["topPages"][0]["path"], "/brief");
    let mcp_brief = router
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/mcp",
            Some(&token),
            json!({
                "jsonrpc":"2.0","id":1,"method":"tools/call",
                "params":{"name":"marketing_brief","arguments":{"siteId":site_id,"days":1}}
            }),
        ))
        .await
        .unwrap();
    assert_eq!(mcp_brief.status(), StatusCode::OK);
    let mcp_brief = body_json(mcp_brief.into_body()).await;
    assert_eq!(
        mcp_brief["result"]["structuredContent"]["metrics"]["pageViews"]["value"],
        1
    );

    let subscription = router
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/sites/{site_id}/report-subscriptions"),
            Some(&token),
            json!({
                "name":"Weekly agent brief",
                "webhookUrl":"https://hooks.example.com/slimlytics",
                "frequency":"weekly",
                "anomalyOnly":false
            }),
        ))
        .await
        .unwrap();
    assert_eq!(subscription.status(), StatusCode::CREATED);
    let subscription = body_json(subscription.into_body()).await;
    assert!(subscription["signingSecret"].as_str().unwrap().len() >= 40);
    let subscription_id = uuid::Uuid::parse_str(subscription["id"].as_str().unwrap()).unwrap();
    sqlx::query(
        "UPDATE report_subscriptions SET anomaly_only=true,next_run_at=now()-interval '1 minute'
         WHERE id=$1",
    )
    .bind(subscription_id)
    .execute(&pool)
    .await
    .unwrap();
    assert_eq!(
        process_due_reports(&pool, b"abcdefghijklmnopqrstuvwxyz012345")
            .await
            .unwrap(),
        1
    );
    let scheduled: (Option<String>, bool) = sqlx::query_as(
        "SELECT last_status,next_run_at>now()+interval '6 days'
         FROM report_subscriptions WHERE id=$1",
    )
    .bind(subscription_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(scheduled.0.as_deref(), Some("skipped"));
    assert!(scheduled.1);

    let server_payload = json!({"events":[{
        "idempotencyKey":"e2e-server-request-1",
        "url":"https://example.com/robots-target?token=private&utm_source=crawler",
        "userAgent":"GPTBot/1.0",
        "clientIp":"203.0.113.40",
        "method":"GET","status":200
    }]});
    let mut server_request = json_request("POST", "/api/ingest", None, server_payload.clone());
    server_request
        .headers_mut()
        .insert("x-slimlytics-server-key", server_write_key.parse().unwrap());
    server_request.extensions_mut().insert(ConnectInfo(
        "198.51.100.10:443".parse::<std::net::SocketAddr>().unwrap(),
    ));
    let server_response = router.clone().oneshot(server_request).await.unwrap();
    assert_eq!(server_response.status(), StatusCode::ACCEPTED);
    assert_eq!(body_json(server_response.into_body()).await["accepted"], 1);
    let mut retry = json_request("POST", "/api/ingest", None, server_payload);
    retry
        .headers_mut()
        .insert("x-slimlytics-server-key", server_write_key.parse().unwrap());
    retry.extensions_mut().insert(ConnectInfo(
        "198.51.100.10:443".parse::<std::net::SocketAddr>().unwrap(),
    ));
    let retry = router.clone().oneshot(retry).await.unwrap();
    assert_eq!(body_json(retry.into_body()).await["duplicates"], 1);
    let server_event: (String, String, String, Option<String>) = sqlx::query_as(
        "SELECT ingestion_source,traffic_class::text,url,automation_name FROM events
         WHERE site_id=$1 AND source_event_id='e2e-server-request-1'",
    )
    .bind(site_uuid)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(server_event.0, "server");
    assert_eq!(server_event.1, "bot");
    assert_eq!(server_event.2, "https://example.com/robots-target");
    assert_eq!(server_event.3.as_deref(), Some("GPTBot"));
    let crawler_report = router
        .clone()
        .oneshot(json_request(
            "GET",
            &format!("/api/sites/{site_id}/reports/ai-crawlers?from={today}&to={today}"),
            Some(&token),
            json!({}),
        ))
        .await
        .unwrap();
    assert_eq!(crawler_report.status(), StatusCode::OK);
    assert_eq!(
        body_json(crawler_report.into_body()).await[0]["value"],
        "GPTBot"
    );

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
