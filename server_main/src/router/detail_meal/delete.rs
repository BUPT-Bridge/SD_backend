use axum::{
    Router,
    extract::{Query, State},
    http::HeaderMap,
    routing::delete,
};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::detail_meal as detail_meal_entity;
use interface_types::proto::detail_meal::DetailMealResponse;
use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter};
use serde::Deserialize;
use user_auth::db_exchange::{ExchangeError, token2user};
use user_auth::user_auth::UserPermissionLevel;

use crate::AppState;

/// 创建 detail_meal 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", delete(delete_detail_meal))
}

/// 查询参数
#[derive(Debug, Deserialize)]
struct DetailMealParams {
    /// 明细餐的 ID
    id: i32,
}

/// DELETE /api/detail_meal?id=xxx - 删除明细餐（仅 Admin 权限可以访问）
async fn delete_detail_meal(
    State(state): State<AppState>,
    Query(params): Query<DetailMealParams>,
    headers: HeaderMap,
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

    // 3) 权限校验：只有 Admin (permission=3) 才能删除明细餐
    let user_permission = auth_user.permission.unwrap_or(0);
    if user_permission != UserPermissionLevel::Admin.level() {
        return Protobuf(DetailMealResponse {
            detail_meals: vec![],
            code: 403,
            message: "Permission denied: Only Admin can delete detail meal".to_string(),
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

    // 5) 删除明细餐
    match target.delete(db.as_ref()).await {
        Ok(_) => {
            Protobuf(DetailMealResponse {
                detail_meals: vec![],
                code: 200,
                message: "Delete detail meal success".to_string(),
            })
        }
        Err(err) => {
            Protobuf(DetailMealResponse {
                detail_meals: vec![],
                code: 500,
                message: format!("Failed to delete detail meal: {}", err),
            })
        }
    }
}
