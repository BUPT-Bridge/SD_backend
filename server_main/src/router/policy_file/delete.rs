use axum::{
    Router,
    extract::{Query, State},
    http::HeaderMap,
    routing::delete,
};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::policy_file as policy_file_entity;
use interface_types::proto::policy_file::PolicyFileResponse;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;
use user_auth::db_exchange::{ExchangeError, token2user};
use user_auth::user_auth::UserPermissionLevel;

use crate::AppState;

/// 创建 policy_file 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", delete(delete_policy_file))
}

/// 查询参数
#[derive(Debug, Deserialize)]
struct PolicyFileParams {
    /// 政策文件的 ID
    id: i32,
}

/// DELETE /api/policy_file?id=xxx - 删除政策文件（仅 Admin 权限可以访问）
async fn delete_policy_file(
    State(state): State<AppState>,
    Query(params): Query<PolicyFileParams>,
    headers: HeaderMap,
) -> Protobuf<PolicyFileResponse> {
    // 1) 从 Header 提取 token
    let token: &str = match headers.get("Authorization") {
        Some(t) => match t.to_str() {
            Ok(s) => s,
            Err(_) => {
                return Protobuf(PolicyFileResponse {
                    policy_files: vec![],
                    code: 401,
                    message: "Invalid token format".to_string(),
                });
            }
        },
        None => {
            return Protobuf(PolicyFileResponse {
                policy_files: vec![],
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
            return Protobuf(PolicyFileResponse {
                policy_files: vec![],
                code: 401,
                message: msg,
            });
        }
    };

    // 3) 权限校验：只有 Admin (permission=3) 才能删除政策文件
    let user_permission = auth_user.permission.unwrap_or(0);
    if user_permission != UserPermissionLevel::Admin.level() {
        return Protobuf(PolicyFileResponse {
            policy_files: vec![],
            code: 403,
            message: "Permission denied: Only Admin can delete policy file".to_string(),
        });
    }

    // 4) 查找要删除的政策文件
    let db = state.database.clone();
    let policy_file_to_delete = match policy_file_entity::Entity::find()
        .filter(policy_file_entity::Column::Id.eq(params.id))
        .one(db.as_ref())
        .await
    {
        Ok(Some(s)) => s,
        Ok(None) => {
            return Protobuf(PolicyFileResponse {
                policy_files: vec![],
                code: 404,
                message: format!("Policy file with id '{}' not found", params.id),
            });
        }
        Err(err) => {
            return Protobuf(PolicyFileResponse {
                policy_files: vec![],
                code: 500,
                message: format!("Database error: {}", err),
            });
        }
    };

    // 5) 执行删除
    match policy_file_entity::Entity::delete_by_id(policy_file_to_delete.id)
        .exec(db.as_ref())
        .await
    {
        Ok(_) => {}
        Err(err) => {
            return Protobuf(PolicyFileResponse {
                policy_files: vec![],
                code: 500,
                message: format!("Failed to delete policy file: {}", err),
            });
        }
    };

    // 6) 返回成功响应
    Protobuf(PolicyFileResponse {
        policy_files: vec![],
        code: 200,
        message: "Delete policy file success".to_string(),
    })
}
