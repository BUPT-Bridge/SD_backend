use axum::{
    Router,
    extract::{Query, State},
    http::HeaderMap,
    routing::{get, post, put},
};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::user as user_entity;
use interface_types::proto::user::{
    ChangePasswordRequest, ChangePasswordResponse, PasswordLoginRequest, PasswordLoginResponse,
    User as ProtoUser, UserResponse,
};
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use user_auth::db_exchange::{
    ExchangeError, User as AuthUser, md5_hash_password, token2user, user2token,
};
use user_auth::wx_auth::*;

use crate::AppState;

#[derive(Deserialize)]
struct LoginQuery {
    /// For demo we accept `openid` query param; typically this would be `js_code`.
    js_code: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", get(login))
        .route("/login", post(password_login))
        .route("/login", put(change_password))
}

// 微信登录 - 原有的 GET /login
async fn login(
    State(state): State<AppState>,
    Query(query): Query<LoginQuery>,
) -> Protobuf<UserResponse> {
    // Use wx_auth to resolve the provided token/code into an openid.
    let wx_result = wx_auth_session_to_json(&query.js_code).await;

    let openid = match wx_result {
        Ok(resp) => match resp.openid {
            Some(oid) => oid,
            None => {
                return Protobuf(UserResponse {
                    user: None,
                    code: 400,
                    message: "WeiXin auth did not return an openid".to_string(),
                });
            }
        },
        Err(err) => {
            return Protobuf(UserResponse {
                user: None,
                code: 500,
                message: format!("failed to resolve openid: {:?}", err),
            });
        }
    };

    // Query user in the database.
    let user = match query_user_in_db(&state, &openid).await {
        Ok(u) => u,
        Err(err) => {
            return Protobuf(UserResponse {
                user: None,
                code: 500,
                message: err,
            });
        }
    };

    Protobuf(UserResponse {
        user: Some(user),
        code: 200,
        message: "login success".to_string(),
    })
}

// 密码登录 - POST /login
async fn password_login(
    State(state): State<AppState>,
    Protobuf(payload): Protobuf<PasswordLoginRequest>,
) -> Protobuf<PasswordLoginResponse> {
    let db = state.database.clone();
    let phone_number = payload.phone_number;
    let password = payload.password;
    let x_api_key = std::env::var("SERVER_X_API_KEY").unwrap_or_default();

    // Hash the password using MD5
    let hashed_password = md5_hash_password(&password);

    // Find user by phone number
    let user_model = match user_entity::Entity::find()
        .filter(user_entity::Column::PhoneNumber.eq(&phone_number))
        .one(db.as_ref())
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            return Protobuf(PasswordLoginResponse {
                user: None,
                code: 404,
                message: "User not found".to_string(),
            });
        }
        Err(err) => {
            return Protobuf(PasswordLoginResponse {
                user: None,
                code: 500,
                message: format!("Database error: {:?}", err),
            });
        }
    };

    // Check if the password matches
    let stored_password = match &user_model.password {
        Some(pwd) => pwd,
        None => {
            return Protobuf(PasswordLoginResponse {
                user: None,
                code: 401,
                message: "Password not set for this user".to_string(),
            });
        }
    };

    if stored_password != &hashed_password {
        return Protobuf(PasswordLoginResponse {
            user: None,
            code: 401,
            message: "Invalid password".to_string(),
        });
    }

    // Generate token
    let auth_user = AuthUser {
        open_id: user_model.open_id.clone(),
        nickname: user_model.nickname.clone(),
        name: user_model.name.clone(),
        phone_number: user_model.phone_number.clone(),
        address: user_model.address.clone(),
        is_important: user_model.is_important,
        avatar: user_model.avatar.clone(),
        permission: user_model.permission,
        password: user_model.password.clone(),
    };

    let jwt_token = match user2token(&auth_user) {
        Ok(token) => token,
        Err(err) => {
            return Protobuf(PasswordLoginResponse {
                user: None,
                code: 500,
                message: format!("Failed to generate token: {:?}", err),
            });
        }
    };

    let proto_user = ProtoUser {
        token: Some(jwt_token),
        nickname: user_model.nickname,
        name: user_model.name,
        phone_number: user_model.phone_number,
        address: user_model.address,
        is_important: user_model.is_important.map(|b| b.to_string()),
        avatar: user_model.avatar,
        permission: user_model.permission.map(|p| p.to_string()),
        x_api_key: Some(x_api_key),
    };

    Protobuf(PasswordLoginResponse {
        user: Some(proto_user),
        code: 200,
        message: "login success".to_string(),
    })
}

// 修改密码 - PUT /login
async fn change_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Protobuf(payload): Protobuf<ChangePasswordRequest>,
) -> Protobuf<ChangePasswordResponse> {
    // Parse and validate token
    let token: &str = match headers.get("Authorization") {
        Some(t) => match t.to_str() {
            Ok(s) => s,
            Err(_) => {
                return Protobuf(ChangePasswordResponse {
                    code: 401,
                    message: "Invalid authorization header".to_string(),
                });
            }
        },
        None => {
            return Protobuf(ChangePasswordResponse {
                code: 401,
                message: "Missing authorization token".to_string(),
            });
        }
    };

    let auth_user: AuthUser = match token2user(token) {
        Ok(u) => u,
        Err(err) => {
            let msg = match err {
                ExchangeError::InvalidToken => "Invalid token".to_string(),
                ExchangeError::TokenExpired => "Token expired".to_string(),
                ExchangeError::TokenGenerationError(e) | ExchangeError::OtherError(e) => e,
            };
            return Protobuf(ChangePasswordResponse {
                code: 401,
                message: msg,
            });
        }
    };

    // Hash the new password using MD5
    let new_hashed_password = md5_hash_password(&payload.password);

    // Update the user's password in the database
    let db = state.database.clone();
    let user_model = match user_entity::Entity::find()
        .filter(user_entity::Column::OpenId.eq(&auth_user.open_id))
        .one(db.as_ref())
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            return Protobuf(ChangePasswordResponse {
                code: 404,
                message: "User not found".to_string(),
            });
        }
        Err(err) => {
            return Protobuf(ChangePasswordResponse {
                code: 500,
                message: format!("Database error: {:?}", err),
            });
        }
    };

    // Update the password
    let mut active_model: user_entity::ActiveModel = user_model.into();
    active_model.password = Set(Some(new_hashed_password));
    active_model.id = ActiveValue::Unchanged(active_model.id.clone().unwrap());

    match active_model.update(db.as_ref()).await {
        Ok(_) => (),
        Err(err) => {
            return Protobuf(ChangePasswordResponse {
                code: 500,
                message: format!("Failed to update password: {:?}", err),
            });
        }
    }

    Protobuf(ChangePasswordResponse {
        code: 200,
        message: "Password changed successfully".to_string(),
    })
}

async fn query_user_in_db(state: &AppState, openid: &str) -> Result<ProtoUser, String> {
    let db = state.database.clone();
    let x_api_key = std::env::var("SERVER_X_API_KEY").map_err(|e| e.to_string())?;

    let user_queryed_result = user_entity::Entity::find()
        .filter(user_entity::Column::OpenId.eq(openid))
        .one(db.as_ref())
        .await
        .unwrap();

    if user_queryed_result == None {
        return Err("User not found".to_string());
    }

    let model = user_queryed_result.unwrap();
    let user = AuthUser {
        open_id: model.open_id.clone(),
        nickname: model.nickname.clone(),
        name: model.name.clone(),
        phone_number: model.phone_number.clone(),
        address: model.address.clone(),
        is_important: model.is_important,
        avatar: model.avatar.clone(),
        permission: model.permission,
        password: model.password.clone(),
    };
    let jwt_token = user2token(&user).unwrap();

    Ok(ProtoUser {
        token: Some(jwt_token),
        nickname: model.nickname,
        name: model.name,
        phone_number: model.phone_number,
        address: model.address,
        is_important: model.is_important.map(|b| b.to_string()),
        avatar: model.avatar,
        permission: model.permission.map(|p| p.to_string()),
        x_api_key: Some(x_api_key),
    })
}
