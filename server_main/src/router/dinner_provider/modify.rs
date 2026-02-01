use axum::{
    Router,
    extract::{Query, State},
    http::HeaderMap,
    routing::put,
};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::dinner_provider as dinner_provider_entity;
use interface_types::proto::dinner_provider::{
    DinnerProvider as ProtoDinnerProvider, DinnerProviderRequest, DinnerProviderResponse,
};
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use user_auth::db_exchange::{ExchangeError, token2user};
use user_auth::user_auth::UserPermissionLevel;

use crate::AppState;

/// 创建 dinner_provider 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", put(modify_dinner_provider))
}

/// 查询参数
#[derive(Debug, Deserialize)]
struct DinnerProviderParams {
    /// 供餐点的 ID
    id: i32,
}

/// PUT /api/dinner_provider?id=xxx - 修改供餐点（仅 Manager/Admin 权限可以访问）
async fn modify_dinner_provider(
    State(state): State<AppState>,
    Query(params): Query<DinnerProviderParams>,
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

    // 3) 权限校验：只有 Manager/Admin (permission=2/3) 才能修改供餐点
    let user_permission = auth_user.permission.unwrap_or(0);
    if user_permission != UserPermissionLevel::Admin.level()
        && user_permission != UserPermissionLevel::Provider.level()
    {
        return Protobuf(DinnerProviderResponse {
            dinner_providers: vec![],
            code: 403,
            message: "Permission denied: Only Manager/Admin can modify dinner provider".to_string(),
        });
    }

    // 4) 查找目标供餐点
    let db = state.database.clone();
    let target = match dinner_provider_entity::Entity::find()
        .filter(dinner_provider_entity::Column::Id.eq(params.id))
        .one(db.as_ref())
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            return Protobuf(DinnerProviderResponse {
                dinner_providers: vec![],
                code: 404,
                message: "Dinner provider not found".to_string(),
            });
        }
        Err(err) => {
            return Protobuf(DinnerProviderResponse {
                dinner_providers: vec![],
                code: 500,
                message: format!("Database error: {}", err),
            });
        }
    };

    // 5) 应用部分更新：payload 中非空/非零 的字段覆盖，其他保持不变
    let mut active: dinner_provider_entity::ActiveModel = target.clone().into();

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
    if !payload.bonus_info.is_empty() {
        active.bonus_info = Set(Some(payload.bonus_info));
    }
    if !payload.meal_style.is_empty() {
        active.meal_style = Set(Some(payload.meal_style));
    }

    // 保留原主键
    active.id = ActiveValue::Unchanged(target.id);

    // 6) 更新数据库
    let target_updated = match active.update(db.as_ref()).await {
        Ok(m) => m,
        Err(err) => {
            return Protobuf(DinnerProviderResponse {
                dinner_providers: vec![],
                code: 500,
                message: format!("Failed to update dinner provider: {}", err),
            });
        }
    };

    // 7) 返回更新后的供餐点
    Protobuf(DinnerProviderResponse {
        dinner_providers: vec![ProtoDinnerProvider {
            id: target_updated.id,
            name: target_updated.name.unwrap_or_default(),
            address: target_updated.address.unwrap_or_default(),
            phone: target_updated.phone.unwrap_or_default(),
            latitude: target_updated.latitude.unwrap_or_default(),
            longitude: target_updated.longitude.unwrap_or_default(),
            service_time: target_updated.service_time.unwrap_or_default(),
            bonus_info: target_updated.bonus_info.unwrap_or_default(),
            meal_style: target_updated.meal_style.unwrap_or_default(),
            create_time: target_updated.create_time.and_utc().timestamp(),
        }],
        code: 200,
        message: "Modify dinner provider success".to_string(),
    })
}
