#![feature(proc_macro_hygiene, decl_macro, custom_attribute)]

#[macro_use]
extern crate rocket;
extern crate chrono;
extern crate postgres;
extern crate rand;
extern crate reqwest;
extern crate rocket_contrib;
extern crate serde;
#[macro_use]
extern crate serde_json;
extern crate toml;

use rocket::http::{Cookies, Status};
use rocket::request::Form;
use rocket::response::Redirect;
use rocket::State;
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::fs;

mod azolve;
mod config;
mod db;
mod expwatch;
mod lichess;
mod org;
mod randstr;
mod session;
mod tempctx;
mod textlog;
mod types;

use config::Config;
use db::OrgDbClient;
use randstr::random_oauth_state;
use session::Session;
use tempctx::*;
use types::*;

#[get("/", rank = 2)]
fn index(config: State<Config>) -> Template {
    Template::render("index", &empty_context(&config))
}

#[get("/auth")]
fn auth(config: State<Config>, cookies: Cookies<'_>) -> Result<Redirect, ErrorBox> {
    let oauth_state = random_oauth_state()?;
    session::set_oauth_state_cookie(cookies, &oauth_state);

    let url = format!("https://oauth.{}/oauth/authorize?response_type=code&client_id={}&redirect_uri={}/oauth_redirect&scope=team:write&state={}",
        config.lichess.domain, config.lichess.client_id, config.server.url, oauth_state);

    Ok(Redirect::to(url))
}

#[get("/oauth_redirect?<code>&<state>")]
fn oauth_redirect(
    mut cookies: Cookies<'_>,
    code: String,
    state: String,
    config: State<Config>,
    http_client: State<reqwest::Client>,
) -> Result<Result<Template, Status>, ErrorBox> {
    match session::pop_oauth_state(&mut cookies).map(|v| v == state) {
        Some(true) => {
            let token = lichess::oauth_token_from_code(
                &code,
                &http_client,
                &config.lichess.domain,
                &config.lichess.client_id,
                &config.lichess.client_secret,
                &format!("{}/oauth_redirect", config.server.url),
            )
            .unwrap();
            let user = lichess::get_user(&token, &http_client, &config.lichess.domain).unwrap();
            session::set_session(
                cookies,
                Session {
                    lichess_id: user.id,
                    lichess_username: user.username,
                    oauth_token: token.access_token,
                },
            )?;
            Ok(Ok(Template::render("redirect", &empty_context(&config))))
        }
        _ => Ok(Err(Status::BadRequest)),
    }
}

#[get("/")]
fn manage_authed(
    session: Session,
    config: State<Config>,
    db: State<OrgDbClient>,
) -> Result<Template, ErrorBox> {
    let logged_in = make_logged_in_context(&session, &config);

    match db.get_member_for_lichess_id(&session.lichess_id)? {
        Some(member) => Ok(Template::render(
            "linked",
            make_linked_context(
                logged_in,
                member.org_id,
                member.exp_year,
                can_use_form(&session, &config, &db)?,
                &config.expiry,
            ),
        )),
        None => Ok(Template::render("notlinked", logged_in)),
    }
}

fn can_use_form(
    session: &Session,
    config: &State<Config>,
    db: &State<OrgDbClient>,
) -> Result<bool, ErrorBox> {
    let timezone = org::timezone_from_string(&config.org.timezone)?;
    db.get_member_for_lichess_id(&session.lichess_id)
        .map(|maybe_member| match maybe_member {
            Some(member) => org::is_past_expiry(
                member.exp_year,
                timezone,
                config.expiry.membership_month,
                config.expiry.membership_day,
            ),
            None => true,
        })
}

#[get("/link")]
fn show_form(
    session: Session,
    config: State<Config>,
    db: State<OrgDbClient>,
) -> Result<Result<Template, Redirect>, ErrorBox> {
    if !can_use_form(&session, &config, &db)? {
        Ok(Err(Redirect::to(uri!(index))))
    } else {
        Ok(Ok(Template::render(
            "form",
            make_error_context(make_logged_in_context(&session, &config), ""),
        )))
    }
}

#[get("/link", rank = 2)]
fn form_redirect_index() -> Redirect {
    Redirect::to(uri!(index))
}

fn org_id_unused(
    org_id: &str,
    session: &Session,
    db: &State<OrgDbClient>,
) -> Result<bool, ErrorBox> {
    match db.get_member_for_org_id(&org_id)? {
        Some(member) => Ok(session.lichess_id == member.lichess_id),
        None => Ok(true),
    }
}

#[derive(FromForm)]
struct OrgInfo {
    #[form(field = "org-id")]
    org_id: String,
    #[form(field = "org-password")]
    org_password: String,
}

#[post("/link", data = "<form>")]
fn link_memberships(
    form: Option<Form<OrgInfo>>,
    session: Session,
    config: State<Config>,
    db: State<OrgDbClient>,
    http_client: State<reqwest::Client>,
) -> Result<Result<Redirect, Template>, ErrorBox> {
    if !can_use_form(&session, &config, &db)? {
        return Ok(Ok(Redirect::to(uri!(index))));
    }

    let logged_in = make_logged_in_context(&session, &config);

    let timezone = org::timezone_from_string(&config.org.timezone)?;

    Ok(match form {
        Some(org_info) => {
            match azolve::verify_user(
                    &http_client,
                    &org_info.org_id,
                    &org_info.org_password,
                    &config.azolve.api,
                    &config.azolve.api_pwd,
                    &config.azolve.api_token,
                    &config.org.authentication_secret,
                ) {
                Ok(true) => {
                    if lichess::join_team(
                        &http_client,
                        &session.oauth_token,
                        &config.lichess.domain,
                        &config.org.team_id,
                    ) {
                        if org_id_unused(&org_info.org_id, &session, &db)? {
                            db.register_member(
                                &org_info.org_id,
                                &session.lichess_id,
                                org::current_year(timezone)
                                    + (if org::is_past_expiry_this_year(timezone, config.expiry.membership_month, config.expiry.membership_day) {
                                        1
                                    } else {
                                        0
                                    }),
                            )?;
                            Ok(Redirect::to(uri!(index)))
                        } else {
                            Err(Template::render("form", make_error_context(logged_in, "This membership is already linked to a Lichess account.")))
                        }
                    } else {
                        Err(Template::render("form", make_error_context(logged_in, "Could not add you to the Lichess team, please try again later.")))
                    }
                }
                Ok(false) => {
                    Err(Template::render("form", make_error_context(logged_in, "Membership verification failed, please check your member ID and password.")))
                }
                _ => {
                    Err(Template::render("form", make_error_context(logged_in, "At the moment we're unable to verify your membership. Please try again later.")))
                }
            }
        }
        None => Err(Template::render(
            "form",
            make_error_context(logged_in, "Invalid form data."),
        )),
    })
}

#[post("/link", rank = 2)]
fn try_link_unauthenticated() -> Redirect {
    Redirect::to(uri!(index))
}

#[post("/logout")]
fn logout(cookies: Cookies<'_>, config: State<Config>) -> Template {
    session::remove_session(cookies);
    Template::render("redirect", &empty_context(&config))
}

#[get("/admin")]
fn admin(
    session: Session,
    config: State<Config>,
    db: State<OrgDbClient>,
) -> Result<Result<Template, Status>, ErrorBox> {
    let logged_in = make_logged_in_context(&session, &config);

    if logged_in.admin {
        let members = db.get_members()?;
        let ref_count = db.referral_count()?;
        Ok(Ok(Template::render(
            "admin",
            make_admin_context(logged_in, ref_count, members),
        )))
    } else {
        Ok(Err(Status::Forbidden))
    }
}

#[get("/admin", rank = 2)]
fn admin_unauthed() -> Redirect {
    Redirect::to(uri!(index))
}

#[get("/admin/user-json")]
fn admin_user_json(
    session: Session,
    config: State<Config>,
    db: State<OrgDbClient>,
) -> Result<Result<rocket::response::content::Json<String>, Status>, ErrorBox> {
    let logged_in = make_logged_in_context(&session, &config);

    if logged_in.admin {
        let members = db.get_members()?;
        let mut map: HashMap<String, serde_json::Value> = HashMap::new();
        for member in members {
            let details = json!([member.lichess_id, 0]);
            map.insert(member.org_id.to_string(), details);
        }
        Ok(Ok(rocket::response::content::Json(serde_json::to_string(
            &map,
        )?)))
    } else {
        Ok(Err(Status::Forbidden))
    }
}

#[get("/admin/kick/<who>")]
fn admin_kick(who: String, session: Session, config: State<Config>) -> Result<Template, Status> {
    let logged_in = make_logged_in_context(&session, &config);

    if logged_in.admin {
        Ok(Template::render(
            "kickconfirm",
            make_kick_confirm_context(logged_in, who),
        ))
    } else {
        Err(Status::Forbidden)
    }
}

#[post("/admin/kick/<who>")]
fn admin_kick_confirmed(
    who: String,
    session: Session,
    config: State<Config>,
    db: State<OrgDbClient>,
    http_client: State<reqwest::Client>,
) -> Result<Result<Redirect, Status>, ErrorBox> {
    let logged_in = make_logged_in_context(&session, &config);

    if logged_in.admin {
        db.remove_membership_by_lichess_id(&who)?;
        lichess::try_kick_from_team(
            &http_client,
            &config.lichess.personal_api_token,
            &config.lichess.domain,
            &config.org.team_id,
            &who,
        )?;
        Ok(Ok(Redirect::to(uri!(admin))))
    } else {
        Ok(Err(Status::Forbidden))
    }
}

#[get("/org-ref")]
fn referral(
    session: Session,
    config: State<Config>,
    db: State<OrgDbClient>,
) -> Result<Redirect, ErrorBox> {
    db.referral_click(&session.lichess_id)?;
    Ok(Redirect::to(config.org.referral_link.clone()))
}

fn main() {
    let config_contents = fs::read_to_string("Config.toml").expect("Cannot read Config.toml");
    let config: Config = toml::from_str(&config_contents).expect("Invalid Config.toml");

    let db_client = db::connect(&config.server.postgres_options).unwrap();

    expwatch::launch(
        db_client.clone(),
        config.lichess.domain.clone(),
        config.org.team_id.clone(),
        config.lichess.personal_api_token.clone(),
        config.server.expiry_check_interval_seconds,
        org::timezone_from_string(&config.org.timezone).unwrap(),
        config.expiry.renewal_month,
        config.expiry.renewal_day,
    );

    let http_client = reqwest::Client::new();

    rocket::ignite()
        .attach(Template::fairing())
        .manage(config)
        .manage(http_client)
        .manage(db_client)
        .mount(
            "/",
            routes![
                index,
                auth,
                oauth_redirect,
                manage_authed,
                show_form,
                form_redirect_index,
                link_memberships,
                logout,
                try_link_unauthenticated,
                admin,
                admin_unauthed,
                admin_user_json,
                admin_kick,
                admin_kick_confirmed,
                referral
            ],
        )
        .launch();
}
