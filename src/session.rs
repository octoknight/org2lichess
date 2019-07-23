use chrono::Duration;
use rocket::http::{Cookie, Cookies};
use rocket::outcome::IntoOutcome;
use rocket::request::{FromRequest, Outcome};
use rocket::Request;

pub struct Session {
    pub lichess_id: String,
    pub lichess_username: String,
    pub oauth_token: String,
}

const LICHESS_ID_COOKIE: &'static str = "lichess_id";
const LICHESS_USERNAME_COOKIE: &'static str = "lichess_username";
const OAUTH_TOKEN_COOKIE: &'static str = "oauth";

impl<'a, 'r> FromRequest<'a, 'r> for Session {
    type Error = std::convert::Infallible;

    fn from_request(request: &'a Request<'r>) -> Outcome<Session, Self::Error> {
        let mut cookies = request.cookies();
        let maybe_lichess_id: Option<String> = cookies
            .get_private(LICHESS_ID_COOKIE)
            .and_then(|c| c.value().parse().ok());
        let maybe_lichess_username_cookie: Option<String> = cookies
            .get_private(LICHESS_USERNAME_COOKIE)
            .and_then(|c| c.value().parse().ok());
        let maybe_oauth_token_cookie: Option<String> = cookies
            .get_private(OAUTH_TOKEN_COOKIE)
            .and_then(|c| c.value().parse().ok());
        let id_and_name_and_oauth = maybe_lichess_id.and_then(|id| {
            maybe_lichess_username_cookie
                .and_then(|name| maybe_oauth_token_cookie.map(|oa| (id, name, oa)))
        });
        id_and_name_and_oauth
            .map(|t| Session {
                lichess_id: t.0,
                lichess_username: t.1,
                oauth_token: t.2,
            })
            .or_forward(())
    }
}

pub fn set_session(mut cookies: Cookies<'_>, session: Session) {
    let mut id_cookie = Cookie::new(LICHESS_ID_COOKIE, session.lichess_id);
    id_cookie.set_max_age(Duration::minutes(55));
    cookies.add_private(id_cookie);

    let mut username_cookie = Cookie::new(LICHESS_USERNAME_COOKIE, session.lichess_username);
    username_cookie.set_max_age(Duration::minutes(55));
    cookies.add_private(username_cookie);

    let mut oauth_cookie = Cookie::new(OAUTH_TOKEN_COOKIE, session.oauth_token);
    oauth_cookie.set_max_age(Duration::minutes(55));
    cookies.add_private(oauth_cookie);
}

pub fn remove_session(mut cookies: Cookies<'_>) {
    cookies.remove_private(Cookie::named(LICHESS_ID_COOKIE));
    cookies.remove_private(Cookie::named(LICHESS_USERNAME_COOKIE));
    cookies.remove_private(Cookie::named(OAUTH_TOKEN_COOKIE));
}
