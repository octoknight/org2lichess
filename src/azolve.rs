use crate::types::*;
use reqwest::{Client, Method, Request, Url};

pub fn verify_user(
    http_client: &Client,
    member_id: &str,
    member_password: &str,
    azolve_url_stage1: &str,
    azolve_url_stage2: &str,
    azolve_password: &str,
    azolve_token: &str,
    auth_secret: &str,
) -> Result<bool, ErrorBox> {
    let enc_password = if azolve_url_stage1 == "" {
        member_password.to_string()
    } else {
        let mut url = Url::parse(azolve_url_stage1)?;
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("source", &member_password);
        }
        let req = Request::new(Method::GET, url);
        let response = http_client.execute(req)?.text()?;
        let trimmed = response.trim().trim_matches('"');
        trimmed.to_string()
    };

    let mut url = Url::parse(azolve_url_stage2)?;
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
                member_id, auth_secret, enc_password, azolve_token
            ),
        );
    }
    let req = Request::new(Method::GET, url);
    let response = http_client.execute(req)?.text()?;
    println!("{}", response);
    Ok(response.trim() == "[[\"Return Code\",\"Message\"],[\"1\",\"Success\"]]")
}
