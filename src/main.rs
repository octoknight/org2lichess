use base64::Engine;
use rocket::form::Form;
use rocket::http::{CookieJar, Status};
use rocket::response::{status, Redirect};
use rocket::serde::json::Json;
use rocket::{get, launch, post, routes, uri, FromForm, State};
use rocket_dyn_templates::Template;
use serde_json::json;
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

use base64::engine::general_purpose::STANDARD as BASE64;
use config::Config;
use db::OrgDbClient;
use randstr::random_string;
use session::Session;
use sha2::{Digest, Sha256};
use tempctx::*;
use types::*;

type ErrorStatus = status::Custom<&'static str>;

fn to_500(e: ErrorBox) -> ErrorStatus {
    println!("Internal server error: {}", e);
    status::Custom(Status::InternalServerError, "Internal Server Error")
}

#[get("/", rank = 2)]
async fn index(config: &State<Config>) -> Template {
    Template::render("index", &empty_context(&config))
}

#[get("/auth")]
async fn auth(config: &State<Config>, cookies: &CookieJar<'_>) -> Result<Redirect, ErrorStatus> {
    let oauth_state = random_string().map_err(|e| to_500(Box::new(e)))?;
    session::set_oauth_state_cookie(cookies, &oauth_state);

    let code_verifier = random_string().map_err(|e| to_500(Box::new(e)))?;
    session::set_oauth_code_verifier(cookies, &code_verifier);

    let mut hasher = Sha256::default();
    hasher.update(code_verifier.as_bytes());
    let hash_result = BASE64
        .encode(hasher.finalize())
        .replace("=", "")
        .replace("+", "-")
        .replace("/", "_");

    let url = format!(
        "https://lichess.org/oauth?response_type=code\
            &client_id={}&scope=team:write\
            &redirect_uri={}%2Foauth_redirect\
            &state={}&code_challenge_method=S256&code_challenge={}",
        urlencoding::encode(&config.lichess.client_id),
        urlencoding::encode(&config.server.url),
        oauth_state,
        &hash_result
    );

    Ok(Redirect::to(url))
}

#[get("/oauth_redirect?<code>&<state>")]
async fn oauth_redirect(
    cookies: &CookieJar<'_>,
    code: String,
    state: String,
    config: &State<Config>,
    http_client: &State<reqwest::Client>,
) -> Result<Result<Template, Status>, ErrorStatus> {
    match (
        session::pop_oauth_state(cookies).map(|v| v == state),
        session::pop_oauth_code_verifier(cookies),
    ) {
        (Some(true), Some(code_verifier)) => {
            let token = lichess::oauth_token_from_code(
                &code,
                &http_client,
                &config.lichess.client_id,
                &code_verifier,
                &format!("{}/oauth_redirect", config.server.url),
            )
            .await
            .unwrap();
            let user = lichess::get_user(&token, &http_client, "lichess.org")
                .await
                .unwrap();
            session::set_session(
                cookies,
                Session {
                    lichess_id: user.id,
                    lichess_username: user.username,
                    oauth_token: token.access_token,
                },
            )
            .map_err(to_500)?;
            Ok(Ok(Template::render("redirect", &empty_context(&config))))
        }
        _ => Ok(Err(Status::BadRequest)),
    }
}

#[get("/")]
async fn manage_authed(
    session: Session,
    config: &State<Config>,
    db: &State<OrgDbClient>,
) -> Result<Template, ErrorStatus> {
    let logged_in = make_logged_in_context(&session, &config);

    match db
        .get_member_for_lichess_id(&session.lichess_id)
        .await
        .map_err(to_500)?
    {
        Some(member) => Ok(Template::render(
            "linked",
            make_linked_context(
                logged_in,
                member.org_id,
                member.exp_year,
                can_use_form(&session, &config, &db).await.map_err(to_500)?,
                &config.expiry,
            ),
        )),
        None => Ok(Template::render("notlinked", logged_in)),
    }
}

async fn can_use_form(
    session: &Session,
    config: &State<Config>,
    db: &State<OrgDbClient>,
) -> Result<bool, ErrorBox> {
    let timezone = org::timezone_from_string(&config.org.timezone)?;
    db.get_member_for_lichess_id(&session.lichess_id)
        .await
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
async fn show_form(
    session: Session,
    config: &State<Config>,
    db: &State<OrgDbClient>,
) -> Result<Result<Template, Redirect>, ErrorStatus> {
    if !can_use_form(&session, &config, &db).await.map_err(to_500)? {
        Ok(Err(Redirect::to(uri!(index))))
    } else {
        Ok(Ok(Template::render(
            "form",
            make_error_context(make_logged_in_context(&session, &config), ""),
        )))
    }
}

#[get("/link", rank = 2)]
async fn form_redirect_index() -> Redirect {
    Redirect::to(uri!(index))
}

async fn org_id_unused(
    org_id: &str,
    session: &Session,
    db: &State<OrgDbClient>,
) -> Result<bool, ErrorBox> {
    match db.get_member_for_org_id(&org_id).await? {
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
async fn link_memberships(
    form: Option<Form<OrgInfo>>,
    session: Session,
    config: &State<Config>,
    db: &State<OrgDbClient>,
    http_client: &State<reqwest::Client>,
) -> Result<Result<Redirect, Template>, ErrorStatus> {
    if !can_use_form(&session, &config, &db).await.map_err(to_500)? {
        return Ok(Ok(Redirect::to(uri!(index))));
    }

    let logged_in = make_logged_in_context(&session, &config);

    let timezone = org::timezone_from_string(&config.org.timezone).map_err(to_500)?;

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
                    &config.azolve.test_backdoor_member_id,
                    &config.azolve.test_backdoor_password,
                ).await {
                Ok(true) => {
                    if org_id_unused(&org_info.org_id, &session, &db).await.map_err(to_500)? {
                        if lichess::join_team(
                            &http_client,
                            &session.oauth_token,
                            "lichess.org",
                            &config.org.team_id,
                            &config.lichess.team_password,
                        ).await {
                            db.register_member(
                                &org_info.org_id,
                                &session.lichess_id,
                                org::current_year(timezone)
                                    + (if org::is_past_expiry_this_year(timezone, config.expiry.membership_month, config.expiry.membership_day) {
                                        1
                                    } else {
                                        0
                                    }),
                            ).await.map_err(to_500)?;
                            Ok(Redirect::to(uri!(index)))
                        } else {
                            Err(Template::render("form", make_error_context(logged_in, "Could not add you to the Lichess team, please try again later.")))
                        }
                    } else {
                        Err(Template::render("form", make_error_context(logged_in, "This membership is already linked to a Lichess account.")))
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
async fn try_link_unauthenticated() -> Redirect {
    Redirect::to(uri!(index))
}

#[post("/logout")]
async fn logout(cookies: &CookieJar<'_>, config: &State<Config>) -> Template {
    session::remove_session(cookies);
    Template::render("redirect", &empty_context(&config))
}

#[get("/admin")]
async fn admin(
    session: Session,
    config: &State<Config>,
    db: &State<OrgDbClient>,
) -> Result<Result<Template, Status>, ErrorStatus> {
    let logged_in = make_logged_in_context(&session, &config);

    if logged_in.admin {
        let members = db.get_members().await.map_err(to_500)?;
        let ref_count = db.referral_count().await.map_err(to_500)?;
        Ok(Ok(Template::render(
            "admin",
            make_admin_context(logged_in, ref_count, members),
        )))
    } else {
        Ok(Err(Status::Forbidden))
    }
}

#[get("/admin", rank = 2)]
async fn admin_unauthed() -> Redirect {
    Redirect::to(uri!(index))
}

#[get("/admin/user-json")]
async fn admin_user_json(
    session: Session,
    config: &State<Config>,
    db: &State<OrgDbClient>,
) -> Result<Result<Json<HashMap<String, serde_json::Value>>, Status>, ErrorStatus> {
    let logged_in = make_logged_in_context(&session, &config);

    if logged_in.admin {
        let members = db.get_members().await.map_err(to_500)?;
        let mut map: HashMap<String, serde_json::Value> = HashMap::new();
        for member in members {
            let details = json!([member.lichess_id, 0]);
            map.insert(member.org_id.to_string(), details);
        }
        Ok(Ok(Json(map)))
    } else {
        Ok(Err(Status::Forbidden))
    }
}

#[get("/admin/kick/<who>")]
async fn admin_kick(
    who: String,
    session: Session,
    config: &State<Config>,
) -> Result<Template, Status> {
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
async fn admin_kick_confirmed(
    who: String,
    session: Session,
    config: &State<Config>,
    db: &State<OrgDbClient>,
    http_client: &State<reqwest::Client>,
) -> Result<Result<Redirect, Status>, ErrorStatus> {
    let logged_in = make_logged_in_context(&session, &config);

    if logged_in.admin {
        db.remove_membership_by_lichess_id(&who)
            .await
            .map_err(to_500)?;
        lichess::try_kick_from_team(
            &http_client,
            &config.lichess.personal_api_token,
            "lichess.org",
            &config.org.team_id,
            &who,
        )
        .await
        .map_err(to_500)?;
        Ok(Ok(Redirect::to(uri!(admin))))
    } else {
        Ok(Err(Status::Forbidden))
    }
}

#[get("/org-ref")]
async fn referral(
    session: Session,
    config: &State<Config>,
    db: &State<OrgDbClient>,
) -> Result<Redirect, ErrorStatus> {
    db.referral_click(&session.lichess_id)
        .await
        .map_err(to_500)?;
    Ok(Redirect::to(config.org.referral_link.clone()))
}

#[launch]
async fn rocket() -> _ {
    let config_contents = fs::read_to_string("Config.toml").expect("Cannot read Config.toml");
    let config: Config = toml::from_str(&config_contents).expect("Invalid Config.toml");

    let db_client = db::connect(&config.server.postgres_options).await.unwrap();

    if config.expiry.enable {
        expwatch::launch(
            db_client.clone(),
            String::from("lichess.org"),
            config.org.team_id.clone(),
            config.lichess.personal_api_token.clone(),
            config.server.expiry_check_interval_seconds,
            org::timezone_from_string(&config.org.timezone).unwrap(),
            config.expiry.renewal_month,
            config.expiry.renewal_day,
        );
    }

    let http_client = reqwest::Client::new();

    rocket::build()
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
}
