use crate::{reporting::date_bounds, webhooks::send_signed_json};
use chrono::{Days, NaiveDate, Utc};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn build_marketing_brief(
    pool: &PgPool,
    site: Uuid,
    days: i64,
) -> Result<Value, sqlx::Error> {
    let days = days.clamp(1, 90) as u64;
    let (name, domain, timezone, today): (String, String, String, NaiveDate) = sqlx::query_as(
        "SELECT name,domain,timezone,(now() AT TIME ZONE timezone)::date FROM sites WHERE id=$1",
    )
    .bind(site)
    .fetch_one(pool)
    .await?;
    let to = today.checked_sub_days(Days::new(1)).unwrap_or(today);
    let from = to
        .checked_sub_days(Days::new(days.saturating_sub(1)))
        .unwrap_or(to);
    let previous_to = from.checked_sub_days(Days::new(1)).unwrap_or(from);
    let previous_from = previous_to
        .checked_sub_days(Days::new(days.saturating_sub(1)))
        .unwrap_or(previous_to);
    let totals = |start: NaiveDate, end: NaiveDate| async move {
        sqlx::query_as::<_, (i64, i64, i64, i64, i64, i64, String)>(
            "SELECT COALESCE(sum(page_views),0)::bigint,
               COALESCE(sum(visitors),0)::bigint,COALESCE(sum(sessions),0)::bigint,
               COALESCE(sum(custom_events),0)::bigint,COALESCE(sum(bot_requests),0)::bigint,
               COALESCE(sum(ai_crawler_requests),0)::bigint,
               COALESCE(sum(revenue),0)::text
             FROM daily_site_rollups WHERE site_id=$1 AND metric_date BETWEEN $2 AND $3",
        )
        .bind(site)
        .bind(start)
        .bind(end)
        .fetch_one(pool)
        .await
    };
    let current = totals(from, to).await?;
    let previous = totals(previous_from, previous_to).await?;
    let (start, end, _) = date_bounds(from, to, &timezone)
        .map_err(|message| sqlx::Error::Protocol(message.into()))?;
    let top_pages: Vec<(String, i64, i64)> = sqlx::query_as(
        "SELECT path,count(*)::bigint,count(DISTINCT visitor_id)::bigint FROM events
         WHERE site_id=$1 AND occurred_at >= $2 AND occurred_at < $3
           AND traffic_class='human' AND event_name='pageview'
         GROUP BY path ORDER BY count(*) DESC LIMIT 10",
    )
    .bind(site)
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await?;
    let anomalies: Vec<(NaiveDate, i64, f64)> = sqlx::query_as(
        "WITH daily AS (
           SELECT metric_date,page_views,
             avg(page_views) OVER(ORDER BY metric_date ROWS BETWEEN 7 PRECEDING AND 1 PRECEDING) baseline
           FROM daily_site_rollups WHERE site_id=$1 AND metric_date <= $3
         )
         SELECT metric_date,page_views,
           CASE WHEN baseline>0 THEN 100.0*(page_views-baseline)/baseline ELSE 0 END::float8
         FROM daily WHERE metric_date BETWEEN $2 AND $3 AND baseline>0
           AND abs(page_views-baseline)/baseline >= 0.30
         ORDER BY metric_date",
    )
    .bind(site)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await?;
    let change = |now: i64, before: i64| {
        (before != 0).then(|| (now - before) as f64 * 100.0 / before as f64)
    };
    Ok(json!({
        "schemaVersion":"2026-07-31",
        "type":"slimlytics.marketing-brief",
        "site":{"id":site,"name":name,"domain":domain,"timezone":timezone},
        "period":{"from":from,"to":to,"days":days},
        "generatedAt":Utc::now(),"dataThrough":to,
        "metrics":{
            "pageViews":{"value":current.0,"previous":previous.0,"changePercent":change(current.0,previous.0)},
            "dailyUniqueVisitors":{"value":current.1,"previous":previous.1,"definition":"Sum of privacy-rotating daily unique visitors"},
            "dailyUniqueSessions":{"value":current.2,"previous":previous.2,"definition":"Sum of 30-minute sessions per local day"},
            "customEvents":current.3,"botRequests":current.4,"aiCrawlerRequests":current.5,
            "revenue":{"value":current.6,"currency":"mixed-or-unspecified"}
        },
        "topPages":top_pages.into_iter().map(|row|json!({"path":row.0,"views":row.1,"visitors":row.2})).collect::<Vec<_>>(),
        "anomalies":anomalies.into_iter().map(|row|json!({"date":row.0,"pageViews":row.1,"deviationPercent":row.2})).collect::<Vec<_>>(),
        "evidence":{"rollupTable":"daily_site_rollups","rawEventWindow":{"from":start,"toExclusive":end}}
    }))
}

pub async fn deliver_report(
    pool: &PgPool,
    identity_secret: &[u8],
    subscription: Uuid,
) -> Result<&'static str, sqlx::Error> {
    let row: Option<(Uuid, String, String, bool)> = sqlx::query_as(
        "SELECT site_id,webhook_url,frequency,anomaly_only FROM report_subscriptions
         WHERE id=$1 AND enabled",
    )
    .bind(subscription)
    .fetch_optional(pool)
    .await?;
    let Some((site, destination, frequency, anomaly_only)) = row else {
        return Ok("skipped");
    };
    let payload =
        build_marketing_brief(pool, site, if frequency == "daily" { 1 } else { 7 }).await?;
    let should_send = !anomaly_only
        || payload["anomalies"]
            .as_array()
            .is_some_and(|items| !items.is_empty());
    let (status, response_status, error) = if should_send {
        let secret = crate::webhooks::signing_secret(identity_secret, subscription);
        match send_signed_json(&destination, &payload, &secret).await {
            Ok(response) => ("success", Some(response.as_u16() as i32), None),
            Err(error) => ("error", None, Some(error)),
        }
    } else {
        ("skipped", None, None)
    };
    let mut tx = pool.begin().await?;
    sqlx::query(
        "INSERT INTO report_deliveries(subscription_id,site_id,payload,status,response_status,error)
         VALUES($1,$2,$3,$4,$5,$6)",
    )
    .bind(subscription)
    .bind(site)
    .bind(&payload)
    .bind(status)
    .bind(response_status)
    .bind(error.as_deref())
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE report_subscriptions SET last_sent_at=CASE WHEN $2='success' THEN now() ELSE last_sent_at END,
           last_status=$2,last_error=$3,updated_at=now() WHERE id=$1",
    )
    .bind(subscription)
    .bind(status)
    .bind(error.as_deref())
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(status)
}

pub async fn process_due_reports(
    pool: &PgPool,
    identity_secret: &[u8],
) -> Result<u64, sqlx::Error> {
    let candidates: Vec<Uuid> = sqlx::query_scalar(
        "SELECT id FROM report_subscriptions WHERE enabled AND next_run_at<=now()
         ORDER BY next_run_at LIMIT 20",
    )
    .fetch_all(pool)
    .await?;
    let mut claimed = 0;
    for id in candidates {
        let won: Option<Uuid> = sqlx::query_scalar(
            "UPDATE report_subscriptions SET next_run_at=now()+interval '15 minutes',
               updated_at=now()
             WHERE id=$1 AND enabled AND next_run_at<=now() RETURNING id",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;
        if won.is_some() {
            claimed += 1;
            let status = deliver_report(pool, identity_secret, id).await?;
            if matches!(status, "success" | "skipped") {
                sqlx::query(
                    "UPDATE report_subscriptions SET next_run_at=now()+
                       CASE frequency WHEN 'daily' THEN interval '1 day' ELSE interval '7 days' END,
                       updated_at=now() WHERE id=$1",
                )
                .bind(id)
                .execute(pool)
                .await?;
            }
        }
    }
    Ok(claimed)
}
