use axum::{
    Router,
    extract::{Query, State},
    http::HeaderMap,
    routing::put,
};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::resource_service as resource_service_entity;
use interface_types::proto::resource_service::{
    ResourceService as ProtoResourceService, ResourceServiceRequest, ResourceServiceResponse,
};
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use user_auth::db_exchange::{ExchangeError, token2user};
use user_auth::user_auth::UserPermissionLevel;

use crate::AppState;

/// 创建 resource_service 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", put(modify_resource_service))
}

/// 查询参数
#[derive(Debug, Deserialize)]
struct ResourceServiceParams {
    /// 资源服务的 ID
    id: i32,
}

/// PUT /api/resource_service?id=xxx - 修改资源服务（仅 Admin 权限可以访问）
async fn modify_resource_service(
    State(state): State<AppState>,
    Query(params): Query<ResourceServiceParams>,
    headers: HeaderMap,
    Protobuf(payload): Protobuf<ResourceServiceRequest>,
) -> Protobuf<ResourceServiceResponse> {
    // 1) 从 Header 提取 token
    let token: &str = match headers.get("Authorization") {
        Some(t) => match t.to_str() {
            Ok(s) => s,
            Err(_) => {
                return Protobuf(ResourceServiceResponse {
                    resource_services: vec![],
                    code: 401,
                    message: "Invalid token format".to_string(),
                });
            }
        },
        None => {
            return Protobuf(ResourceServiceResponse {
                resource_services: vec![],
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
            return Protobuf(ResourceServiceResponse {
                resource_services: vec![],
                code: 401,
                message: msg,
            });
        }
    };

    // 3) 权限校验：只有 Admin (permission=3) 才能修改资源服务
    let user_permission = auth_user.permission.unwrap_or(0);
    if user_permission != UserPermissionLevel::Admin.level() {
        return Protobuf(ResourceServiceResponse {
            resource_services: vec![],
            code: 403,
            message: "Permission denied: Only Admin can modify resource service".to_string(),
        });
    }

    // 4) 查找目标资源服务
    let db = state.database.clone();
    let target = match resource_service_entity::Entity::find()
        .filter(resource_service_entity::Column::Id.eq(params.id))
        .one(db.as_ref())
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            return Protobuf(ResourceServiceResponse {
                resource_services: vec![],
                code: 404,
                message: "Resource service not found".to_string(),
            });
        }
        Err(err) => {
            return Protobuf(ResourceServiceResponse {
                resource_services: vec![],
                code: 500,
                message: format!("Database error: {}", err),
            });
        }
    };

    // 5) 应用部分更新：payload 中非空/非零 的字段覆盖，其他保持不变
    let mut active: resource_service_entity::ActiveModel = target.clone().into();

    if !payload.name.is_empty() {
        active.name = Set(Some(payload.name));
    }
    if !payload.address.is_empty() {
        active.address = Set(Some(payload.address));
    }
    if !payload.phone.is_empty() {
        active.phone = Set(Some(payload.phone));
    }
    if payload.latitude != 0.0 {
        active.latitude = Set(Some(payload.latitude));
    }
    if payload.longitude != 0.0 {
        active.longitude = Set(Some(payload.longitude));
    }
    if !payload.service_time.is_empty() {
        active.service_time = Set(Some(payload.service_time));
    }
    if !payload.boss.is_empty() {
        active.boss = Set(Some(payload.boss));
    }

    // 保留原主键
    active.id = ActiveValue::Unchanged(target.id);

    // 6) 更新数据库
    let target_updated = match active.update(db.as_ref()).await {
        Ok(m) => m,
        Err(err) => {
            return Protobuf(ResourceServiceResponse {
                resource_services: vec![],
                code: 500,
                message: format!("Failed to update resource service: {}", err),
            });
        }
    };

    // 7) 返回更新后的资源服务
    Protobuf(ResourceServiceResponse {
        resource_services: vec![ProtoResourceService {
            id: target_updated.id,
            name: target_updated.name.unwrap_or_default(),
            address: target_updated.address.unwrap_or_default(),
            phone: target_updated.phone.unwrap_or_default(),
            latitude: target_updated.latitude.unwrap_or_default(),
            longitude: target_updated.longitude.unwrap_or_default(),
            service_time: target_updated.service_time.unwrap_or_default(),
            boss: target_updated.boss.unwrap_or_default(),
            create_time: target_updated.create_time.and_utc().timestamp(),
        }],
        code: 200,
        message: "Modify resource service success".to_string(),
    })
}
