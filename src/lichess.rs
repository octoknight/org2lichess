use crate::types::*;
use reqwest::header::*;
use reqwest::{Client, Request};
use reqwest::{Method, Url};
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
    headers.insert(ORIGIN, "https://lichess.org".parse()?);
    Ok(req)
}

pub async fn get_user(
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
    let response: User = http_client.execute(req).await?.json().await?;
    Ok(response)
}

pub async fn oauth_token_from_code(
    code: &str,
    http_client: &Client,
    client_id: &str,
    code_verifier: &str,
    redirect_uri: &str,
) -> Result<OAuthToken, Box<dyn std::error::Error>> {
    let mut req = Request::new(Method::POST, Url::parse("https://lichess.org/api/token")?);
    let body = req.body_mut();
    *body = Some(
        format!(
            "grant_type=authorization_code&code={}&redirect_uri={}&client_id={}&code_verifier={}",
            code,
            urlencoding::encode(redirect_uri),
            client_id,
            code_verifier
        )
        .into(),
    );
    let headers = req.headers_mut();
    headers.insert(ACCEPT, "application/json".parse()?);
    headers.insert(CONTENT_TYPE, "application/x-www-form-urlencoded".parse()?);

    let response: OAuthToken = http_client.execute(req).await?.json().await?;
    Ok(response)
}

async fn try_join_team(
    http_client: &Client,
    token: &str,
    lichess_domain: &str,
    team_id: &str,
    team_password: &str,
) -> Result<bool, ErrorBox> {
    let mut req = create_request(
        Method::POST,
        format!("https://{}/team/{}/join", lichess_domain, team_id),
        "application/json",
        format!("Bearer {}", token),
    )?;
    let body = req.body_mut();
    *body = Some(("password=".to_owned() + &urlencoding::encode(team_password)).into());
    let headers = req.headers_mut();
    headers.insert(CONTENT_TYPE, "application/x-www-form-urlencoded".parse()?);
    let response: MaybeOk = http_client.execute(req).await?.json().await?;
    Ok(response.ok)
}

pub async fn join_team(
    http_client: &Client,
    token: &str,
    lichess_domain: &str,
    team_id: &str,
    team_password: &str,
) -> bool {
    try_join_team(http_client, token, lichess_domain, team_id, team_password)
        .await
        .unwrap_or(false)
}

pub async fn try_kick_from_team(
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
    let response: MaybeOk = http_client.execute(req).await?.json().await?;
    Ok(response.ok)
}

pub async fn kick_from_team(
    http_client: &Client,
    token: &str,
    lichess_domain: &str,
    team_id: &str,
    user_id: &str,
) -> bool {
    try_kick_from_team(http_client, token, lichess_domain, team_id, user_id)
        .await
        .unwrap_or(false)
}
