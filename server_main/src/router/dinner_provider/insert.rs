use axum::{Router, extract::State, http::HeaderMap, routing::post};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::dinner_provider as dinner_provider_entity;
use interface_types::proto::dinner_provider::{
    DinnerProvider as ProtoDinnerProvider, DinnerProviderRequest, DinnerProviderResponse,
};
use sea_orm::{ActiveModelTrait, Set};
use user_auth::db_exchange::{ExchangeError, token2user};
use user_auth::user_auth::UserPermissionLevel;

use crate::AppState;

/// 创建 dinner_provider 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", post(insert_dinner_provider))
}

/// POST /api/dinner_provider - 新增供餐点（仅 Admin 权限可以访问）
async fn insert_dinner_provider(
    State(state): State<AppState>,
    headers: HeaderMap,
    Protobuf(payload): Protobuf<DinnerProviderRequest>,
) -> Protobuf<DinnerProviderResponse> {
    // 1) 从 Header 提取 token
    let token: &str = match headers.get("Authorization") {
        Some(t) => match t.to_str() {
            Ok(s) => s,
            Err(_) => {
                return Protobuf(DinnerProviderResponse {
                    dinner_providers: vec![],
                    code: 401,
                    message: "Invalid token format".to_string(),
                });
            }
        },
        None => {
            return Protobuf(DinnerProviderResponse {
                dinner_providers: vec![],
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
            return Protobuf(DinnerProviderResponse {
                dinner_providers: vec![],
                code: 401,
                message: msg,
            });
        }
    };

    // 3) 权限校验：只有 Admin (permission=3) 才能新增供餐点
    let user_permission = auth_user.permission.unwrap_or(0);
    if user_permission != UserPermissionLevel::Admin.level() {
        return Protobuf(DinnerProviderResponse {
            dinner_providers: vec![],
            code: 403,
            message: "Permission denied: Only Admin can insert dinner provider".to_string(),
        });
    }

    // 4) 创建新的 ActiveModel 并插入
    let db = state.database.clone();
    let new_dinner_provider = dinner_provider_entity::ActiveModel {
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
        bonus_info: Set(if payload.bonus_info.is_empty() {
            None
        } else {
            Some(payload.bonus_info)
        }),
        meal_style: Set(if payload.meal_style.is_empty() {
            None
        } else {
            Some(payload.meal_style)
        }),
        ..Default::default()
    };

    // 5) 执行插入
    let inserted_dinner_provider = match new_dinner_provider.insert(db.as_ref()).await {
        Ok(n) => n,
        Err(err) => {
            return Protobuf(DinnerProviderResponse {
                dinner_providers: vec![],
                code: 500,
                message: format!("Failed to insert dinner provider: {}", err),
            });
        }
    };

    // 6) 返回新增的供餐点
    Protobuf(DinnerProviderResponse {
        dinner_providers: vec![ProtoDinnerProvider {
            id: inserted_dinner_provider.id,
            name: inserted_dinner_provider.name.unwrap_or_default(),
            address: inserted_dinner_provider.address.unwrap_or_default(),
            phone: inserted_dinner_provider.phone.unwrap_or_default(),
            latitude: inserted_dinner_provider.latitude.unwrap_or_default(),
            longitude: inserted_dinner_provider.longitude.unwrap_or_default(),
            service_time: inserted_dinner_provider.service_time.unwrap_or_default(),
            bonus_info: inserted_dinner_provider.bonus_info.unwrap_or_default(),
            meal_style: inserted_dinner_provider.meal_style.unwrap_or_default(),
            create_time: inserted_dinner_provider.create_time.and_utc().timestamp(),
        }],
        code: 200,
        message: "Insert dinner provider success".to_string(),
    })
}
