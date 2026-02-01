use axum::{Router, extract::State, http::HeaderMap, routing::post};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::resource_service as resource_service_entity;
use interface_types::proto::resource_service::{
    ResourceService as ProtoResourceService, ResourceServiceRequest, ResourceServiceResponse,
};
use sea_orm::{ActiveModelTrait, Set};
use user_auth::db_exchange::{ExchangeError, token2user};
use user_auth::user_auth::UserPermissionLevel;

use crate::AppState;

/// 创建 resource_service 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", post(insert_resource_service))
}

/// POST /api/resource_service - 新增资源服务（仅 Admin 权限可以访问）
async fn insert_resource_service(
    State(state): State<AppState>,
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

    // 3) 权限校验：只有 Admin (permission=3) 才能新增资源服务
    let user_permission = auth_user.permission.unwrap_or(0);
    if user_permission != UserPermissionLevel::Admin.level() {
        return Protobuf(ResourceServiceResponse {
            resource_services: vec![],
            code: 403,
            message: "Permission denied: Only Admin can insert resource service".to_string(),
        });
    }

    // 4) 创建新的 ActiveModel 并插入
    let db = state.database.clone();
    let new_resource_service = resource_service_entity::ActiveModel {
        name: Set(if payload.name.is_empty() {
            None
        } else {
            Some(payload.name)
        }),
        address: Set(if payload.address.is_empty() {
            None
        } else {
            Some(payload.address)
        }),
        phone: Set(if payload.phone.is_empty() {
            None
        } else {
            Some(payload.phone)
        }),
        latitude: Set(if payload.latitude == 0.0 {
            None
        } else {
            Some(payload.latitude)
        }),
        longitude: Set(if payload.longitude == 0.0 {
            None
        } else {
            Some(payload.longitude)
        }),
        service_time: Set(if payload.service_time.is_empty() {
            None
        } else {
            Some(payload.service_time)
        }),
        boss: Set(if payload.boss.is_empty() {
            None
        } else {
            Some(payload.boss)
        }),
        ..Default::default()
    };

    // 5) 执行插入
    let inserted_resource_service = match new_resource_service.insert(db.as_ref()).await {
        Ok(n) => n,
        Err(err) => {
            return Protobuf(ResourceServiceResponse {
                resource_services: vec![],
                code: 500,
                message: format!("Failed to insert resource service: {}", err),
            });
        }
    };

    // 6) 返回新增的资源服务
    Protobuf(ResourceServiceResponse {
        resource_services: vec![ProtoResourceService {
            id: inserted_resource_service.id,
            name: inserted_resource_service.name.unwrap_or_default(),
            address: inserted_resource_service.address.unwrap_or_default(),
            phone: inserted_resource_service.phone.unwrap_or_default(),
            latitude: inserted_resource_service.latitude.unwrap_or_default(),
            longitude: inserted_resource_service.longitude.unwrap_or_default(),
            service_time: inserted_resource_service.service_time.unwrap_or_default(),
            boss: inserted_resource_service.boss.unwrap_or_default(),
            create_time: inserted_resource_service.create_time.and_utc().timestamp(),
        }],
        code: 200,
        message: "Insert resource service success".to_string(),
    })
}
