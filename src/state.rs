use crate::config::Config;
use reqwest::Client;

pub struct State {
    pub config: Config,
    pub oauth_state: String,
    pub http_client: Client,
}
