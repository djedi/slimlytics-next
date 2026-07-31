use sqlx::PgPool;

/// Rebuild completed local calendar days. The current day remains raw so dashboards include
/// newly collected events immediately without maintaining approximate distinct counters.
pub async fn refresh_daily_rollups(pool: &PgPool, days: i64) -> Result<u64, sqlx::Error> {
    let days = days.clamp(2, 3660) as i32;
    let mut tx = pool.begin().await?;
    sqlx::query(
        "DELETE FROM daily_site_rollups r USING sites s
         WHERE r.site_id=s.id
           AND r.metric_date < (now() AT TIME ZONE s.timezone)::date - s.retention_days",
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "DELETE FROM daily_site_rollups r USING sites s
         WHERE r.site_id=s.id
           AND r.metric_date >= (now() AT TIME ZONE s.timezone)::date - $1",
    )
    .bind(days)
    .execute(&mut *tx)
    .await?;
    let inserted = sqlx::query(
        "INSERT INTO daily_site_rollups(
           site_id,metric_date,page_views,visitors,sessions,custom_events,bot_requests,
           ai_crawler_requests,revenue,refreshed_at
         )
         SELECT s.id,(e.occurred_at AT TIME ZONE s.timezone)::date,
           count(*) FILTER(WHERE e.traffic_class='human' AND e.event_name='pageview'),
           count(DISTINCT e.visitor_id) FILTER(WHERE e.traffic_class='human'),
           count(DISTINCT e.session_id) FILTER(WHERE e.traffic_class='human'),
           count(*) FILTER(WHERE e.traffic_class='human' AND e.event_name<>'pageview'),
           count(*) FILTER(WHERE e.traffic_class='bot'),
           count(*) FILTER(WHERE e.automation_category='ai-crawler'),
           COALESCE(sum(e.revenue_amount) FILTER(WHERE e.traffic_class='human'),0),now()
         FROM sites s JOIN events e ON e.site_id=s.id
         WHERE (e.occurred_at AT TIME ZONE s.timezone)::date >=
                 (now() AT TIME ZONE s.timezone)::date - $1
           AND (e.occurred_at AT TIME ZONE s.timezone)::date <
                 (now() AT TIME ZONE s.timezone)::date
         GROUP BY s.id,(e.occurred_at AT TIME ZONE s.timezone)::date",
    )
    .bind(days)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    tx.commit().await?;
    Ok(inserted)
}

/// Delete at most `batch_size` raw events that have exceeded their site's retention period.
/// Related stream and goal-completion rows are removed by foreign-key cascades.
pub async fn prune_expired_events(pool: &PgPool, batch_size: i64) -> Result<u64, sqlx::Error> {
    let limit = batch_size.clamp(1, 10_000);
    let result = sqlx::query(
        "WITH expired AS (\
             SELECT e.id \
             FROM events e \
             JOIN sites s ON s.id = e.site_id \
             WHERE e.occurred_at < now() - make_interval(days => s.retention_days) \
             ORDER BY e.occurred_at \
             LIMIT $1 \
             FOR UPDATE OF e SKIP LOCKED\
         ) \
         DELETE FROM events e \
         USING expired \
         WHERE e.id = expired.id",
    )
    .bind(limit)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}
