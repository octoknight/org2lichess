use chrono::{Datelike, TimeZone, Utc};
use chrono_tz::Europe::London;

pub fn is_past_expiry(year: i32) -> bool {
    let expiry = London.ymd(year, 8, 31).and_hms(23, 59, 59);
    let now = London.from_utc_datetime(&Utc::now().naive_utc());
    now > expiry
}

pub fn current_london_year() -> i32 {
    London.from_utc_datetime(&Utc::now().naive_utc()).year()
}

pub fn is_past_expiry_this_year() -> bool {
    is_past_expiry(current_london_year())
}

pub fn is_past_renewal(exp_year: i32) -> bool {
    let renewal_deadline = London.ymd(exp_year, 9, 14).and_hms(23, 59, 59);
    let now = London.from_utc_datetime(&Utc::now().naive_utc());
    now > renewal_deadline
}
