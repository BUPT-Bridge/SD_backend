use axum::{Router, extract::State, http::HeaderMap, routing::get};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::user as user_entity;
use interface_types::proto::user::{User as ProtoUser, UserResponse};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use user_auth::db_exchange::{ExchangeError, User as AuthUser, token2user};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/info", get(info))
}

async fn info(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Protobuf<UserResponse> {
    // 1) 解析 token，拿到用户 openid
    let token: &str = if let Some(token) = headers.get("Authorization") {
        token.to_str().unwrap()
    } else {
        return Protobuf(UserResponse {
            user: None,
            code: 401,
            message: "Missing token".to_string(),
        });
    };

    let auth_user: AuthUser = match token2user(&token) {
        Ok(u) => u,
        Err(err) => {
            let msg = match err {
                ExchangeError::InvalidToken => "Invalid token".to_string(),
                ExchangeError::TokenExpired => "Token expired".to_string(),
                ExchangeError::TokenGenerationError(e) | ExchangeError::OtherError(e) => e,
            };
            return Protobuf(UserResponse {
                user: None,
                code: 401,
                message: msg,
            });
        }
    };

    let openid = auth_user.open_id.clone();

    // 2) 查询用户信息
    let db = state.database.clone();
    let user = match user_entity::Entity::find()
        .filter(user_entity::Column::OpenId.eq(openid.clone()))
        .one(db.as_ref())
        .await
        .map_err(|e| e.to_string())
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            return Protobuf(UserResponse {
                user: None,
                code: 404,
                message: "User not found".to_string(),
            });
        }
        Err(err) => {
            return Protobuf(UserResponse {
                user: None,
                code: 500,
                message: err,
            });
        }
    };

    // 3) 构造返回
    Protobuf(UserResponse {
        user: Some(ProtoUser {
            token: Some(token.to_string()),
            nickname: user.nickname,
            name: user.name,
            phone_number: user.phone_number,
            address: user.address,
            is_important: user.is_important.map(|b| b.to_string()),
            avatar: user.avatar,
            permission: user.permission.map(|p| p.to_string()),
        }),
        code: 200,
        message: "success".to_string(),
    })
}
