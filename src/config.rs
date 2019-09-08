use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Config {
    pub org: OrgConfig,
    pub expiry: ExpiryConfig,
    pub server: ServerConfig,
    pub lichess: LichessConfig,
    pub azolve: AzolveConfig,
}

#[derive(Serialize, Deserialize)]
pub struct OrgConfig {
    pub long_name: String,
    pub short_name: String,
    pub icon: String,
    pub image: String,
    pub team_id: String,
    pub timezone: String,
    pub referral_link: String,
    pub authentication_secret: String,
    pub authentication_secret_first_word: String,
    pub memberid_placeholder: String,
    pub password_placeholder: String,
    pub password_explanation: String,
    pub memberid_pattern: String,
}

#[derive(Deserialize)]
pub struct ExpiryConfig {
    pub membership_month: u32,
    pub membership_day: u32,
    pub renewal_month: u32,
    pub renewal_day: u32,
}

#[derive(Deserialize)]
pub struct ServerConfig {
    pub url: String,
    pub expiry_check_interval_seconds: u64,
    pub postgres_options: String,
}

#[derive(Deserialize)]
pub struct LichessConfig {
    pub domain: String,
    pub client_id: String,
    pub client_secret: String,
    pub team_admin: String,
    pub personal_api_token: String,
}

#[derive(Deserialize)]
pub struct AzolveConfig {
    pub api: String,
    pub api_pwd: String,
    pub api_token: String,
}
