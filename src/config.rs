use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub url: String,
    pub lichess: String,
    pub client_id: String,
    pub client_secret: String,
    pub connection_string: String,
}
