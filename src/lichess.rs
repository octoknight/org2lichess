use crate::types::*;
use reqwest::header::*;
use reqwest::{Client, Method, Request, Url};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct OAuthToken {
    token_type: String,
    pub access_token: String,
}

#[derive(Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
}

#[derive(Deserialize)]
pub struct MaybeOk {
    pub ok: bool,
}

fn create_request(
    method: Method,
    url: String,
    accept: &str,
    authorization: String,
) -> Result<Request, ErrorBox> {
    let mut req = Request::new(method, Url::parse(&url)?);
    let headers = req.headers_mut();
    headers.insert(ACCEPT, accept.parse()?);
    headers.insert(AUTHORIZATION, authorization.parse()?);
    Ok(req)
}

pub fn get_user(
    token: &OAuthToken,
    http_client: &Client,
    lichess_domain: &str,
) -> Result<User, ErrorBox> {
    let req = create_request(
        Method::GET,
        format!("https://{}/api/account", lichess_domain),
        "application/json",
        format!("{} {}", token.token_type, token.access_token),
    )?;
    let response: User = http_client.execute(req)?.json()?;
    Ok(response)
}

pub fn oauth_token_from_code(
    code: &str,
    http_client: &Client,
    lichess_domain: &str,
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
) -> Result<OAuthToken, ErrorBox> {
    let mut req = Request::new(
        Method::POST,
        Url::parse(&format!("https://oauth.{}/oauth", lichess_domain))?,
    );
    let body = req.body_mut();
    *body = Some(
        format!(
            "grant_type=authorization_code&code={}&redirect_uri={}&client_id={}&client_secret={}",
            code, redirect_uri, client_id, client_secret
        )
        .into(),
    );
    let headers = req.headers_mut();
    headers.insert(ACCEPT, "application/json".parse()?);
    headers.insert(CONTENT_TYPE, "application/x-www-form-urlencoded".parse()?);
    let response: OAuthToken = http_client.execute(req)?.json()?;
    Ok(response)
}

fn try_join_team(
    http_client: &Client,
    token: &str,
    lichess_domain: &str,
    team_id: &str,
) -> Result<bool, ErrorBox> {
    let req = create_request(
        Method::POST,
        format!("https://{}/team/{}/join", lichess_domain, team_id),
        "application/json",
        format!("Bearer {}", token),
    )?;
    let response: MaybeOk = http_client.execute(req)?.json()?;
    Ok(response.ok)
}

pub fn join_team(http_client: &Client, token: &str, lichess_domain: &str, team_id: &str) -> bool {
    try_join_team(http_client, token, lichess_domain, team_id).unwrap_or(false)
}

fn try_kick_from_team(
    http_client: &Client,
    token: &str,
    lichess_domain: &str,
    team_id: &str,
    user_id: &str,
) -> Result<bool, ErrorBox> {
    let req = create_request(
        Method::POST,
        format!(
            "https://{}/team/{}/kick/{}",
            lichess_domain, team_id, user_id
        ),
        "application/json",
        format!("Bearer {}", token),
    )?;
    let response: MaybeOk = http_client.execute(req)?.json()?;
    Ok(response.ok)
}

pub fn kick_from_team(
    http_client: &Client,
    token: &str,
    lichess_domain: &str,
    team_id: &str,
    user_id: &str,
) -> bool {
    try_kick_from_team(http_client, token, lichess_domain, team_id, user_id).unwrap_or(false)
}
