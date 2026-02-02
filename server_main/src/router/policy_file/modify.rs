use axum::{
    Router,
    extract::{Query, State},
    http::HeaderMap,
    routing::put,
};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::policy_file as policy_file_entity;
use interface_types::proto::policy_file::{
    PolicyFile as ProtoPolicyFile, PolicyFileRequest, PolicyFileResponse,
};
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use user_auth::db_exchange::{ExchangeError, token2user};
use user_auth::user_auth::UserPermissionLevel;

use crate::AppState;

/// 创建 policy_file 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", put(modify_policy_file))
}

/// 查询参数
#[derive(Debug, Deserialize)]
struct PolicyFileParams {
    /// 政策文件的 ID
    id: i32,
}

/// PUT /api/policy_file?id=xxx - 修改政策文件（仅 Admin 权限可以访问）
async fn modify_policy_file(
    State(state): State<AppState>,
    Query(params): Query<PolicyFileParams>,
    headers: HeaderMap,
    Protobuf(payload): Protobuf<PolicyFileRequest>,
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

    // 3) 权限校验：只有 Admin (permission=3) 才能修改政策文件
    let user_permission = auth_user.permission.unwrap_or(0);
    if user_permission != UserPermissionLevel::Admin.level() {
        return Protobuf(PolicyFileResponse {
            policy_files: vec![],
            code: 403,
            message: "Permission denied: Only Admin can modify policy file".to_string(),
        });
    }

    // 4) 查找目标政策文件
    let db = state.database.clone();
    let target = match policy_file_entity::Entity::find()
        .filter(policy_file_entity::Column::Id.eq(params.id))
        .one(db.as_ref())
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            return Protobuf(PolicyFileResponse {
                policy_files: vec![],
                code: 404,
                message: "Policy file not found".to_string(),
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

    // 5) 应用部分更新：payload 中非空的字段覆盖，其他保持不变
    let mut active: policy_file_entity::ActiveModel = target.clone().into();

    if !payload.title.is_empty() {
        active.title = Set(Some(payload.title));
    }
    if !payload.r#type.is_empty() {
        active.r#type = Set(Some(payload.r#type));
    }
    if !payload.index.is_empty() {
        active.index = Set(Some(payload.index));
    }

    // 保留原主键和创建时间
    active.id = ActiveValue::Unchanged(target.id);
    active.create_time = ActiveValue::Unchanged(target.create_time);

    // 6) 更新数据库
    let target_updated = match active.update(db.as_ref()).await {
        Ok(m) => m,
        Err(err) => {
            return Protobuf(PolicyFileResponse {
                policy_files: vec![],
                code: 500,
                message: format!("Failed to update policy file: {}", err),
            });
        }
    };

    // 7) 返回更新后的政策文件
    Protobuf(PolicyFileResponse {
        policy_files: vec![ProtoPolicyFile {
            id: target_updated.id,
            title: target_updated.title.unwrap_or_default(),
            r#type: target_updated.r#type.unwrap_or_default(),
            index: target_updated.index.unwrap_or_default(),
            create_time: target_updated.create_time.and_utc().timestamp(),
        }],
        code: 200,
        message: "Modify policy file success".to_string(),
    })
}
