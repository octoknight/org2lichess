use chrono::Duration;
use rocket::http::{Cookie, Cookies};
use rocket::outcome::IntoOutcome;
use rocket::request::{FromRequest, Outcome};
use rocket::Request;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize)]
pub struct Session {
    pub lichess_id: String,
    pub lichess_username: String,
    pub oauth_token: String,
}

const SESSION_COOKIE: &'static str = "e2lsession";

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

pub fn set_session(
    mut cookies: Cookies<'_>,
    session: Session,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut session_cookie = Cookie::new(SESSION_COOKIE, serde_json::to_string(&session)?);
    session_cookie.set_max_age(Duration::minutes(55));
    cookies.add_private(session_cookie);
    Ok(())
}

pub fn remove_session(mut cookies: Cookies<'_>) {
    cookies.remove_private(Cookie::named(SESSION_COOKIE));
}
