use axum::{
    Router,
    extract::{Query, State},
    http::HeaderMap,
    routing::delete,
};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::service_map_type as service_map_type_entity;
use interface_types::proto::service_map_type::ServiceMapTypeResponse;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;
use user_auth::db_exchange::{ExchangeError, token2user};
use user_auth::user_auth::UserPermissionLevel;

use crate::AppState;

/// 创建 service_map_type 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", delete(delete_service_map_type))
}

/// 查询参数
#[derive(Debug, Deserialize)]
struct ServiceMapTypeParams {
    /// 服务地图类型的 ID
    id: i32,
}

/// DELETE /api/service_map_type?id=xxx - 删除服务地图类型（仅 Admin 权限可以访问）
async fn delete_service_map_type(
    State(state): State<AppState>,
    Query(params): Query<ServiceMapTypeParams>,
    headers: HeaderMap,
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

    // 3) 权限校验：只有 Admin (permission=3) 才能删除服务地图类型
    let user_permission = auth_user.permission.unwrap_or(0);
    if user_permission != UserPermissionLevel::Admin.level() {
        return Protobuf(ServiceMapTypeResponse {
            service_map_types: vec![],
            code: 403,
            message: "Permission denied: Only Admin can delete service map type".to_string(),
        });
    }

    // 4) 查找要删除的服务地图类型
    let db = state.database.clone();
    let service_map_type_to_delete = match service_map_type_entity::Entity::find()
        .filter(service_map_type_entity::Column::Id.eq(params.id))
        .one(db.as_ref())
        .await
    {
        Ok(Some(s)) => s,
        Ok(None) => {
            return Protobuf(ServiceMapTypeResponse {
                service_map_types: vec![],
                code: 404,
                message: format!("Service map type with id '{}' not found", params.id),
            });
        }
        Err(err) => {
            return Protobuf(ServiceMapTypeResponse {
                service_map_types: vec![],
                code: 500,
                message: format!("Database error: {}", err),
            });
        }
    };

    // 5) 执行删除
    match service_map_type_entity::Entity::delete_by_id(service_map_type_to_delete.id)
        .exec(db.as_ref())
        .await
    {
        Ok(_) => {}
        Err(err) => {
            return Protobuf(ServiceMapTypeResponse {
                service_map_types: vec![],
                code: 500,
                message: format!("Failed to delete service map type: {}", err),
            });
        }
    };

    // 6) 返回成功响应
    Protobuf(ServiceMapTypeResponse {
        service_map_types: vec![],
        code: 200,
        message: "Delete service map type success".to_string(),
    })
}
