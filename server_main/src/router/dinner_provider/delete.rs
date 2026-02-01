use axum::{
    Router,
    extract::{Query, State},
    http::HeaderMap,
    routing::delete,
};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::dinner_provider as dinner_provider_entity;
use interface_types::proto::dinner_provider::DinnerProviderResponse;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;
use user_auth::db_exchange::{ExchangeError, token2user};
use user_auth::user_auth::UserPermissionLevel;

use crate::AppState;

/// 创建 dinner_provider 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", delete(delete_dinner_provider))
}

/// 查询参数
#[derive(Debug, Deserialize)]
struct DinnerProviderParams {
    /// 供餐点的 ID
    id: i32,
}

/// DELETE /api/dinner_provider?id=xxx - 删除供餐点（仅 Admin 权限可以访问）
async fn delete_dinner_provider(
    State(state): State<AppState>,
    Query(params): Query<DinnerProviderParams>,
    headers: HeaderMap,
) -> Protobuf<DinnerProviderResponse> {
    // 1) 从 Header 提取 token
    let token: &str = match headers.get("Authorization") {
        Some(t) => match t.to_str() {
            Ok(s) => s,
            Err(_) => {
                return Protobuf(DinnerProviderResponse {
                    dinner_providers: vec![],
                    code: 401,
                    message: "Invalid token format".to_string(),
                });
            }
        },
        None => {
            return Protobuf(DinnerProviderResponse {
                dinner_providers: vec![],
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
            return Protobuf(DinnerProviderResponse {
                dinner_providers: vec![],
                code: 401,
                message: msg,
            });
        }
    };

    // 3) 权限校验：只有 Admin (permission=3) 才能删除供餐点
    let user_permission = auth_user.permission.unwrap_or(0);
    if user_permission != UserPermissionLevel::Admin.level() {
        return Protobuf(DinnerProviderResponse {
            dinner_providers: vec![],
            code: 403,
            message: "Permission denied: Only Admin can delete dinner provider".to_string(),
        });
    }

    // 4) 查找要删除的供餐点
    let db = state.database.clone();
    let dinner_provider_to_delete = match dinner_provider_entity::Entity::find()
        .filter(dinner_provider_entity::Column::Id.eq(params.id))
        .one(db.as_ref())
        .await
    {
        Ok(Some(s)) => s,
        Ok(None) => {
            return Protobuf(DinnerProviderResponse {
                dinner_providers: vec![],
                code: 404,
                message: format!("Dinner provider with id '{}' not found", params.id),
            });
        }
        Err(err) => {
            return Protobuf(DinnerProviderResponse {
                dinner_providers: vec![],
                code: 500,
                message: format!("Database error: {}", err),
            });
        }
    };

    // 5) 执行删除
    match dinner_provider_entity::Entity::delete_by_id(dinner_provider_to_delete.id)
        .exec(db.as_ref())
        .await
    {
        Ok(_) => {}
        Err(err) => {
            return Protobuf(DinnerProviderResponse {
                dinner_providers: vec![],
                code: 500,
                message: format!("Failed to delete dinner provider: {}", err),
            });
        }
    };

    // 6) 返回成功响应
    Protobuf(DinnerProviderResponse {
        dinner_providers: vec![],
        code: 200,
        message: "Delete dinner provider success".to_string(),
    })
}

