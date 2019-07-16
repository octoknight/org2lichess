#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate postgres;
extern crate rand;
extern crate rocket_contrib;
extern crate serde;
extern crate toml;

use rand::Rng;
use rocket::response::Redirect;
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::fs;

mod config;
mod db;
mod state;

use config::Config;

fn empty_context() -> HashMap<u8, u8> {
    HashMap::new()
}

#[get("/")]
fn index() -> Template {
    Template::render("index", &empty_context())
}

#[get("/auth")]
fn auth(state: rocket::State<state::State>) -> Redirect {
    Redirect::to(
        format!("https://oauth.lichess.org/oauth/authorize?response_type=code&client_id={}&redirect_uri={}/oauth_redirect&scope=&state={}",
        state.config.client_id, state.config.url, state.oauth_state)
    )
}

#[get("/oauth_redirect")]
fn oauth_redirect() -> Template {
    Template::render("index", &empty_context())
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

    rocket::ignite()
        .attach(Template::fairing())
        .manage(state::State {
            config,
            oauth_state,
        })
        .mount("/", routes![index, auth])
        .launch();
}
