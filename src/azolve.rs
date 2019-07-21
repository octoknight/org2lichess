use reqwest::header::*;
use reqwest::{Client, Method, Request, Url};

pub fn verify_user(http_client: &Client, member_id: i32, member_password: &str, azolve_url: &str, azolve_password: &str) -> Result<bool, Box<std::error::Error>> {
    let mut req = Request::new(
        Method::GET,
        Url::parse(&format!("{}?userId=AzolveAPI&password={}&clientReference=ECF&objectName=Cus_SSO&objectType=sp&meters=MID|{};{}",
        azolve_url,
        azolve_password,
        member_id,
        member_password))?,
    );
    let response = http_client.execute(req)?.text()?;
    Ok(response == "{Verification successful}")
}