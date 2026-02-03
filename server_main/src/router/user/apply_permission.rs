use axum::{
    Router,
    extract::{Query, State},
    http::HeaderMap,
    routing::{get, post},
};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::user as user_entity;
use interface_types::proto::user::{
    ApplyPermission as ProtoApplyPermission, ApplyPermissionResponse,
};
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use user_auth::db_exchange::{ExchangeError, jwt_secret_from_env, now_timestamp, token2user};
use user_auth::user_auth::UserPermissionLevel;

use hmac::{Hmac, digest::KeyInit};
use jwt::{SignWithKey, VerifyWithKey};
use sha2::Sha256;

use crate::AppState;

const APPLY_PERMISSION_EXPIRE_SECONDS: u64 = 180;

#[derive(Debug, Deserialize)]
struct ApplyPermissionQuery {
    /// 申请权限类型：2 或 3
    apply_type: i32,
}

#[derive(Debug, Deserialize)]
struct ApplyPermissionCodeQuery {
    /// 校验码
    code: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApplyPermissionClaims {
    exp: u64,
    apply_type: i32,
}

/// 创建 apply_permission 路由
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/apply_permission", get(generate_apply_code))
        .route("/apply_permission", post(apply_permission))
}

/// GET /api/user/apply_permission?apply_type=2|3
/// 仅 Admin 权限可以获取校验码
async fn generate_apply_code(
    State(_state): State<AppState>,
    Query(params): Query<ApplyPermissionQuery>,
    headers: HeaderMap,
) -> Protobuf<ApplyPermissionResponse> {
    // 1) 从 Header 提取 token
    let token: &str = match headers.get("Authorization") {
        Some(t) => match t.to_str() {
            Ok(s) => s,
            Err(_) => {
                return Protobuf(ApplyPermissionResponse {
                    apply_permission: None,
                    code: 401,
                    message: "Invalid token format".to_string(),
                });
            }
        },
        None => {
            return Protobuf(ApplyPermissionResponse {
                apply_permission: None,
                code: 401,
                message: "Missing token".to_string(),
            });
        }
    };

    // 2) 解析 token，获取用户信息
    let auth_user = match token2user(&token) {
        Ok(u) => u,
        Err(err) => {
            let msg = match err {
                ExchangeError::InvalidToken => "Invalid token".to_string(),
                ExchangeError::TokenExpired => "Token expired".to_string(),
                ExchangeError::TokenGenerationError(e) | ExchangeError::OtherError(e) => e,
            };
            return Protobuf(ApplyPermissionResponse {
                apply_permission: None,
                code: 401,
                message: msg,
            });
        }
    };

    // 3) 权限校验：只有 Admin (permission=3) 才能获取校验码
    let user_permission = auth_user.permission.unwrap_or(0);
    if user_permission != UserPermissionLevel::Admin.level() {
        return Protobuf(ApplyPermissionResponse {
            apply_permission: None,
            code: 403,
            message: "Permission denied: Only Admin can generate apply code".to_string(),
        });
    }

    // 4) 校验 apply_type
    if params.apply_type != UserPermissionLevel::Provider.level()
        && params.apply_type != UserPermissionLevel::Admin.level()
    {
        return Protobuf(ApplyPermissionResponse {
            apply_permission: None,
            code: 400,
            message: "Invalid apply_type: must be 2 or 3".to_string(),
        });
    }

    // 5) 生成 3 分钟有效期的校验码（JWT）
    let expire_time = now_timestamp() + APPLY_PERMISSION_EXPIRE_SECONDS;
    let claims = ApplyPermissionClaims {
        exp: expire_time,
        apply_type: params.apply_type,
    };

    let secret = match jwt_secret_from_env() {
        Ok(s) => s,
        Err(err) => {
            let msg = match err {
                ExchangeError::OtherError(e) => e,
                _ => "Failed to get jwt secret".to_string(),
            };
            return Protobuf(ApplyPermissionResponse {
                apply_permission: None,
                code: 500,
                message: msg,
            });
        }
    };
    let key: Hmac<Sha256> = match Hmac::new_from_slice(secret.as_bytes()) {
        Ok(k) => k,
        Err(err) => {
            return Protobuf(ApplyPermissionResponse {
                apply_permission: None,
                code: 500,
                message: format!("Failed to init jwt key: {}", err),
            });
        }
    };

    let code = match claims.sign_with_key(&key) {
        Ok(c) => c,
        Err(err) => {
            return Protobuf(ApplyPermissionResponse {
                apply_permission: None,
                code: 500,
                message: format!("Failed to generate apply code: {}", err),
            });
        }
    };

    Protobuf(ApplyPermissionResponse {
        apply_permission: Some(ProtoApplyPermission {
            code,
            apply_type: params.apply_type,
            expire_time: expire_time as i64,
        }),
        code: 200,
        message: "Generate apply code success".to_string(),
    })
}

/// POST /api/user/apply_permission?code=xxx
/// 所有权限均可申请，通过校验码更新自身权限
async fn apply_permission(
    State(state): State<AppState>,
    Query(params): Query<ApplyPermissionCodeQuery>,
    headers: HeaderMap,
) -> Protobuf<ApplyPermissionResponse> {
    // 1) 从 Header 提取 token
    let token: &str = match headers.get("Authorization") {
        Some(t) => match t.to_str() {
            Ok(s) => s,
            Err(_) => {
                return Protobuf(ApplyPermissionResponse {
                    apply_permission: None,
                    code: 401,
                    message: "Invalid token format".to_string(),
                });
            }
        },
        None => {
            return Protobuf(ApplyPermissionResponse {
                apply_permission: None,
                code: 401,
                message: "Missing token".to_string(),
            });
        }
    };

    // 2) 解析 token，获取用户信息
    let auth_user = match token2user(&token) {
        Ok(u) => u,
        Err(err) => {
            let msg = match err {
                ExchangeError::InvalidToken => "Invalid token".to_string(),
                ExchangeError::TokenExpired => "Token expired".to_string(),
                ExchangeError::TokenGenerationError(e) | ExchangeError::OtherError(e) => e,
            };
            return Protobuf(ApplyPermissionResponse {
                apply_permission: None,
                code: 401,
                message: msg,
            });
        }
    };

    // 3) 解码校验码
    let secret = match jwt_secret_from_env() {
        Ok(s) => s,
        Err(err) => {
            let msg = match err {
                ExchangeError::OtherError(e) => e,
                _ => "Failed to get jwt secret".to_string(),
            };
            return Protobuf(ApplyPermissionResponse {
                apply_permission: None,
                code: 500,
                message: msg,
            });
        }
    };
    let key: Hmac<Sha256> = match Hmac::new_from_slice(secret.as_bytes()) {
        Ok(k) => k,
        Err(err) => {
            return Protobuf(ApplyPermissionResponse {
                apply_permission: None,
                code: 500,
                message: format!("Failed to init jwt key: {}", err),
            });
        }
    };

    let claims: ApplyPermissionClaims = match params.code.verify_with_key(&key) {
        Ok(c) => c,
        Err(_) => {
            return Protobuf(ApplyPermissionResponse {
                apply_permission: None,
                code: 400,
                message: "Invalid apply code".to_string(),
            });
        }
    };

    if claims.exp < now_timestamp() {
        return Protobuf(ApplyPermissionResponse {
            apply_permission: None,
            code: 400,
            message: "Apply code expired".to_string(),
        });
    }

    if claims.apply_type != UserPermissionLevel::Provider.level()
        && claims.apply_type != UserPermissionLevel::Admin.level()
    {
        return Protobuf(ApplyPermissionResponse {
            apply_permission: None,
            code: 400,
            message: "Invalid apply_type in code".to_string(),
        });
    }

    // 4) 更新用户权限
    let db = state.database.clone();
    let target = match user_entity::Entity::find()
        .filter(user_entity::Column::OpenId.eq(auth_user.open_id.clone()))
        .one(db.as_ref())
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            return Protobuf(ApplyPermissionResponse {
                apply_permission: None,
                code: 404,
                message: "User not found".to_string(),
            });
        }
        Err(err) => {
            return Protobuf(ApplyPermissionResponse {
                apply_permission: None,
                code: 500,
                message: format!("Database error: {}", err),
            });
        }
    };

    let mut active: user_entity::ActiveModel = target.clone().into();
    active.permission = Set(Some(claims.apply_type));
    active.id = ActiveValue::Unchanged(target.id);
    active.open_id = Set(target.open_id.clone());

    if let Err(err) = active.update(db.as_ref()).await {
        return Protobuf(ApplyPermissionResponse {
            apply_permission: None,
            code: 500,
            message: format!("Failed to update permission: {}", err),
        });
    }

    Protobuf(ApplyPermissionResponse {
        apply_permission: Some(ProtoApplyPermission {
            code: params.code,
            apply_type: claims.apply_type,
            expire_time: claims.exp as i64,
        }),
        code: 200,
        message: "Apply permission success".to_string(),
    })
}
