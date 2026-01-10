use std::env;

pub struct DatabaseConfig {
    uri: String,
}

impl DatabaseConfig {
    pub fn new(uri: String) -> Self {
        Self { uri }
    }

    pub fn from_env() -> Self {
        let uri = env::var("SERVER_DB_URI").expect("SERVER_DB_URI must be set");
        Self::new(uri)
    }

    pub fn uri(&self) -> String {
        self.uri.clone()
    }
}
