use std::env;

use user_auth::db_exchange::{User, token2user::token2user, user2token::user2token};

fn set_secret() {
    // std::env::set_var is unsafe in this environment; confine it here.
    unsafe {
        env::set_var("SERVER_JWT_SECRET", "test-secret");
    }
}

fn make_user() -> User {
    User {
        id: 42,
        open_id: "openid-123".to_string(),
        nickname: Some("alice".to_string()),
        avatar: Some("https://example.com/avatar.png".to_string()),
        permission: Some(2),
        name: Some("Alice".to_string()),
        phone_number: Some("1234567890".to_string()),
        address: Some("Wonderland".to_string()),
        is_important: Some(true),
    }
}

#[test]
fn generates_non_empty_token() {
    set_secret();
    let user = make_user();
    let token = user2token(&user).expect("token generation should succeed");
    assert!(
        !token.trim().is_empty(),
        "generated token should not be empty"
    );
}

#[test]
fn roundtrip_user_to_token_and_back() {
    set_secret();
    let user = make_user();

    let token = user2token(&user).expect("token generation should succeed");
    let parsed = token2user(&token).expect("token should parse back to user");

    assert_eq!(parsed.id, user.id);
    assert_eq!(parsed.open_id, user.open_id);
    assert_eq!(parsed.nickname, user.nickname);
    assert_eq!(parsed.avatar, user.avatar);
    assert_eq!(parsed.permission, user.permission);
    assert_eq!(parsed.name, user.name);
    assert_eq!(parsed.phone_number, user.phone_number);
    assert_eq!(parsed.address, user.address);
    assert_eq!(parsed.is_important, user.is_important);
}
