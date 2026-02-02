use axum::{Router, extract::State, http::HeaderMap, routing::post};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::policy_type as policy_type_entity;
use interface_types::proto::policy_type::{
    PolicyType as ProtoPolicyType, PolicyTypeRequest, PolicyTypeResponse,
};
use sea_orm::{ActiveModelTrait, Set};
use user_auth::db_exchange::{ExchangeError, token2user};
use user_auth::user_auth::UserPermissionLevel;

use crate::AppState;

/// 创建 policy_type 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", post(create_policy_type))
}

/// POST /api/policy_type - 创建政策类型（仅 Admin 权限可以访问）
async fn create_policy_type(
    State(state): State<AppState>,
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

    // 3) 权限校验：只有 Admin (permission=3) 才能创建政策类型
    let user_permission = auth_user.permission.unwrap_or(0);
    if user_permission != UserPermissionLevel::Admin.level() {
        return Protobuf(PolicyTypeResponse {
            policy_types: vec![],
            code: 403,
            message: "Permission denied: Only Admin can create policy type".to_string(),
        });
    }

    // 4) 验证必填参数
    if payload.r#type.is_empty() {
        return Protobuf(PolicyTypeResponse {
            policy_types: vec![],
            code: 400,
            message: "Missing required parameter: type".to_string(),
        });
    }

    // 5) 创建新的政策类型
    let new_policy_type = policy_type_entity::ActiveModel {
        id: Default::default(), // auto increment
        r#type: Set(Some(payload.r#type.clone())),
    };

    let db = state.database.clone();
    let inserted = match new_policy_type.insert(db.as_ref()).await {
        Ok(m) => m,
        Err(err) => {
            return Protobuf(PolicyTypeResponse {
                policy_types: vec![],
                code: 500,
                message: format!("Failed to create policy type: {}", err),
            });
        }
    };

    // 6) 返回创建的政策类型
    Protobuf(PolicyTypeResponse {
        policy_types: vec![ProtoPolicyType {
            id: inserted.id,
            r#type: inserted.r#type.unwrap_or_default(),
        }],
        code: 200,
        message: "Create policy type success".to_string(),
    })
}
