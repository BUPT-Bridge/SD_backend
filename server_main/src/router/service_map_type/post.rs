use axum::{Router, extract::State, http::HeaderMap, routing::post};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::service_map_type as service_map_type_entity;
use interface_types::proto::service_map_type::{
    ServiceMapType as ProtoServiceMapType, ServiceMapTypeRequest, ServiceMapTypeResponse,
};
use sea_orm::{ActiveModelTrait, Set, prelude::Json};
use user_auth::db_exchange::{ExchangeError, token2user};
use user_auth::user_auth::UserPermissionLevel;

use crate::AppState;

/// 创建 service_map_type 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", post(create_service_map_type))
}

/// POST /api/service_map_type - 创建服务地图类型（仅 Admin 权限可以访问）
async fn create_service_map_type(
    State(state): State<AppState>,
    headers: HeaderMap,
    Protobuf(payload): Protobuf<ServiceMapTypeRequest>,
) -> Protobuf<ServiceMapTypeResponse> {
    // 1) 从 Header 提取 token
    let token: &str = match headers.get("Authorization") {
        Some(t) => match t.to_str() {
            Ok(s) => s,
            Err(_) => {
                return Protobuf(ServiceMapTypeResponse {
                    service_map_types: vec![],
                    code: 401,
                    message: "Invalid token format".to_string(),
                });
            }
        },
        None => {
            return Protobuf(ServiceMapTypeResponse {
                service_map_types: vec![],
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
            return Protobuf(ServiceMapTypeResponse {
                service_map_types: vec![],
                code: 401,
                message: msg,
            });
        }
    };

    // 3) 权限校验：只有 Admin (permission=3) 才能创建服务地图类型
    let user_permission = auth_user.permission.unwrap_or(0);
    if user_permission != UserPermissionLevel::Admin.level() {
        return Protobuf(ServiceMapTypeResponse {
            service_map_types: vec![],
            code: 403,
            message: "Permission denied: Only Admin can create service map type".to_string(),
        });
    }

    // 4) 解析 JSON 字符串
    let type_name_json: Option<Json> = if payload.type_name.is_empty() {
        None
    } else {
        match payload.type_name.parse::<Json>() {
            Ok(json) => Some(json),
            Err(_) => {
                return Protobuf(ServiceMapTypeResponse {
                    service_map_types: vec![],
                    code: 400,
                    message: "Invalid JSON format for type_name".to_string(),
                });
            }
        }
    };

    // 5) 创建新的服务地图类型
    let new_service_map_type = service_map_type_entity::ActiveModel {
        id: Default::default(), // auto increment
        community_name: Set(if payload.community_name.is_empty() {
            None
        } else {
            Some(payload.community_name.clone())
        }),
        type_sum: Set(Some(payload.type_sum)),
        type_name: Set(type_name_json),
    };

    let db = state.database.clone();
    let inserted = match new_service_map_type.insert(db.as_ref()).await {
        Ok(m) => m,
        Err(err) => {
            return Protobuf(ServiceMapTypeResponse {
                service_map_types: vec![],
                code: 500,
                message: format!("Failed to create service map type: {}", err),
            });
        }
    };

    // 6) 返回创建的服务地图类型
    Protobuf(ServiceMapTypeResponse {
        service_map_types: vec![ProtoServiceMapType {
            id: inserted.id,
            community_name: inserted.community_name.unwrap_or_default(),
            type_sum: inserted.type_sum.unwrap_or_default(),
            type_name: inserted
                .type_name
                .map(|json| json.to_string())
                .unwrap_or_default(),
        }],
        code: 200,
        message: "Create service map type success".to_string(),
    })
}
