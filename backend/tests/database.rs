use sqlx::postgres::PgPoolOptions;

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to a disposable PostgreSQL database"]
async fn migrations_create_core_schema() {
    let url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL required");
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .unwrap();
    sqlx::migrate!("../migrations").run(&pool).await.unwrap();
    let tables: i64 = sqlx::query_scalar("SELECT count(*) FROM information_schema.tables WHERE table_schema='public' AND table_name = ANY($1)")
        .bind(vec!["users", "sites", "events", "goals", "goal_completions", "stream_events"])
        .fetch_one(&pool).await.unwrap();
    assert_eq!(tables, 6);
}
