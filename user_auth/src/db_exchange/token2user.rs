use super::{Claims, ExchangeError, User, jwt_secret_from_env, now_timestamp};
use hmac::{Hmac, digest::KeyInit};
use jwt::VerifyWithKey;
use sha2::Sha256;

/// Parse a JWT string into a `User`, validating signature and expiration.
pub fn token2user(token: &str) -> Result<User, ExchangeError> {
    let secret = jwt_secret_from_env()?;
    let key: Hmac<Sha256> = Hmac::new_from_slice(secret.as_bytes())
        .map_err(|e| ExchangeError::OtherError(e.to_string()))?;

    let claims: Claims = token
        .verify_with_key(&key)
        .map_err(|_| ExchangeError::InvalidToken)?;

    if claims.exp < now_timestamp() {
        return Err(ExchangeError::TokenExpired);
    }

    Ok(claims.user)
}
