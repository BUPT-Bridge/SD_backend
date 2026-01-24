use super::{Claims, ExchangeError, User, expiration_timestamp, jwt_secret_from_env};
use hmac::{Hmac, digest::KeyInit};
use jwt::SignWithKey;
use sha2::Sha256;

/// Generate a JWT for the given user with an embedded expiration timestamp.
pub fn user2token(user: &User) -> Result<String, ExchangeError> {
    let secret = jwt_secret_from_env()?;
    let key: Hmac<Sha256> = Hmac::new_from_slice(secret.as_bytes())
        .map_err(|e| ExchangeError::TokenGenerationError(e.to_string()))?;

    let claims = Claims {
        sub: user.open_id.clone(),
        exp: expiration_timestamp(),
        user: user.clone(),
    };

    claims
        .sign_with_key(&key)
        .map_err(|e| ExchangeError::TokenGenerationError(e.to_string()))
}
