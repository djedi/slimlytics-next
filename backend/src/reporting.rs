use chrono::{DateTime, Days, NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;

pub type DateBounds = (DateTime<Utc>, DateTime<Utc>, DateTime<Utc>);

pub fn date_bounds(
    from: NaiveDate,
    to: NaiveDate,
    timezone: &str,
) -> Result<DateBounds, &'static str> {
    if to < from || (to - from).num_days() > 366 {
        return Err("invalid date range");
    }
    let timezone: Tz = timezone.parse().map_err(|_| "invalid site timezone")?;
    let day_count = (to - from).num_days() as u64 + 1;
    let end_date = to
        .checked_add_days(Days::new(1))
        .ok_or("invalid date range")?;
    let prior_date = from
        .checked_sub_days(Days::new(day_count))
        .ok_or("invalid date range")?;
    let at_midnight = |date: NaiveDate| {
        timezone
            .from_local_datetime(&date.and_hms_opt(0, 0, 0).expect("valid midnight"))
            .earliest()
            .map(|value| value.with_timezone(&Utc))
            .ok_or("invalid local date")
    };
    Ok((
        at_midnight(from)?,
        at_midnight(end_date)?,
        at_midnight(prior_date)?,
    ))
}
