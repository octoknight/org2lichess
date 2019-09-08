use crate::types::*;
use reqwest::{Client, Method, Request, Url};

pub fn verify_user(
    http_client: &Client,
    member_id: &str,
    member_password: &str,
    azolve_url: &str,
    azolve_password: &str,
    azolve_token: &str,
    auth_secret: &str,
) -> Result<bool, ErrorBox> {
    let mut url = Url::parse(azolve_url)?;
    {
        let mut query = url.query_pairs_mut();
        query.append_pair("userId", "AzolveAPI");
        query.append_pair("password", &azolve_password);
        query.append_pair("clientReference", "ECF");
        query.append_pair("objectName", "Cus_SSO_Pin");
        query.append_pair("objectType", "sp");
        query.append_pair(
            "parameters",
            &format!(
                "MID|{};{}|{};Token|{}",
                member_id, auth_secret, member_password, azolve_token
            ),
        );
    }
    let req = Request::new(Method::GET, url);
    let response = http_client.execute(req)?.text()?;
    println!("{}", response);
    Ok(response.trim() == "[[\"Return Code\",\"Message\"],[\"1\",\"Success\"]]")
}
