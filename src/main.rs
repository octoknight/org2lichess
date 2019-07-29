#![feature(proc_macro_hygiene, decl_macro, custom_attribute)]

#[macro_use]
extern crate rocket;
extern crate chrono;
extern crate postgres;
extern crate rand;
extern crate reqwest;
extern crate rocket_contrib;
extern crate serde;
extern crate toml;

use rocket::http::{Cookies, Status};
use rocket::request::Form;
use rocket::response::Redirect;
use rocket::State;
use rocket_contrib::templates::Template;
use std::fs;
use std::sync::RwLock;

mod azolve;
mod config;
mod db;
mod ecf;
mod expwatch;
mod lichess;
mod randstr;
mod session;
mod tempctx;
mod textlog;
mod types;

use config::Config;
use db::EcfDbClient;
use randstr::random_oauth_state;
use session::Session;
use tempctx::*;
use types::*;

#[get("/", rank = 2)]
fn index() -> Template {
    Template::render("index", &empty_context())
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
    match session::pop_oauth_state(&mut cookies).map(|v| &v == &state) {
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
            Ok(Ok(Template::render("redirect", &empty_context())))
        }
        _ => Ok(Err(Status::BadRequest)),
    }
}

#[get("/")]
fn manage_authed(
    session: Session,
    config: State<Config>,
    db: State<Db>,
) -> Result<Template, ErrorBox> {
    let logged_in = make_logged_in_context(&session, &config);

    match db.get_member_for_lichess_id(&session.lichess_id)? {
        Some(member) => Ok(Template::render(
            "linked",
            make_linked_context(
                logged_in,
                member.ecf_id,
                member.exp_year,
                can_use_form(&session, &db)?,
            ),
        )),
        None => Ok(Template::render("notlinked", logged_in)),
    }
}

fn can_use_form(session: &Session, db: &State<Db>) -> Result<bool, ErrorBox> {
    db.get_member_for_lichess_id(&session.lichess_id)
        .map(|maybe_member| match maybe_member {
            Some(member) => ecf::is_past_expiry(member.exp_year),
            None => true,
        })
}

#[get("/link")]
fn show_form(
    session: Session,
    config: State<Config>,
    db: State<Db>,
) -> Result<Result<Template, Redirect>, ErrorBox> {
    if !can_use_form(&session, &db)? {
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

fn ecf_id_unused(ecf_id: &str, session: &Session, db: &State<Db>) -> Result<bool, ErrorBox> {
    match db.get_member_for_ecf_id(&ecf_id)? {
        Some(member) => Ok(&session.lichess_id == &member.lichess_id),
        None => Ok(true),
    }
}

#[derive(FromForm)]
struct EcfInfo {
    #[form(field = "ecf-id")]
    ecf_id: String,
    #[form(field = "ecf-password")]
    ecf_password: String,
}

#[post("/link", data = "<form>")]
fn link_memberships(
    form: Option<Form<EcfInfo>>,
    session: Session,
    config: State<Config>,
    db: State<Db>,
    http_client: State<reqwest::Client>,
) -> Result<Result<Redirect, Template>, ErrorBox> {
    if !can_use_form(&session, &db)? {
        return Ok(Ok(Redirect::to(uri!(index))));
    }

    let logged_in = make_logged_in_context(&session, &config);

    Ok(match form {
        Some(ecf_info) => {
            match azolve::verify_user(
                    &http_client,
                    &ecf_info.ecf_id,
                    &ecf_info.ecf_password,
                    &config.azolve.api,
                    &config.azolve.api_pwd,
                ) {
                Ok(true) => {
                    if lichess::join_team(
                        &http_client,
                        &session.oauth_token,
                        &config.lichess.domain,
                        &config.lichess.team_id,
                    ) {
                        if ecf_id_unused(&ecf_info.ecf_id, &session, &db)? {
                            db.register_member(
                                &ecf_info.ecf_id,
                                &session.lichess_id,
                                ecf::current_london_year()
                                    + (if ecf::is_past_expiry_this_year() {
                                        1
                                    } else {
                                        0
                                    }),
                            )?;
                            Ok(Redirect::to(uri!(index)))
                        } else {
                            Err(Template::render("form", make_error_context(logged_in, "This ECF membership is already linked to a Lichess account.")))
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
fn logout(cookies: Cookies<'_>) -> Template {
    session::remove_session(cookies);
    Template::render("redirect", &empty_context())
}

#[get("/admin")]
fn admin(
    session: Session,
    config: State<Config>,
    db: State<Db>,
) -> Result<Result<Template, Status>, ErrorBox> {
    let logged_in = make_logged_in_context(&session, &config);

    if logged_in.admin {
        let members = db.get_members()?;
        Ok(Ok(Template::render(
            "admin",
            make_admin_context(logged_in, members),
        )))
    } else {
        Ok(Err(Status::Forbidden))
    }
}

fn main() {
    let config_contents = fs::read_to_string("Config.toml").expect("Cannot read Config.toml");
    let config: Config = toml::from_str(&config_contents).expect("Invalid Config.toml");

    let db_client1 = RwLock::new(db::connect(&config.server.db_connection_string).unwrap());

    expwatch::launch(
        db_client1,
        config.lichess.domain.clone(),
        config.lichess.team_id.clone(),
        config.lichess.personal_api_token.clone(),
        config.server.expiry_check_interval_seconds,
    );

    let http_client = reqwest::Client::new();

    let db_client2 = RwLock::new(db::connect(&config.server.db_connection_string).unwrap());

    rocket::ignite()
        .attach(Template::fairing())
        .manage(config)
        .manage(http_client)
        .manage(db_client2)
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
                admin
            ],
        )
        .launch();
}
