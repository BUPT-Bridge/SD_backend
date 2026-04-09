use axum::{Router, extract::State, http::HeaderMap, routing::put};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::user as user_entity;
use interface_types::proto::user::{User as ProtoUser, UserRequest, UserResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter, Set};
use user_auth::db_exchange::{
    ExchangeError, User as AuthUser, md5_hash_password, token2user, user2token,
};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/modify", put(modify))
}

async fn modify(
    State(state): State<AppState>,
    headers: HeaderMap,
    Protobuf(payload): Protobuf<UserRequest>,
) -> Protobuf<UserResponse> {
    // 1) 解析 token，拿到当前用户信息
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

    // 2) 查询当前用户
    let db = state.database.clone();
    let user_model = match user_entity::Entity::find()
        .filter(user_entity::Column::OpenId.eq(&auth_user.open_id))
        .one(db.as_ref())
        .await
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
                message: format!("Database error: {:?}", err),
            });
        }
    };

    // 3) 应用部分更新：payload 中非 None 的字段覆盖，其他保持不变
    let mut active: user_entity::ActiveModel = user_model.clone().into();

    if let Some(v) = payload.nickname.clone() {
        active.nickname = Set(Some(v));
    }
    if let Some(v) = payload.name.clone() {
        active.name = Set(Some(v));
    }
    if let Some(v) = payload.phone_number.clone() {
        active.phone_number = Set(Some(v));
    }
    if let Some(v) = payload.address.clone() {
        active.address = Set(Some(v));
    }
    if let Some(v) = payload.avatar.clone() {
        active.avatar = Set(Some(v));
    }
    if let Some(v) = payload.permission.clone() {
        if let Ok(p) = v.parse::<i32>() {
            active.permission = Set(Some(p));
        }
    }
    if let Some(v) = payload.is_important.clone() {
        if let Ok(b) = v.parse::<bool>() {
            active.is_important = Set(Some(b));
        }
    }

    // 处理密码字段：如果提供了新密码，使用 MD5 加密
    if let Some(v) = payload.password.clone() {
        if !v.is_empty() {
            let hashed_password = md5_hash_password(&v);
            active.password = Set(Some(hashed_password));
        }
    }

    // 确保 openid 和 id 不变
    active.open_id = ActiveValue::Unchanged(active.open_id.unwrap());
    active.id = ActiveValue::Unchanged(active.id.unwrap());

    // 4) 更新数据库
    let updated_user = match active.update(db.as_ref()).await {
        Ok(m) => m,
        Err(err) => {
            return Protobuf(UserResponse {
                user: None,
                code: 500,
                message: format!("Failed to update user: {:?}", err),
            });
        }
    };

    // 5) 重新生成 token
    let auth_user = AuthUser {
        open_id: updated_user.open_id.clone(),
        nickname: updated_user.nickname.clone(),
        avatar: updated_user.avatar.clone(),
        permission: updated_user.permission,
        name: updated_user.name.clone(),
        phone_number: updated_user.phone_number.clone(),
        address: updated_user.address.clone(),
        is_important: updated_user.is_important,
        password: updated_user.password.clone(),
    };
    let new_token = user2token(&auth_user).unwrap_or_default();

    // 6) 构造返回
    Protobuf(UserResponse {
        user: Some(ProtoUser {
            token: Some(new_token),
            nickname: updated_user.nickname,
            name: updated_user.name,
            phone_number: updated_user.phone_number,
            address: updated_user.address,
            is_important: updated_user.is_important.map(|b| b.to_string()),
            avatar: updated_user.avatar,
            permission: updated_user.permission.map(|p| p.to_string()),
            x_api_key: None,
        }),
        code: 200,
        message: "modify success".to_string(),
    })
}
