use crate::types::*;
use chrono::Duration;
use rocket::http::{Cookie, Cookies, SameSite};
use rocket::outcome::IntoOutcome;
use rocket::request::{FromRequest, Outcome};
use rocket::Request;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Session {
    pub lichess_id: String,
    pub lichess_username: String,
    pub oauth_token: String,
}

const SESSION_COOKIE: &str = "e2lsession";
const OAUTH_STATE_COOKIE: &str = "e2loauthstate";
const OAUTH_VERIFIER_COOKIE: &str = "e2loauthverifier";

impl<'a, 'r> FromRequest<'a, 'r> for Session {
    type Error = std::convert::Infallible;

    fn from_request(request: &'a Request<'r>) -> Outcome<Session, Self::Error> {
        let mut cookies = request.cookies();

        let maybe_session: Option<String> = cookies
            .get_private(SESSION_COOKIE)
            .and_then(|c| c.value().parse().ok());
        maybe_session
            .and_then(|session| serde_json::from_str(&session).ok())
            .or_forward(())
    }
}

pub fn set_session(mut cookies: Cookies<'_>, session: Session) -> Result<(), ErrorBox> {
    let mut session_cookie = Cookie::new(SESSION_COOKIE, serde_json::to_string(&session)?);
    session_cookie.set_max_age(Duration::minutes(55));
    session_cookie.set_same_site(SameSite::Lax);
    session_cookie.set_secure(true);
    cookies.add_private(session_cookie);
    Ok(())
}

pub fn remove_session(mut cookies: Cookies<'_>) {
    cookies.remove_private(Cookie::named(SESSION_COOKIE));
}

pub fn set_oauth_state_cookie(cookies: &mut Cookies<'_>, oauth_state: &str) {
    let mut oauth_state_cookie = Cookie::new(OAUTH_STATE_COOKIE, oauth_state.to_string());
    oauth_state_cookie.set_max_age(Duration::minutes(5));
    cookies.add(oauth_state_cookie);
}

pub fn pop_oauth_state(cookies: &mut Cookies<'_>) -> Option<String> {
    let cookie_value = cookies
        .get(OAUTH_STATE_COOKIE)
        .map(|c| c.value().to_string());
    cookies.remove(Cookie::named(OAUTH_STATE_COOKIE));
    cookie_value
}

pub fn set_oauth_code_verifier(cookies: &mut Cookies<'_>, code_verifier: &str) {
    let mut verifier_cookie = Cookie::new(OAUTH_VERIFIER_COOKIE, code_verifier.to_string());
    verifier_cookie.set_max_age(Duration::minutes(5));
    verifier_cookie.set_same_site(SameSite::Lax);
    verifier_cookie.set_secure(true);
    cookies.add_private(verifier_cookie);
}

pub fn pop_oauth_code_verifier(cookies: &mut Cookies<'_>) -> Option<String> {
    let cookie_value = cookies
        .get_private(OAUTH_VERIFIER_COOKIE)
        .map(|c| c.value().to_string());
    cookies.remove_private(Cookie::named(OAUTH_VERIFIER_COOKIE));
    cookie_value
}
