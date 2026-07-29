use chrono::{TimeZone, Utc};
use slimlytics_backend::identity::derive_ids;

#[test]
fn ids_are_stable_within_day_and_session_bucket_but_rotate() {
    let secret = b"01234567890123456789012345678901";
    let t = Utc.with_ymd_and_hms(2026, 7, 28, 10, 1, 0).unwrap();
    let same = Utc.with_ymd_and_hms(2026, 7, 28, 10, 20, 0).unwrap();
    let later = Utc.with_ymd_and_hms(2026, 7, 28, 11, 1, 0).unwrap();
    let first = derive_ids(secret, "site", "203.0.113.7", "ua", t);
    let second = derive_ids(secret, "site", "203.0.113.7", "ua", same);
    let third = derive_ids(secret, "site", "203.0.113.7", "ua", later);
    assert_eq!(first, second);
    assert_eq!(first.visitor_id, third.visitor_id);
    assert_ne!(first.session_id, third.session_id);
}
