use std::env;
use std::sync::{Mutex, OnceLock};

use hmac::{Hmac, digest::KeyInit};
use jwt::SignWithKey;
use sha2::Sha256;
use user_auth::db_exchange::{Claims, ExchangeError, User, token2user::token2user};

static ENV_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

fn env_lock() -> &'static Mutex<()> {
    ENV_MUTEX.get_or_init(|| Mutex::new(()))
}

fn with_secret<F: FnOnce() -> R, R>(secret: &str, f: F) -> R {
    let _guard = env_lock().lock().expect("env mutex poisoned");
    // std::env::set_var is unsafe in this environment; confine it here under a lock.
    unsafe {
        env::set_var("SERVER_JWT_SECRET", secret);
    }
    f()
}

fn make_user() -> User {
    User {
        open_id: "openid-abc".to_string(),
        nickname: Some("bob".to_string()),
        avatar: Some("avatar.png".to_string()),
        permission: Some(1),
        name: Some("Bob".to_string()),
        phone_number: Some("123".to_string()),
        address: Some("Somewhere".to_string()),
        is_important: Some(false),
    }
}

fn sign_with_secret(claims: &Claims, secret: &str) -> String {
    let key: Hmac<Sha256> =
        Hmac::new_from_slice(secret.as_bytes()).expect("failed to build key for test");
    claims
        .sign_with_key(&key)
        .expect("failed to sign claims for test")
}

#[test]
fn rejects_invalid_token_format() {
    with_secret("secret", || {
        let err = token2user("not-a-jwt").unwrap_err();
        match err {
            ExchangeError::InvalidToken => {}
            other => panic!("expected InvalidToken, got {:?}", other),
        }
    });
}

#[test]
fn rejects_wrong_signature() {
    let user = make_user();
    // Sign with secret A, verify with secret B.
    let claims = Claims {
        sub: user.open_id.clone(),
        exp: u64::MAX, // far in the future
        user,
    };
    let token = sign_with_secret(&claims, "secret-a");

    with_secret("secret-b", || {
        let err = token2user(&token).unwrap_err();
        match err {
            ExchangeError::InvalidToken => {}
            other => panic!("expected InvalidToken, got {:?}", other),
        }
    });
}

#[test]
fn rejects_expired_token() {
    let user = make_user();
    let claims = Claims {
        sub: user.open_id.clone(),
        exp: 1, // definitely in the past
        user,
    };
    let token = sign_with_secret(&claims, "secret");

    with_secret("secret", || {
        let err = token2user(&token).unwrap_err();
        match err {
            ExchangeError::TokenExpired => {}
            other => panic!("expected TokenExpired, got {:?}", other),
        }
    });
}
