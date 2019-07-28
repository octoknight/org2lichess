use postgres;

pub type ErrorBox = Box<dyn std::error::Error>;
pub type Db = std::sync::RwLock<postgres::Client>;
