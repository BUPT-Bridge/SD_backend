use axum::{Router, extract::State, http::HeaderMap, routing::post};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::policy_file as policy_file_entity;
use interface_types::proto::policy_file::{
    PolicyFile as ProtoPolicyFile, PolicyFileRequest, PolicyFileResponse,
};
use sea_orm::{ActiveModelTrait, Set};
use user_auth::db_exchange::{ExchangeError, token2user};
use user_auth::user_auth::UserPermissionLevel;

use crate::AppState;

/// 创建 policy_file 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", post(create_policy_file))
}

/// POST /api/policy_file - 创建政策文件（仅 Admin 权限可以访问）
async fn create_policy_file(
    State(state): State<AppState>,
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

    // 3) 权限校验：只有 Admin (permission=3) 才能创建政策文件
    let user_permission = auth_user.permission.unwrap_or(0);
    if user_permission != UserPermissionLevel::Admin.level() {
        return Protobuf(PolicyFileResponse {
            policy_files: vec![],
            code: 403,
            message: "Permission denied: Only Admin can create policy file".to_string(),
        });
    }

    // 4) 创建新的政策文件（id 和 create_time 由数据库自动处理）
    let new_policy_file = policy_file_entity::ActiveModel {
        id: Default::default(), // auto increment
        title: Set(if payload.title.is_empty() {
            None
        } else {
            Some(payload.title.clone())
        }),
        r#type: Set(if payload.r#type.is_empty() {
            None
        } else {
            Some(payload.r#type.clone())
        }),
        index: Set(if payload.index.is_empty() {
            None
        } else {
            Some(payload.index.clone())
        }),
        create_time: Default::default(), // auto set by database
    };

    let db = state.database.clone();
    let inserted = match new_policy_file.insert(db.as_ref()).await {
        Ok(m) => m,
        Err(err) => {
            return Protobuf(PolicyFileResponse {
                policy_files: vec![],
                code: 500,
                message: format!("Failed to create policy file: {}", err),
            });
        }
    };

    // 5) 返回创建的政策文件
    Protobuf(PolicyFileResponse {
        policy_files: vec![ProtoPolicyFile {
            id: inserted.id,
            title: inserted.title.unwrap_or_default(),
            r#type: inserted.r#type.unwrap_or_default(),
            index: inserted.index.unwrap_or_default(),
            create_time: inserted.create_time.and_utc().timestamp(),
        }],
        code: 200,
        message: "Create policy file success".to_string(),
    })
}
