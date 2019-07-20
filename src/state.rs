use crate::config::Config;
use std::sync::RwLock;

pub struct State {
    pub config: Config,
    pub oauth_state: String,
    pub http_client: reqwest::Client,
    pub db: RwLock<postgres::Client>,
}
