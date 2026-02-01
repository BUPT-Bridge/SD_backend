use axum::{Router, extract::State, http::HeaderMap, routing::post};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::detail_meal as detail_meal_entity;
use interface_types::proto::detail_meal::{
    DetailMeal as ProtoDetailMeal, DetailMealRequest, DetailMealResponse,
};
use sea_orm::{ActiveModelTrait,  prelude::Json, Set};
use user_auth::db_exchange::{ExchangeError, token2user};
use user_auth::user_auth::UserPermissionLevel;

use crate::AppState;

/// 创建 detail_meal 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", post(insert_detail_meal))
}

/// POST /api/detail_meal - 新增明细餐（仅 Provider/Admin 权限可以访问）
async fn insert_detail_meal(
    State(state): State<AppState>,
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

    // 3) 权限校验：只有 Provider/Admin (permission=2/3) 才能新增明细餐
    let user_permission = auth_user.permission.unwrap_or(0);
    if user_permission != UserPermissionLevel::Admin.level()
        && user_permission != UserPermissionLevel::Provider.level()
    {
        return Protobuf(DetailMealResponse {
            detail_meals: vec![],
            code: 403,
            message: "Permission denied: Only Provider/Admin can insert detail meal".to_string(),
        });
    }

    // Provider 强制使用自身 name 或 open_id 作为 belong_to
    let belong_to_value = if user_permission == UserPermissionLevel::Provider.level() {
        Some(
            auth_user
                .name
                .clone()
                .filter(|name| !name.is_empty())
                .unwrap_or(auth_user.open_id.clone()),
        )
    } else if payload.belong_to.is_empty() {
        None
    } else {
        Some(payload.belong_to)
    };

    // 4) 解析 meal_info 的 JSON 字符串
    let meal_info_json: Option<Json> = if payload.meal_info.is_empty() {
        None
    } else {
        match payload.meal_info.parse::<Json>() {
            Ok(v) => Some(v),
            Err(_) => {
                return Protobuf(DetailMealResponse {
                    detail_meals: vec![],
                    code: 400,
                    message: "Invalid meal_info JSON".to_string(),
                });
            }
        }
    };

    // 5) 创建新的 ActiveModel 并插入
    let db = state.database.clone();
    let new_detail_meal = detail_meal_entity::ActiveModel {
        r#type: Set(if payload.r#type.is_empty() {
            None
        } else {
            Some(payload.r#type)
        }),
        date_time: Set(if payload.date_time.is_empty() {
            None
        } else {
            Some(payload.date_time)
        }),
        meal_info: Set(meal_info_json),
        belong_to: Set(belong_to_value),
        ..Default::default()
    };

    // 6) 执行插入
    let inserted_detail_meal = match new_detail_meal.insert(db.as_ref()).await {
        Ok(n) => n,
        Err(err) => {
            return Protobuf(DetailMealResponse {
                detail_meals: vec![],
                code: 500,
                message: format!("Failed to insert detail meal: {}", err),
            });
        }
    };

    // 7) 返回新增的明细餐
    Protobuf(DetailMealResponse {
        detail_meals: vec![ProtoDetailMeal {
            id: inserted_detail_meal.id,
            r#type: inserted_detail_meal.r#type.unwrap_or_default(),
            date_time: inserted_detail_meal.date_time.unwrap_or_default(),
            meal_info: inserted_detail_meal
                .meal_info
                .map(|v| v.to_string())
                .unwrap_or_default(),
            belong_to: inserted_detail_meal.belong_to.unwrap_or_default(),
        }],
        code: 200,
        message: "Insert detail meal success".to_string(),
    })
}
