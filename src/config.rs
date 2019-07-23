use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub url: String,
    pub lichess: String,
    pub client_id: String,
    pub client_secret: String,
    pub connection_string: String,
    pub azolve_api: String,
    pub azolve_api_pwd: String,
    pub team_id: String,
    pub lichess_admin_id: String,
    pub personal_api_token: String,
    pub expiry_check_interval_seconds: u64,
}
