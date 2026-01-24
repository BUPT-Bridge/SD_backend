use axum::{Router, extract::State, http::HeaderMap, routing::put};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::user as user_entity;
use interface_types::proto::user::{User as ProtoUser, UserRequest, UserResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter, Set};
use user_auth::db_exchange::{ExchangeError, User as AuthUser, token2user, user2token};
use user_auth::user_auth::{UserPermissionAuthorizeResult, UserPermissionLevel, authorize_user};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/modify", put(modify))
}

async fn modify(
    State(state): State<AppState>,
    headers: HeaderMap,
    Protobuf(payload): Protobuf<UserRequest>,
) -> Protobuf<UserResponse> {
    // 1) 解析 token，拿到操作用户 openid 与权限
    let token: &str = if let Some(token) = headers.get("Authorization") {
        token.to_str().unwrap()
    } else {
        return Protobuf(UserResponse {
            user: None,
            code: 401,
            message: "Missing token".to_string(),
        });
    };
    // println!("token: {}", token);
    // println!("nickname: {}", payload.nickname.clone().unwrap_or_default());
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
    let actor_openid = auth_user.open_id.clone();

    // 2) 目标 openid（不传则修改自己）
    let target_openid = payload
        .target_openid
        .clone()
        .unwrap_or_else(|| actor_openid.clone());

    // 3) 查询操作用户与目标用户
    let db = state.database.clone();
    let actor = match user_entity::Entity::find()
        .filter(user_entity::Column::OpenId.eq(actor_openid.clone()))
        .one(db.as_ref())
        .await
        .map_err(|e| e.to_string())
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            return Protobuf(UserResponse {
                user: None,
                code: 404,
                message: "Operator not found".to_string(),
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

    let target = match user_entity::Entity::find()
        .filter(user_entity::Column::OpenId.eq(target_openid.clone()))
        .one(db.as_ref())
        .await
        .map_err(|e| e.to_string())
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            return Protobuf(UserResponse {
                user: None,
                code: 404,
                message: "target user not found".to_string(),
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

    // 4) 权限校验：自己或权限足够
    let actor_perm_code = actor.permission.unwrap_or(0);
    let target_perm_level: UserPermissionLevel = target.permission.unwrap_or(0).into();
    if actor_openid != target_openid {
        let auth_res = authorize_user(actor_perm_code, target_perm_level);
        if auth_res != UserPermissionAuthorizeResult::Authorized {
            return Protobuf(UserResponse {
                user: None,
                code: 403,
                message: "permission denied".to_string(),
            });
        }
    }

    // 5) 应用部分更新：payload 中非 None 的字段覆盖，其他保持不变
    let mut active: user_entity::ActiveModel = target.clone().into();
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
        // permission 字符串转 i32，失败则保持不变
        if let Ok(p) = v.parse::<i32>() {
            active.permission = Set(Some(p));
        }
    }
    if let Some(v) = payload.is_important.clone() {
        if let Ok(b) = v.parse::<bool>() {
            active.is_important = Set(Some(b));
        }
    }

    // 确保 openid 不变，并保留原主键
    active.open_id = Set(target_openid.clone());
    active.id = ActiveValue::Unchanged(target.id);

    // 6) 更新数据库
    let target_updated = match active.update(db.as_ref()).await.map_err(|e| e.to_string()) {
        Ok(m) => m,
        Err(err) => {
            return Protobuf(UserResponse {
                user: None,
                code: 500,
                message: err,
            });
        }
    };

    // 7) 重新生成 token（返回 actor 用户信息）
    let actor_model = if actor_openid == target_openid {
        target_updated
    } else {
        actor
    };

    let auth_user = AuthUser {
        open_id: actor_model.open_id.clone(),
        nickname: actor_model.nickname.clone(),
        avatar: actor_model.avatar.clone(),
        permission: actor_model.permission,
        name: actor_model.name.clone(),
        phone_number: actor_model.phone_number.clone(),
        address: actor_model.address.clone(),
        is_important: actor_model.is_important,
    };
    let new_token = user2token(&auth_user).unwrap_or_default();

    // 8) 构造返回（actor 信息）
    Protobuf(UserResponse {
        user: Some(ProtoUser {
            token: Some(new_token),
            nickname: actor_model.nickname,
            name: actor_model.name,
            phone_number: actor_model.phone_number,
            address: actor_model.address,
            is_important: actor_model.is_important.map(|b| b.to_string()),
            avatar: actor_model.avatar,
            permission: actor_model.permission.map(|p| p.to_string()),
        }),
        code: 200,
        message: "modify success".to_string(),
    })
}
