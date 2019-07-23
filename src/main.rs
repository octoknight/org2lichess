#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate chrono;
extern crate postgres;
extern crate rand;
extern crate reqwest;
extern crate rocket_contrib;
extern crate serde;
extern crate toml;

use rand::Rng;
use rocket::http::Cookies;
use rocket::request::Form;
use rocket::response::Redirect;
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::fs;
use std::sync::RwLock;

mod azolve;
mod config;
mod db;
mod ecf;
mod lichess;
mod session;
mod state;

use config::Config;
use db::EcfDbClient;
use session::Session;

fn empty_context() -> HashMap<u8, u8> {
    HashMap::new()
}

#[get("/", rank = 2)]
fn index() -> Template {
    Template::render("index", &empty_context())
}

#[get("/auth")]
fn auth(state: rocket::State<state::State>) -> Redirect {
    Redirect::to(
        format!("https://oauth.{}/oauth/authorize?response_type=code&client_id={}&redirect_uri={}/oauth_redirect&scope=team:write&state={}",
        state.config.lichess, state.config.client_id, state.config.url, state.oauth_state)
    )
}

#[get("/oauth_redirect?<code>&<state>")]
fn oauth_redirect(
    cookies: Cookies<'_>,
    code: String,
    state: String,
    rocket_state: rocket::State<state::State>,
) -> Template {
    let token = lichess::oauth_token_from_code(
        &code,
        &rocket_state.http_client,
        &rocket_state.config.lichess,
        &rocket_state.config.client_id,
        &rocket_state.config.client_secret,
        &format!("{}/oauth_redirect", rocket_state.config.url),
    )
    .unwrap();
    let user = lichess::get_user(
        &token,
        &rocket_state.http_client,
        &rocket_state.config.lichess,
    )
    .unwrap();
    session::set_session(
        cookies,
        Session {
            lichess_id: user.id,
            lichess_username: user.username,
            oauth_token: token.access_token,
        },
    );
    Template::render("redirect", &empty_context())
}

#[get("/")]
fn manage_authed(
    session: Session,
    state: rocket::State<state::State>,
) -> Result<Template, postgres::Error> {
    let mut ctx: HashMap<&str, &str> = HashMap::new();

    match state.db.get_member_for_lichess_id(&session.lichess_id)? {
        Some(member) => {
            ctx.insert("lichess", &session.lichess_username);
            let memid_str = &member.ecf_id.to_string();
            ctx.insert("ecf", &memid_str);
            let exp_str = &member.exp_year.to_string();
            ctx.insert("exp", &exp_str);
            ctx.insert(
                "can_renew",
                if can_use_form(&session, &state)? {
                    "true"
                } else {
                    "false"
                },
            );
            Ok(Template::render("linked", &ctx))
        }
        None => {
            ctx.insert("lichess", &session.lichess_username);
            Ok(Template::render("notlinked", &ctx))
        }
    }
}

fn can_use_form(
    session: &Session,
    state: &rocket::State<state::State>,
) -> Result<bool, postgres::Error> {
    state
        .db
        .get_member_for_lichess_id(&session.lichess_id)
        .map(|maybe_member| match maybe_member {
            Some(member) => ecf::is_past_expiry(member.exp_year),
            None => true,
        })
}

#[get("/link")]
fn show_form(
    session: Session,
    state: rocket::State<state::State>,
) -> Result<Result<Template, Redirect>, postgres::Error> {
    if !can_use_form(&session, &state)? {
        Ok(Err(Redirect::to(uri!(index))))
    } else {
        let mut ctx: HashMap<&str, &str> = HashMap::new();
        ctx.insert("lichess", &session.lichess_username);
        ctx.insert("error", "");

        Ok(Ok(Template::render("form", &ctx)))
    }
}

#[get("/link", rank = 2)]
fn form_redirect_index() -> Redirect {
    Redirect::to(uri!(index))
}

fn ecf_id_unused(
    ecf_id: i32,
    session: &Session,
    state: &rocket::State<state::State>,
) -> Result<bool, postgres::Error> {
    match state.db.get_member_for_ecf_id(ecf_id)? {
        Some(member) => Ok(&session.lichess_id == &member.lichess_id),
        None => Ok(true),
    }
}

#[derive(FromForm)]
struct EcfInfo {
    #[form(field = "ecf-id")]
    ecf_id: i32,
    #[form(field = "ecf-password")]
    ecf_password: String,
}

#[post("/link", data = "<form>")]
fn link_memberships(
    form: Option<Form<EcfInfo>>,
    session: Session,
    state: rocket::State<state::State>,
) -> Result<Result<Redirect, Template>, Box<dyn std::error::Error>> {
    if !can_use_form(&session, &state)? {
        return Ok(Ok(Redirect::to(uri!(index))));
    }

    let mut ctx: HashMap<&str, &str> = HashMap::new();
    ctx.insert("lichess", &session.lichess_username);

    Ok(match form {
        Some(ecf_info) => {
            if ecf_info.ecf_id < 0 {
                ctx.insert("error", "Invalid ECF member ID.");
                Err(Template::render("form", &ctx))
            } else {
                match azolve::verify_user(
                    &state.http_client,
                    ecf_info.ecf_id,
                    &ecf_info.ecf_password,
                    &state.config.azolve_api,
                    &state.config.azolve_api_pwd,
                ) {
                    Ok(true) => {
                        if lichess::join_team(
                            &state.http_client,
                            &session.oauth_token,
                            &state.config.lichess,
                            &state.config.team_id,
                        ) {
                            if ecf_id_unused(ecf_info.ecf_id, &session, &state)? {
                                state.db.register_member(
                                    ecf_info.ecf_id,
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
                                ctx.insert(
                                    "error",
                                    "This ECF membership is already linked to a Lichess account.",
                                );
                                Err(Template::render("form", &ctx))
                            }
                        } else {
                            ctx.insert(
                                "error",
                                "Could not add you to the Lichess team, please try again later.",
                            );
                            Err(Template::render("form", &ctx))
                        }
                    }
                    Ok(false) => {
                        ctx.insert(
                                "error",
                                "Membership verification failed, please check your member ID and password",
                            );
                        Err(Template::render("form", &ctx))
                    }
                    _ => {
                        ctx.insert("error", "At the moment we're unable to verify your membership. Please try again later.");
                        Err(Template::render("form", &ctx))
                    }
                }
            }
        }
        None => {
            ctx.insert("error", "Invalid form data.");
            Err(Template::render("form", &ctx))
        }
    })
}

#[post("/logout")]
fn logout(cookies: Cookies<'_>) -> Template {
    session::remove_session(cookies);
    Template::render("redirect", &empty_context())
}

fn main() {
    let config_contents = fs::read_to_string("Config.toml").expect("Cannot read Config.toml");
    let config: Config = toml::from_str(&config_contents).expect("Invalid Config.toml");

    let mut rng = rand::thread_rng();
    let mut oauth_state_bytes: [u8; 12] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    for x in &mut oauth_state_bytes {
        *x = (rng.gen::<u8>() % 26) + 97;
    }
    let oauth_state = std::str::from_utf8(&oauth_state_bytes)
        .expect("Invalid OAuth state")
        .to_string();
    println!("OAuth state: {}", &oauth_state);

    let http_client = reqwest::Client::new();

    let db_client = RwLock::new(db::connect(&config.connection_string).unwrap());

    rocket::ignite()
        .attach(Template::fairing())
        .manage(state::State {
            config,
            oauth_state,
            http_client,
            db: db_client,
        })
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
                logout
            ],
        )
        .launch();
}
