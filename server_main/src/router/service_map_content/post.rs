use axum::{Router, extract::State, http::HeaderMap, routing::post};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::service_map_content as service_map_content_entity;
use interface_types::proto::service_map_content::{
    ServiceMapContent as ProtoServiceMapContent, ServiceMapContentRequest,
    ServiceMapContentResponse,
};
use sea_orm::{ActiveModelTrait, Set, prelude::Json};
use user_auth::db_exchange::{ExchangeError, token2user};
use user_auth::user_auth::UserPermissionLevel;

use crate::AppState;

/// 创建 service_map_content 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", post(create_service_map_content))
}

/// POST /api/service_map_content - 创建服务地图内容（仅 Admin 权限可以访问）
async fn create_service_map_content(
    State(state): State<AppState>,
    headers: HeaderMap,
    Protobuf(payload): Protobuf<ServiceMapContentRequest>,
) -> Protobuf<ServiceMapContentResponse> {
    // 1) 从 Header 提取 token
    let token: &str = match headers.get("Authorization") {
        Some(t) => match t.to_str() {
            Ok(s) => s,
            Err(_) => {
                return Protobuf(ServiceMapContentResponse {
                    service_map_contents: vec![],
                    code: 401,
                    message: "Invalid token format".to_string(),
                });
            }
        },
        None => {
            return Protobuf(ServiceMapContentResponse {
                service_map_contents: vec![],
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
            return Protobuf(ServiceMapContentResponse {
                service_map_contents: vec![],
                code: 401,
                message: msg,
            });
        }
    };

    // 3) 权限校验：只有 Admin (permission=3) 才能创建服务地图内容
    let user_permission = auth_user.permission.unwrap_or(0);
    if user_permission != UserPermissionLevel::Admin.level() {
        return Protobuf(ServiceMapContentResponse {
            service_map_contents: vec![],
            code: 403,
            message: "Permission denied: Only Admin can create service map content".to_string(),
        });
    }

    // 4) 验证必填参数
    if payload.type_one == 0 {
        return Protobuf(ServiceMapContentResponse {
            service_map_contents: vec![],
            code: 400,
            message: "Missing required parameter: type_one".to_string(),
        });
    }

    if payload.type_two.is_empty() {
        return Protobuf(ServiceMapContentResponse {
            service_map_contents: vec![],
            code: 400,
            message: "Missing required parameter: type_two".to_string(),
        });
    }

    // 5) 解析 JSON 字符串
    let content_json: Option<Json> = if payload.content.is_empty() {
        None
    } else {
        match payload.content.parse::<Json>() {
            Ok(json) => Some(json),
            Err(_) => {
                return Protobuf(ServiceMapContentResponse {
                    service_map_contents: vec![],
                    code: 400,
                    message: "Invalid JSON format for content".to_string(),
                });
            }
        }
    };

    // 6) 创建新的服务地图内容
    let new_service_map_content = service_map_content_entity::ActiveModel {
        id: Default::default(), // auto increment
        type_one: Set(Some(payload.type_one)),
        type_two: Set(Some(payload.type_two.clone())),
        content: Set(content_json),
    };

    let db = state.database.clone();
    let inserted = match new_service_map_content.insert(db.as_ref()).await {
        Ok(m) => m,
        Err(err) => {
            return Protobuf(ServiceMapContentResponse {
                service_map_contents: vec![],
                code: 500,
                message: format!("Failed to create service map content: {}", err),
            });
        }
    };

    // 7) 返回创建的服务地图内容
    Protobuf(ServiceMapContentResponse {
        service_map_contents: vec![ProtoServiceMapContent {
            id: inserted.id,
            type_one: inserted.type_one.unwrap_or_default(),
            type_two: inserted.type_two.unwrap_or_default(),
            content: inserted
                .content
                .map(|json| json.to_string())
                .unwrap_or_default(),
        }],
        code: 200,
        message: "Create service map content success".to_string(),
    })
}

