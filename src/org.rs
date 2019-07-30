use crate::types::*;
use chrono::{Datelike, TimeZone, Utc};
use chrono_tz::Tz;

pub fn timezone_from_string(timezone: &str) -> Result<Tz, ErrorBox> {
    Ok(timezone.parse()?)
}

pub fn is_past_expiry(year: i32, timezone: Tz, month: u32, day: u32) -> bool {
    let expiry = timezone.ymd(year, month, day).and_hms(23, 59, 59);
    let now = timezone.from_utc_datetime(&Utc::now().naive_utc());
    now > expiry
}

pub fn current_year(timezone: Tz) -> i32 {
    timezone.from_utc_datetime(&Utc::now().naive_utc()).year()
}

pub fn is_past_expiry_this_year(timezone: Tz, month: u32, day: u32) -> bool {
    is_past_expiry(current_year(timezone), timezone, month, day)
}

pub fn is_past_renewal(exp_year: i32, timezone: Tz, month: u32, day: u32) -> bool {
    let renewal_deadline = timezone.ymd(exp_year, month, day).and_hms(23, 59, 59);
    let now = timezone.from_utc_datetime(&Utc::now().naive_utc());
    now > renewal_deadline
}
