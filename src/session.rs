use rocket::http::{Cookie, Cookies};
use rocket::outcome::IntoOutcome;
use rocket::request::{FromRequest, Outcome};
use rocket::Request;

pub struct Session {
    pub lichess_id: String,
    pub lichess_username: String,
}

const LICHESS_ID_COOKIE: &'static str = "lichess_id";
const LICHESS_USERNAME_COOKIE: &'static str = "lichess_username";

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
        let id_and_name =
            maybe_lichess_id.and_then(|id| maybe_lichess_username_cookie.map(|name| (id, name)));
        id_and_name
            .map(|t| Session {
                lichess_id: t.0,
                lichess_username: t.1,
            })
            .or_forward(())
    }
}

pub fn set_session(mut cookies: Cookies<'_>, session: Session) {
    cookies.add_private(Cookie::new(LICHESS_ID_COOKIE, session.lichess_id));
    cookies.add_private(Cookie::new(
        LICHESS_USERNAME_COOKIE,
        session.lichess_username,
    ));
}
