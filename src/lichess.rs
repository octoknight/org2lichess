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

pub fn get_user(
    token: &OAuthToken,
    http_client: &Client,
    lichess_domain: &str,
) -> Result<User, Box<dyn std::error::Error>> {
    let mut req = Request::new(
        Method::GET,
        Url::parse(&format!("https://{}/api/account", lichess_domain))?,
    );
    let headers = req.headers_mut();
    headers.insert(ACCEPT, "application/json".parse()?);
    headers.insert(
        AUTHORIZATION,
        format!("{} {}", token.token_type, token.access_token).parse()?,
    );
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
) -> Result<OAuthToken, Box<dyn std::error::Error>> {
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
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut req = Request::new(
        Method::POST,
        Url::parse(&format!("https://{}/team/{}/join", lichess_domain, team_id))?,
    );
    let headers = req.headers_mut();
    headers.insert(ACCEPT, "application/json".parse()?);
    headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse()?);
    let response: MaybeOk = http_client.execute(req)?.json()?;
    Ok(response.ok)
}

pub fn join_team(http_client: &Client, token: &str, lichess_domain: &str, team_id: &str) -> bool {
    try_join_team(http_client, token, lichess_domain, team_id).unwrap_or(false)
}
