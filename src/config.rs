use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub lichess: LichessConfig,
    pub azolve: AzolveConfig,
}

#[derive(Deserialize)]
pub struct ServerConfig {
    pub url: String,
    pub expiry_check_interval_seconds: u64,
    pub db_connection_string: String,
}

#[derive(Deserialize)]
pub struct LichessConfig {
    pub domain: String,
    pub client_id: String,
    pub client_secret: String,
    pub team_id: String,
    pub team_admin: String,
    pub personal_api_token: String,
}

#[derive(Deserialize)]
pub struct AzolveConfig {
    pub api: String,
    pub api_pwd: String,
}
