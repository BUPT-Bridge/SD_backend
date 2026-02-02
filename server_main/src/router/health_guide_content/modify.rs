use axum::{
    Router,
    extract::{Query, State},
    http::HeaderMap,
    routing::put,
};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::health_guide_content as health_guide_content_entity;
use interface_types::proto::health_guide_content::{
    HealthGuideContent as ProtoHealthGuideContent, HealthGuideContentRequest,
    HealthGuideContentResponse,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter, Set, prelude::Json,
};
use serde::Deserialize;
use user_auth::db_exchange::{ExchangeError, token2user};
use user_auth::user_auth::UserPermissionLevel;

use crate::AppState;

/// 创建 health_guide_content 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", put(modify_health_guide_content))
}

/// 查询参数
#[derive(Debug, Deserialize)]
struct HealthGuideContentParams {
    /// 类型一（一级类型 ID）
    type_one: Option<i32>,
    /// 类型二（二级类型名称）
    type_two: Option<String>,
}

/// PUT /api/health_guide_content?type_one=xxx&type_two=xxx - 修改健康指南内容（仅 Admin 权限可以访问）
/// 必须提供 type_one 和 type_two 参数来筛选要修改的记录
async fn modify_health_guide_content(
    State(state): State<AppState>,
    Query(params): Query<HealthGuideContentParams>,
    headers: HeaderMap,
    Protobuf(payload): Protobuf<HealthGuideContentRequest>,
) -> Protobuf<HealthGuideContentResponse> {
    // 1) 从 Header 提取 token
    let token: &str = match headers.get("Authorization") {
        Some(t) => match t.to_str() {
            Ok(s) => s,
            Err(_) => {
                return Protobuf(HealthGuideContentResponse {
                    health_guide_contents: vec![],
                    code: 401,
                    message: "Invalid token format".to_string(),
                });
            }
        },
        None => {
            return Protobuf(HealthGuideContentResponse {
                health_guide_contents: vec![],
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
            return Protobuf(HealthGuideContentResponse {
                health_guide_contents: vec![],
                code: 401,
                message: msg,
            });
        }
    };

    // 3) 权限校验：只有 Admin (permission=3) 才能修改健康指南内容
    let user_permission = auth_user.permission.unwrap_or(0);
    if user_permission != UserPermissionLevel::Admin.level() {
        return Protobuf(HealthGuideContentResponse {
            health_guide_contents: vec![],
            code: 403,
            message: "Permission denied: Only Admin can modify health guide content".to_string(),
        });
    }

    // 4) 检查必填参数
    let type_one = match params.type_one {
        Some(v) => v,
        None => {
            return Protobuf(HealthGuideContentResponse {
                health_guide_contents: vec![],
                code: 400,
                message: "Missing required parameter: type_one".to_string(),
            });
        }
    };

    let type_two = match params.type_two {
        Some(v) if !v.is_empty() => v,
        _ => {
            return Protobuf(HealthGuideContentResponse {
                health_guide_contents: vec![],
                code: 400,
                message: "Missing required parameter: type_two".to_string(),
            });
        }
    };

    // 5) 查找目标健康指南内容
    let db = state.database.clone();
    let target = match health_guide_content_entity::Entity::find()
        .filter(health_guide_content_entity::Column::TypeOne.eq(type_one))
        .filter(health_guide_content_entity::Column::TypeTwo.eq(&type_two))
        .one(db.as_ref())
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            return Protobuf(HealthGuideContentResponse {
                health_guide_contents: vec![],
                code: 404,
                message: "Health guide content not found".to_string(),
            });
        }
        Err(err) => {
            return Protobuf(HealthGuideContentResponse {
                health_guide_contents: vec![],
                code: 500,
                message: format!("Database error: {}", err),
            });
        }
    };

    // 6) 应用部分更新：payload 中非空/非零 的字段覆盖，其他保持不变
    let mut active: health_guide_content_entity::ActiveModel = target.clone().into();

    if payload.type_one != 0 {
        active.type_one = Set(Some(payload.type_one));
    }
    if !payload.type_two.is_empty() {
        active.type_two = Set(Some(payload.type_two));
    }
    if !payload.content.is_empty() {
        // 解析 JSON 字符串
        match payload.content.parse::<Json>() {
            Ok(json) => {
                active.content = Set(Some(json));
            }
            Err(_) => {
                return Protobuf(HealthGuideContentResponse {
                    health_guide_contents: vec![],
                    code: 400,
                    message: "Invalid JSON format for content".to_string(),
                });
            }
        }
    }

    // 保留原主键
    active.id = ActiveValue::Unchanged(target.id);

    // 7) 更新数据库
    let target_updated = match active.update(db.as_ref()).await {
        Ok(m) => m,
        Err(err) => {
            return Protobuf(HealthGuideContentResponse {
                health_guide_contents: vec![],
                code: 500,
                message: format!("Failed to update health guide content: {}", err),
            });
        }
    };

    // 8) 返回更新后的健康指南内容
    Protobuf(HealthGuideContentResponse {
        health_guide_contents: vec![ProtoHealthGuideContent {
            id: target_updated.id,
            type_one: target_updated.type_one.unwrap_or_default(),
            type_two: target_updated.type_two.unwrap_or_default(),
            content: target_updated
                .content
                .map(|json| json.to_string())
                .unwrap_or_default(),
        }],
        code: 200,
        message: "Modify health guide content success".to_string(),
    })
}
