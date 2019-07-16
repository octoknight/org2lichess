use crate::config::Config;

pub struct State {
    pub config: Config,
    pub oauth_state: String,
}
