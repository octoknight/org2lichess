use reqwest::header::*;
use reqwest::{Client, Method, Request, Url};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct OAuthToken {
    scopes: String,
    expires_in: i32,
    token_type: String,
    access_token: String,
}

#[derive(Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
}

pub fn get_username(
    token: &OAuthToken,
    http_client: &Client,
    lichess_domain: &str,
) -> Result<User, Box<std::error::Error>> {
    let mut req = Request::new(
        Method::GET,
        Url::parse(&format!("https://{}/api/account", lichess_domain))?,
    );
    let mut headers = req.headers_mut();
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
) -> Result<OAuthToken, Box<std::error::Error>> {
    let mut req = Request::new(
        Method::POST,
        Url::parse(&format!("https://oauth.{}/oauth", lichess_domain))?,
    );
    let mut body = req.body_mut();
    *body = Some(
        format!(
            "grant_type=authorization_code&code={}&redirect_uri={}&client_id={}&client_secret={}",
            code, redirect_uri, client_id, client_secret
        )
        .into(),
    );
    let mut headers = req.headers_mut();
    headers.insert(ACCEPT, "application/json".parse()?);
    headers.insert(CONTENT_TYPE, "application/x-www-form-urlencoded".parse()?);
    let response: OAuthToken = http_client.execute(req)?.json()?;
    Ok(response)
}
