use axum::{
    Router,
    extract::{Query, State},
    http::HeaderMap,
    routing::put,
};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::policy_type as policy_type_entity;
use interface_types::proto::policy_type::{
    PolicyType as ProtoPolicyType, PolicyTypeRequest, PolicyTypeResponse,
};
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use user_auth::db_exchange::{ExchangeError, token2user};
use user_auth::user_auth::UserPermissionLevel;

use crate::AppState;

/// 创建 policy_type 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", put(modify_policy_type))
}

/// 查询参数
#[derive(Debug, Deserialize)]
struct PolicyTypeParams {
    /// 政策类型的 ID
    id: i32,
}

/// PUT /api/policy_type?id=xxx - 修改政策类型（仅 Admin 权限可以访问）
async fn modify_policy_type(
    State(state): State<AppState>,
    Query(params): Query<PolicyTypeParams>,
    headers: HeaderMap,
    Protobuf(payload): Protobuf<PolicyTypeRequest>,
) -> Protobuf<PolicyTypeResponse> {
    // 1) 从 Header 提取 token
    let token: &str = match headers.get("Authorization") {
        Some(t) => match t.to_str() {
            Ok(s) => s,
            Err(_) => {
                return Protobuf(PolicyTypeResponse {
                    policy_types: vec![],
                    code: 401,
                    message: "Invalid token format".to_string(),
                });
            }
        },
        None => {
            return Protobuf(PolicyTypeResponse {
                policy_types: vec![],
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
            return Protobuf(PolicyTypeResponse {
                policy_types: vec![],
                code: 401,
                message: msg,
            });
        }
    };

    // 3) 权限校验：只有 Admin (permission=3) 才能修改政策类型
    let user_permission = auth_user.permission.unwrap_or(0);
    if user_permission != UserPermissionLevel::Admin.level() {
        return Protobuf(PolicyTypeResponse {
            policy_types: vec![],
            code: 403,
            message: "Permission denied: Only Admin can modify policy type".to_string(),
        });
    }

    // 4) 查找目标政策类型
    let db = state.database.clone();
    let target = match policy_type_entity::Entity::find()
        .filter(policy_type_entity::Column::Id.eq(params.id))
        .one(db.as_ref())
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            return Protobuf(PolicyTypeResponse {
                policy_types: vec![],
                code: 404,
                message: "Policy type not found".to_string(),
            });
        }
        Err(err) => {
            return Protobuf(PolicyTypeResponse {
                policy_types: vec![],
                code: 500,
                message: format!("Database error: {}", err),
            });
        }
    };

    // 5) 应用部分更新：payload 中非空的字段覆盖，其他保持不变
    let mut active: policy_type_entity::ActiveModel = target.clone().into();

    if !payload.r#type.is_empty() {
        active.r#type = Set(Some(payload.r#type));
    }

    // 保留原主键
    active.id = ActiveValue::Unchanged(target.id);

    // 6) 更新数据库
    let target_updated = match active.update(db.as_ref()).await {
        Ok(m) => m,
        Err(err) => {
            return Protobuf(PolicyTypeResponse {
                policy_types: vec![],
                code: 500,
                message: format!("Failed to update policy type: {}", err),
            });
        }
    };

    // 7) 返回更新后的政策类型
    Protobuf(PolicyTypeResponse {
        policy_types: vec![ProtoPolicyType {
            id: target_updated.id,
            r#type: target_updated.r#type.unwrap_or_default(),
        }],
        code: 200,
        message: "Modify policy type success".to_string(),
    })
}
