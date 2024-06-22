use chrono::{DateTime, Utc};

pub fn unix_to_datetime(timestamp: i64) -> DateTime<Utc> {
    DateTime::from_timestamp(timestamp, 0).expect("Valid timestamp expected")
}

pub fn diff_for_humans(timestamp1: i64, timestamp2: i64) -> String {
    let dt1 = unix_to_datetime(timestamp1);
    let dt2 = unix_to_datetime(timestamp2);

    let diff = dt2.signed_duration_since(dt1);

    let (unit, value) = match diff.num_seconds().abs() {
        0..=59 => ("second", diff.num_seconds().abs()),
        60..=3599 => ("minute", diff.num_minutes().abs()),
        3600..=86399 => ("hour", diff.num_hours().abs()),
        86400..=2591999 => ("day", diff.num_days().abs()),
        2592000..=31535999 => ("month", (diff.num_days().abs() / 30)),
        _ => ("year", (diff.num_days().abs() / 365)),
    };

    let plural = if value == 1 { "" } else { "s" };
    let hours = diff.num_hours();
    let minutes = diff.num_minutes() % 60;
    let seconds = diff.num_seconds() % 60;

    format!(
        "{} {}{} ago ({:02}:{:02}:{:02})",
        value, unit, plural, hours, minutes, seconds
    )
}
