use crate::config::Config;
use crate::db::Membership;
use crate::session::Session;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct LoggedInContext {
    pub lichess: String,
    pub admin: bool,
}

#[derive(Serialize)]
pub struct LoggedInWithErrorContext {
    #[serde(flatten)]
    pub logged_in: LoggedInContext,
    pub error: String,
}

#[derive(Serialize)]
pub struct LinkedContext {
    #[serde(flatten)]
    pub logged_in: LoggedInContext,
    pub ecf: i32,
    pub exp: i32,
    pub can_renew: bool,
}

#[derive(Serialize)]
pub struct AdminContext {
    #[serde(flatten)]
    pub logged_in: LoggedInContext,
    pub members: Vec<Membership>,
}

pub fn empty_context() -> HashMap<u8, u8> {
    HashMap::new()
}

pub fn make_logged_in_context(session: &Session, config: &Config) -> LoggedInContext {
    LoggedInContext {
        lichess: String::from(&session.lichess_username),
        admin: &session.lichess_id == &config.lichess_admin_id,
    }
}

pub fn make_error_context(logged_in: LoggedInContext, error: &str) -> LoggedInWithErrorContext {
    LoggedInWithErrorContext {
        logged_in,
        error: error.to_string(),
    }
}

pub fn make_admin_context(
    logged_in: LoggedInContext,
    members: Vec<Membership>,
) -> AdminContext {
    AdminContext { logged_in, members }
}

pub fn make_linked_context(
    logged_in: LoggedInContext,
    ecf_id: i32,
    exp_year: i32,
    can_renew: bool,
) -> LinkedContext {
    LinkedContext {
        logged_in,
        ecf: ecf_id,
        exp: exp_year,
        can_renew,
    }
}
