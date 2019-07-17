#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate postgres;
extern crate rand;
extern crate reqwest;
extern crate rocket_contrib;
extern crate serde;
extern crate toml;

use rand::Rng;
use reqwest::header::*;
use reqwest::{Method, Request, Url};
use rocket::http::Cookies;
use rocket::response::Redirect;
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::fs;

mod config;
mod db;
mod lichess;
mod session;
mod state;

use config::Config;
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
        format!("https://oauth.{}/oauth/authorize?response_type=code&client_id={}&redirect_uri={}/oauth_redirect&scope=&state={}",
        state.config.lichess, state.config.client_id, state.config.url, state.oauth_state)
    )
}

#[get("/oauth_redirect?<code>&<state>")]
fn oauth_redirect(
    mut cookies: Cookies<'_>,
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
    let user = lichess::get_username(
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
        },
    );
    Template::render("redirect", &empty_context())
}

#[get("/")]
fn manage_authed(session: Session) -> Template {
    let mut ctx: HashMap<&str, &str> = HashMap::new();
    ctx.insert("lichess_id", &session.lichess_id);
    ctx.insert("lichess_user", &session.lichess_username);
    Template::render("user", &ctx)
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

    rocket::ignite()
        .attach(Template::fairing())
        .manage(state::State {
            config,
            oauth_state,
            http_client,
        })
        .mount("/", routes![index, auth, oauth_redirect, manage_authed])
        .launch();
}
