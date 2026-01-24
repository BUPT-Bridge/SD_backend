use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

pub mod token2user;
pub mod user2token;

pub const EXPIRATION_TIME: u64 = 36000; // 10 hour

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub open_id: String,
    pub nickname: Option<String>,
    pub avatar: Option<String>,
    pub permission: Option<i32>,
    pub name: Option<String>,
    pub phone_number: Option<String>,
    pub address: Option<String>,
    pub is_important: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: u64,
    pub user: User,
}

#[derive(Debug)]
pub enum ExchangeError {
    TokenGenerationError(String),
    InvalidToken,
    TokenExpired,
    OtherError(String),
}

pub fn jwt_secret_from_env() -> Result<String, ExchangeError> {
    std::env::var("SERVER_JWT_SECRET").map_err(|e| ExchangeError::OtherError(e.to_string()))
}

pub fn now_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn expiration_timestamp() -> u64 {
    now_timestamp() + EXPIRATION_TIME
}
