use axum::{Router, extract::State, http::HeaderMap, routing::post};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::health_guide_type as health_guide_type_entity;
use interface_types::proto::health_guide_type::{
    HealthGuideType as ProtoHealthGuideType, HealthGuideTypeRequest, HealthGuideTypeResponse,
};
use sea_orm::{ActiveModelTrait, Set, prelude::Json};
use user_auth::db_exchange::{ExchangeError, token2user};
use user_auth::user_auth::UserPermissionLevel;

use crate::AppState;

/// 创建 health_guide_type 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", post(create_health_guide_type))
}

/// POST /api/health_guide_type - 创建健康指南类型（仅 Admin 权限可以访问）
async fn create_health_guide_type(
    State(state): State<AppState>,
    headers: HeaderMap,
    Protobuf(payload): Protobuf<HealthGuideTypeRequest>,
) -> Protobuf<HealthGuideTypeResponse> {
    // 1) 从 Header 提取 token
    let token: &str = match headers.get("Authorization") {
        Some(t) => match t.to_str() {
            Ok(s) => s,
            Err(_) => {
                return Protobuf(HealthGuideTypeResponse {
                    health_guide_types: vec![],
                    code: 401,
                    message: "Invalid token format".to_string(),
                });
            }
        },
        None => {
            return Protobuf(HealthGuideTypeResponse {
                health_guide_types: vec![],
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
            return Protobuf(HealthGuideTypeResponse {
                health_guide_types: vec![],
                code: 401,
                message: msg,
            });
        }
    };

    // 3) 权限校验：只有 Admin (permission=3) 才能创建健康指南类型
    let user_permission = auth_user.permission.unwrap_or(0);
    if user_permission != UserPermissionLevel::Admin.level() {
        return Protobuf(HealthGuideTypeResponse {
            health_guide_types: vec![],
            code: 403,
            message: "Permission denied: Only Admin can create health guide type".to_string(),
        });
    }

    // 4) 解析 JSON 字符串
    let type_one_json: Option<Json> = if payload.type_one.is_empty() {
        None
    } else {
        match payload.type_one.parse::<Json>() {
            Ok(json) => Some(json),
            Err(_) => {
                return Protobuf(HealthGuideTypeResponse {
                    health_guide_types: vec![],
                    code: 400,
                    message: "Invalid JSON format for type_one".to_string(),
                });
            }
        }
    };

    // 5) 创建新的健康指南类型
    let new_health_guide_type = health_guide_type_entity::ActiveModel {
        id: Default::default(), // auto increment
        type_name: Set(if payload.type_name.is_empty() {
            None
        } else {
            Some(payload.type_name.clone())
        }),
        icon: Set(Some(payload.icon)),
        type_sum: Set(Some(payload.type_sum)),
        type_one: Set(type_one_json),
    };

    let db = state.database.clone();
    let inserted = match new_health_guide_type.insert(db.as_ref()).await {
        Ok(m) => m,
        Err(err) => {
            return Protobuf(HealthGuideTypeResponse {
                health_guide_types: vec![],
                code: 500,
                message: format!("Failed to create health guide type: {}", err),
            });
        }
    };

    // 6) 返回创建的健康指南类型
    Protobuf(HealthGuideTypeResponse {
        health_guide_types: vec![ProtoHealthGuideType {
            id: inserted.id,
            type_name: inserted.type_name.unwrap_or_default(),
            icon: inserted.icon.unwrap_or_default(),
            type_sum: inserted.type_sum.unwrap_or_default(),
            type_one: inserted
                .type_one
                .map(|json| json.to_string())
                .unwrap_or_default(),
        }],
        code: 200,
        message: "Create health guide type success".to_string(),
    })
}
