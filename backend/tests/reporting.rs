use chrono::{NaiveDate, TimeZone, Utc};
use slimlytics_backend::reporting::date_bounds;

#[test]
fn site_date_bounds_follow_daylight_saving_time() {
    let date = NaiveDate::from_ymd_opt(2026, 3, 8).unwrap();
    let (start, end, prior) = date_bounds(date, date, "America/Denver").unwrap();

    assert_eq!(start, Utc.with_ymd_and_hms(2026, 3, 8, 7, 0, 0).unwrap());
    assert_eq!(end, Utc.with_ymd_and_hms(2026, 3, 9, 6, 0, 0).unwrap());
    assert_eq!(prior, Utc.with_ymd_and_hms(2026, 3, 7, 7, 0, 0).unwrap());
}

#[test]
fn rejects_unknown_timezones_and_invalid_ranges() {
    let from = NaiveDate::from_ymd_opt(2026, 1, 2).unwrap();
    let to = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

    assert!(date_bounds(from, to, "UTC").is_err());
    assert!(date_bounds(to, to, "Not/A_Timezone").is_err());
}
