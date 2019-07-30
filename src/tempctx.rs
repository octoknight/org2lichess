use crate::config::{Config, OrgConfig};
use crate::db::Membership;
use crate::session::Session;
use serde::Serialize;

#[derive(Serialize)]
pub struct BaseContext<'a> {
    pub org: &'a OrgConfig
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
    pub ecf: String,
    pub exp: i32,
    pub can_renew: bool,
}

#[derive(Serialize)]
pub struct AdminContext<'a> {
    #[serde(flatten)]
    pub logged_in: LoggedInContext<'a>,
    pub members: Vec<Membership>,
}

pub fn empty_context(config: &Config) -> BaseContext {
    BaseContext {
        org: &config.org, 
    }
}

pub fn make_logged_in_context<'a>(session: &Session, config: &'a Config) -> LoggedInContext<'a> {
    LoggedInContext {
        org: &config.org,
        lichess: String::from(&session.lichess_username),
        admin: &session.lichess_id == &config.lichess.team_admin,
    }
}

pub fn make_error_context<'a>(logged_in: LoggedInContext<'a>, error: &str) -> LoggedInWithErrorContext<'a> {
    LoggedInWithErrorContext {
        logged_in,
        error: error.to_string(),
    }
}

pub fn make_admin_context<'a>(logged_in: LoggedInContext<'a>, members: Vec<Membership>) -> AdminContext<'a> {
    AdminContext { logged_in, members }
}

pub fn make_linked_context<'a>(
    logged_in: LoggedInContext<'a>,
    ecf_id: String,
    exp_year: i32,
    can_renew: bool,
) -> LinkedContext<'a> {
    LinkedContext {
        logged_in,
        ecf: ecf_id,
        exp: exp_year,
        can_renew,
    }
}
