use axum::{
    Router,
    extract::{Query, State},
    http::HeaderMap,
    routing::put,
};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::detail_meal as detail_meal_entity;
use interface_types::proto::detail_meal::{
    DetailMeal as ProtoDetailMeal, DetailMealRequest, DetailMealResponse,
};
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait,  prelude::Json, QueryFilter, Set};
use serde::Deserialize;
use user_auth::db_exchange::{ExchangeError, token2user};
use user_auth::user_auth::UserPermissionLevel;

use crate::AppState;

/// 创建 detail_meal 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", put(modify_detail_meal))
}

/// 查询参数
#[derive(Debug, Deserialize)]
struct DetailMealParams {
    /// 明细餐的 ID
    id: i32,
}

/// PUT /api/detail_meal?id=xxx - 修改明细餐（仅 Provider/Admin 权限可以访问）
async fn modify_detail_meal(
    State(state): State<AppState>,
    Query(params): Query<DetailMealParams>,
    headers: HeaderMap,
    Protobuf(payload): Protobuf<DetailMealRequest>,
) -> Protobuf<DetailMealResponse> {
    // 1) 从 Header 提取 token
    let token: &str = match headers.get("Authorization") {
        Some(t) => match t.to_str() {
            Ok(s) => s,
            Err(_) => {
                return Protobuf(DetailMealResponse {
                    detail_meals: vec![],
                    code: 401,
                    message: "Invalid token format".to_string(),
                });
            }
        },
        None => {
            return Protobuf(DetailMealResponse {
                detail_meals: vec![],
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
            return Protobuf(DetailMealResponse {
                detail_meals: vec![],
                code: 401,
                message: msg,
            });
        }
    };

    // 3) 权限校验：只有 Provider/Admin (permission=2/3) 才能修改明细餐
    let user_permission = auth_user.permission.unwrap_or(0);
    if user_permission != UserPermissionLevel::Admin.level()
        && user_permission != UserPermissionLevel::Provider.level()
    {
        return Protobuf(DetailMealResponse {
            detail_meals: vec![],
            code: 403,
            message: "Permission denied: Only Provider/Admin can modify detail meal".to_string(),
        });
    }

    // 4) 查找目标明细餐
    let db = state.database.clone();
    let target = match detail_meal_entity::Entity::find()
        .filter(detail_meal_entity::Column::Id.eq(params.id))
        .one(db.as_ref())
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            return Protobuf(DetailMealResponse {
                detail_meals: vec![],
                code: 404,
                message: "Detail meal not found".to_string(),
            });
        }
        Err(err) => {
            return Protobuf(DetailMealResponse {
                detail_meals: vec![],
                code: 500,
                message: format!("Database error: {}", err),
            });
        }
    };

    // Provider 只能修改自己所属
    if user_permission == UserPermissionLevel::Provider.level() {
        let provider_key = auth_user
            .name
            .clone()
            .filter(|name| !name.is_empty())
            .unwrap_or(auth_user.open_id.clone());
        let belong_to = target.belong_to.clone().unwrap_or_default();
        if belong_to != provider_key {
            return Protobuf(DetailMealResponse {
                detail_meals: vec![],
                code: 403,
                message: "Permission denied: Provider can only modify own detail meal".to_string(),
            });
        }
    }

    // 5) 应用部分更新：payload 中非空字段覆盖，其他保持不变
    let mut active: detail_meal_entity::ActiveModel = target.clone().into();

    if !payload.r#type.is_empty() {
        active.r#type = Set(Some(payload.r#type));
    }
    if !payload.date_time.is_empty() {
        active.date_time = Set(Some(payload.date_time));
    }
    if !payload.belong_to.is_empty() {
        if user_permission == UserPermissionLevel::Provider.level() {
            return Protobuf(DetailMealResponse {
                detail_meals: vec![],
                code: 403,
                message: "Permission denied: Provider cannot change belong_to".to_string(),
            });
        }
        active.belong_to = Set(Some(payload.belong_to));
    }
    if !payload.meal_info.is_empty() {
        match payload.meal_info.parse::<Json>() {
            Ok(v) => {
                active.meal_info = Set(Some(v));
            }
            Err(_) => {
                return Protobuf(DetailMealResponse {
                    detail_meals: vec![],
                    code: 400,
                    message: "Invalid meal_info JSON".to_string(),
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
            return Protobuf(DetailMealResponse {
                detail_meals: vec![],
                code: 500,
                message: format!("Failed to update detail meal: {}", err),
            });
        }
    };

    // 7) 返回更新后的明细餐
    Protobuf(DetailMealResponse {
        detail_meals: vec![ProtoDetailMeal {
            id: target_updated.id,
            r#type: target_updated.r#type.unwrap_or_default(),
            date_time: target_updated.date_time.unwrap_or_default(),
            meal_info: target_updated
                .meal_info
                .map(|v| v.to_string())
                .unwrap_or_default(),
            belong_to: target_updated.belong_to.unwrap_or_default(),
        }],
        code: 200,
        message: "Modify detail meal success".to_string(),
    })
}
