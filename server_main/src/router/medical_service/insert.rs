use axum::{Router, extract::State, http::HeaderMap, routing::post};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::medical_service as medical_service_entity;
use interface_types::proto::medical_service::{
    MedicalService as ProtoMedicalService, MedicalServiceRequest, MedicalServiceResponse,
};
use sea_orm::{ActiveModelTrait, Set};
use user_auth::db_exchange::{ExchangeError, token2user};
use user_auth::user_auth::UserPermissionLevel;

use crate::AppState;

/// 创建 medical_service 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", post(insert_medical_service))
}

/// POST /api/medical_service - 新增医疗服务（仅 Admin 权限可以访问）
async fn insert_medical_service(
    State(state): State<AppState>,
    headers: HeaderMap,
    Protobuf(payload): Protobuf<MedicalServiceRequest>,
) -> Protobuf<MedicalServiceResponse> {
    // 1) 从 Header 提取 token
    let token: &str = match headers.get("Authorization") {
        Some(t) => match t.to_str() {
            Ok(s) => s,
            Err(_) => {
                return Protobuf(MedicalServiceResponse {
                    medical_services: vec![],
                    code: 401,
                    message: "Invalid token format".to_string(),
                });
            }
        },
        None => {
            return Protobuf(MedicalServiceResponse {
                medical_services: vec![],
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
            return Protobuf(MedicalServiceResponse {
                medical_services: vec![],
                code: 401,
                message: msg,
            });
        }
    };

    // 3) 权限校验：只有 Admin (permission=3) 才能新增医疗服务
    let user_permission = auth_user.permission.unwrap_or(0);
    if user_permission != UserPermissionLevel::Admin.level() {
        return Protobuf(MedicalServiceResponse {
            medical_services: vec![],
            code: 403,
            message: "Permission denied: Only Admin can insert medical service".to_string(),
        });
    }

    // 4) 创建新的 ActiveModel 并插入
    let db = state.database.clone();
    let new_medical_service = medical_service_entity::ActiveModel {
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
        ..Default::default()
    };

    // 5) 执行插入
    let inserted_medical_service = match new_medical_service.insert(db.as_ref()).await {
        Ok(n) => n,
        Err(err) => {
            return Protobuf(MedicalServiceResponse {
                medical_services: vec![],
                code: 500,
                message: format!("Failed to insert medical service: {}", err),
            });
        }
    };

    // 6) 返回新增的医疗服务
    Protobuf(MedicalServiceResponse {
        medical_services: vec![ProtoMedicalService {
            id: inserted_medical_service.id,
            name: inserted_medical_service.name.unwrap_or_default(),
            address: inserted_medical_service.address.unwrap_or_default(),
            phone: inserted_medical_service.phone.unwrap_or_default(),
            latitude: inserted_medical_service.latitude.unwrap_or_default(),
            longitude: inserted_medical_service.longitude.unwrap_or_default(),
            service_time: inserted_medical_service.service_time.unwrap_or_default(),
            create_time: inserted_medical_service.create_time.and_utc().timestamp(),
        }],
        code: 200,
        message: "Insert medical service success".to_string(),
    })
}

