use sqlx::PgPool;

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
