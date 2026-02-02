use axum::{
    Router,
    extract::{Query, State},
    http::HeaderMap,
    routing::delete,
};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::service_map_content as service_map_content_entity;
use interface_types::proto::service_map_content::ServiceMapContentResponse;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;
use user_auth::db_exchange::{ExchangeError, token2user};
use user_auth::user_auth::UserPermissionLevel;

use crate::AppState;

/// 创建 service_map_content 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", delete(delete_service_map_content))
}

/// 查询参数
#[derive(Debug, Deserialize)]
struct ServiceMapContentParams {
    /// 类型一（社区 ID）
    type_one: Option<i32>,
    /// 类型二（类型名称）
    type_two: Option<String>,
}

/// DELETE /api/service_map_content?type_one=xxx&type_two=xxx - 删除服务地图内容（仅 Admin 权限可以访问）
/// 必须提供 type_one 和 type_two 参数来筛选要删除的记录
async fn delete_service_map_content(
    State(state): State<AppState>,
    Query(params): Query<ServiceMapContentParams>,
    headers: HeaderMap,
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

    // 3) 权限校验：只有 Admin (permission=3) 才能删除服务地图内容
    let user_permission = auth_user.permission.unwrap_or(0);
    if user_permission != UserPermissionLevel::Admin.level() {
        return Protobuf(ServiceMapContentResponse {
            service_map_contents: vec![],
            code: 403,
            message: "Permission denied: Only Admin can delete service map content".to_string(),
        });
    }

    // 4) 检查必填参数
    let type_one = match params.type_one {
        Some(v) => v,
        None => {
            return Protobuf(ServiceMapContentResponse {
                service_map_contents: vec![],
                code: 400,
                message: "Missing required parameter: type_one".to_string(),
            });
        }
    };

    let type_two = match params.type_two {
        Some(v) if !v.is_empty() => v,
        _ => {
            return Protobuf(ServiceMapContentResponse {
                service_map_contents: vec![],
                code: 400,
                message: "Missing required parameter: type_two".to_string(),
            });
        }
    };

    // 5) 查找要删除的服务地图内容
    let db = state.database.clone();
    let service_map_content_to_delete = match service_map_content_entity::Entity::find()
        .filter(service_map_content_entity::Column::TypeOne.eq(type_one))
        .filter(service_map_content_entity::Column::TypeTwo.eq(&type_two))
        .one(db.as_ref())
        .await
    {
        Ok(Some(s)) => s,
        Ok(None) => {
            return Protobuf(ServiceMapContentResponse {
                service_map_contents: vec![],
                code: 404,
                message: format!(
                    "Service map content with type_one '{}' and type_two '{}' not found",
                    type_one, type_two
                ),
            });
        }
        Err(err) => {
            return Protobuf(ServiceMapContentResponse {
                service_map_contents: vec![],
                code: 500,
                message: format!("Database error: {}", err),
            });
        }
    };

    // 6) 执行删除
    match service_map_content_entity::Entity::delete_by_id(service_map_content_to_delete.id)
        .exec(db.as_ref())
        .await
    {
        Ok(_) => {}
        Err(err) => {
            return Protobuf(ServiceMapContentResponse {
                service_map_contents: vec![],
                code: 500,
                message: format!("Failed to delete service map content: {}", err),
            });
        }
    };

    // 7) 返回成功响应
    Protobuf(ServiceMapContentResponse {
        service_map_contents: vec![],
        code: 200,
        message: "Delete service map content success".to_string(),
    })
}
