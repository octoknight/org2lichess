use crate::config::{Config, ExpiryConfig, OrgConfig};
use crate::db::Membership;
use crate::session::Session;
use serde::Serialize;

#[derive(Serialize)]
pub struct BaseContext<'a> {
    pub org: &'a OrgConfig,
}

#[derive(Serialize)]
pub struct LoggedInContext<'a> {
    pub org: &'a OrgConfig,
    pub lichess: String,
    pub admin: bool,
}

#[derive(Serialize)]
pub struct LoggedInWithErrorContext<'a> {
    #[serde(flatten)]
    pub logged_in: LoggedInContext<'a>,
    pub error: String,
}

#[derive(Serialize)]
pub struct LinkedContext<'a> {
    #[serde(flatten)]
    pub logged_in: LoggedInContext<'a>,
    pub org_id: String,
    pub exp_year: i32,
    pub can_renew: bool,
    pub exp_month: String,
    pub exp_day: u32,
    pub renew_month: String,
    pub renew_day: u32,
}

#[derive(Serialize)]
pub struct AdminContext<'a> {
    #[serde(flatten)]
    pub logged_in: LoggedInContext<'a>,
    pub ref_count: i64,
    pub members: Vec<Membership>,
}

pub fn empty_context(config: &Config) -> BaseContext {
    BaseContext { org: &config.org }
}

pub fn make_logged_in_context<'a>(session: &Session, config: &'a Config) -> LoggedInContext<'a> {
    LoggedInContext {
        org: &config.org,
        lichess: String::from(&session.lichess_username),
        admin: &session.lichess_id == &config.lichess.team_admin,
    }
}

pub fn make_error_context<'a>(
    logged_in: LoggedInContext<'a>,
    error: &str,
) -> LoggedInWithErrorContext<'a> {
    LoggedInWithErrorContext {
        logged_in,
        error: error.to_string(),
    }
}

pub fn make_admin_context<'a>(
    logged_in: LoggedInContext<'a>,
    ref_count: i64,
    members: Vec<Membership>,
) -> AdminContext<'a> {
    AdminContext { logged_in, members, ref_count }
}

fn month_to_string(month: u32) -> String {
    String::from(match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "ERROR",
    })
}

pub fn make_linked_context<'a>(
    logged_in: LoggedInContext<'a>,
    org_id: String,
    exp_year: i32,
    can_renew: bool,
    exp_config: &ExpiryConfig,
) -> LinkedContext<'a> {
    LinkedContext {
        logged_in,
        org_id: org_id,
        exp_year: exp_year,
        can_renew,
        exp_month: month_to_string(exp_config.membership_month),
        exp_day: exp_config.membership_day,
        renew_month: month_to_string(exp_config.renewal_month),
        renew_day: exp_config.renewal_day,
    }
}
