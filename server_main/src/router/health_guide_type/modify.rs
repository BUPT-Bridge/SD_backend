use axum::{
    Router,
    extract::{Query, State},
    http::HeaderMap,
    routing::put,
};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::health_guide_type as health_guide_type_entity;
use interface_types::proto::health_guide_type::{
    HealthGuideType as ProtoHealthGuideType, HealthGuideTypeRequest, HealthGuideTypeResponse,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter, Set, prelude::Json,
};
use serde::Deserialize;
use user_auth::db_exchange::{ExchangeError, token2user};
use user_auth::user_auth::UserPermissionLevel;

use crate::AppState;

/// 创建 health_guide_type 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", put(modify_health_guide_type))
}

/// 查询参数
#[derive(Debug, Deserialize)]
struct HealthGuideTypeParams {
    /// 健康指南类型的 ID
    id: i32,
}

/// PUT /api/health_guide_type?id=xxx - 修改健康指南类型（仅 Admin 权限可以访问）
async fn modify_health_guide_type(
    State(state): State<AppState>,
    Query(params): Query<HealthGuideTypeParams>,
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

    // 3) 权限校验：只有 Admin (permission=3) 才能修改健康指南类型
    let user_permission = auth_user.permission.unwrap_or(0);
    if user_permission != UserPermissionLevel::Admin.level() {
        return Protobuf(HealthGuideTypeResponse {
            health_guide_types: vec![],
            code: 403,
            message: "Permission denied: Only Admin can modify health guide type".to_string(),
        });
    }

    // 4) 查找目标健康指南类型
    let db = state.database.clone();
    let target = match health_guide_type_entity::Entity::find()
        .filter(health_guide_type_entity::Column::Id.eq(params.id))
        .one(db.as_ref())
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            return Protobuf(HealthGuideTypeResponse {
                health_guide_types: vec![],
                code: 404,
                message: "Health guide type not found".to_string(),
            });
        }
        Err(err) => {
            return Protobuf(HealthGuideTypeResponse {
                health_guide_types: vec![],
                code: 500,
                message: format!("Database error: {}", err),
            });
        }
    };

    // 5) 应用部分更新：payload 中非空/非零 的字段覆盖，其他保持不变
    let mut active: health_guide_type_entity::ActiveModel = target.clone().into();

    if !payload.type_name.is_empty() {
        active.type_name = Set(Some(payload.type_name));
    }
    if payload.icon != 0 {
        active.icon = Set(Some(payload.icon));
    }
    if payload.type_sum != 0 {
        active.type_sum = Set(Some(payload.type_sum));
    }
    if !payload.type_one.is_empty() {
        // 解析 JSON 字符串
        match payload.type_one.parse::<Json>() {
            Ok(json) => {
                active.type_one = Set(Some(json));
            }
            Err(_) => {
                return Protobuf(HealthGuideTypeResponse {
                    health_guide_types: vec![],
                    code: 400,
                    message: "Invalid JSON format for type_one".to_string(),
                });
            }
        }
    }

    // 保留原主键
    active.id = ActiveValue::Unchanged(target.id);

    // 6) 更新数据库
    let target_updated = match active.update(db.as_ref()).await {
        Ok(m) => m,
        Err(err) => {
            return Protobuf(HealthGuideTypeResponse {
                health_guide_types: vec![],
                code: 500,
                message: format!("Failed to update health guide type: {}", err),
            });
        }
    };

    // 7) 返回更新后的健康指南类型
    Protobuf(HealthGuideTypeResponse {
        health_guide_types: vec![ProtoHealthGuideType {
            id: target_updated.id,
            type_name: target_updated.type_name.unwrap_or_default(),
            icon: target_updated.icon.unwrap_or_default(),
            type_sum: target_updated.type_sum.unwrap_or_default(),
            type_one: target_updated
                .type_one
                .map(|json| json.to_string())
                .unwrap_or_default(),
        }],
        code: 200,
        message: "Modify health guide type success".to_string(),
    })
}

