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
) -> Result<bool, ErrorBox> {
    let mut url = Url::parse(azolve_url_stage1)?;
    {
        let mut query = url.query_pairs_mut();
        query.append_pair("source", &member_password);
    }
    let req = Request::new(Method::GET, url);
    let response = http_client.execute(req)?.text()?;
    let enc_password = response.trim().trim_matches('"');

    let mut url = Url::parse(azolve_url_stage2)?;
    {
        let mut query = url.query_pairs_mut();
        query.append_pair("userId", "AzolveAPI");
        query.append_pair("password", &azolve_password);
        query.append_pair("clientReference", "ECF");
        query.append_pair("objectName", "Cus_SSO");
        query.append_pair("objectType", "sp");
        query.append_pair(
            "parameters",
            &format!(
                "MID|ME{};Password|{};Token|{}",
                member_id, enc_password, azolve_token
            ),
        );
    }
    println!("{}", url);
    let req = Request::new(Method::GET, url);
    let response = http_client.execute(req)?.text()?;
    println!("{}", &response);
    Ok(response.trim() == "[[\"Return Code\",\"Message\"],[\"1\",\"Success\"]]")
}
