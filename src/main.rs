#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;

use rocket_contrib::templates::Template;
use std::collections::HashMap;

fn empty_context() -> HashMap<u8, u8> {
    HashMap::new()
}

#[get("/")]
fn index() -> Template {
    Template::render("index", &empty_context())
}

fn main() {
    rocket::ignite()
        .attach(Template::fairing())
        .mount("/", routes![index])
        .launch();
}
