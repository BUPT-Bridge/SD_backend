use axum::{
    Router,
    extract::{Query, State},
    routing::get,
};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::detail_meal as detail_meal_entity;
use interface_types::proto::detail_meal::{DetailMeal as ProtoDetailMeal, DetailMealResponse};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;

use crate::AppState;

/// 创建 detail_meal 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_detail_meal))
}

/// 查询参数（每个参数只能是单个值）
/// - belongto: ?belongto=3
/// - datetime: ?datetime=1月21日
/// - type: ?type=早餐
#[derive(Debug, Deserialize)]
struct DetailMealQuery {
    #[serde(rename = "belongto")]
    belong_to: Option<String>,
    #[serde(rename = "datetime")]
    date_time: Option<String>,
    #[serde(rename = "type")]
    r#type: Option<String>,
}

/// GET /api/detail_meal - 获取明细餐列表（所有权限 0-3 都可以访问）
async fn get_detail_meal(
    State(state): State<AppState>,
    Query(params): Query<DetailMealQuery>,
) -> Protobuf<DetailMealResponse> {
    let db = state.database.clone();

    let mut query = detail_meal_entity::Entity::find();

    if let Some(value) = params.belong_to {
        query = query.filter(detail_meal_entity::Column::BelongTo.eq(value));
    }

    if let Some(value) = params.date_time {
        query = query.filter(detail_meal_entity::Column::DateTime.eq(value));
    }

    if let Some(value) = params.r#type {
        query = query.filter(detail_meal_entity::Column::Type.eq(value));
    }

    let detail_meals = match query.all(db.as_ref()).await {
        Ok(n) => n,
        Err(err) => {
            return Protobuf(DetailMealResponse {
                detail_meals: vec![],
                code: 500,
                message: format!("Database error: {}", err),
            });
        }
    };

    let proto_detail_meals = detail_meals
        .into_iter()
        .map(|s| ProtoDetailMeal {
            id: s.id,
            r#type: s.r#type.unwrap_or_default(),
            date_time: s.date_time.unwrap_or_default(),
            meal_info: s.meal_info.map(|v| v.to_string()).unwrap_or_default(),
            belong_to: s.belong_to.unwrap_or_default(),
        })
        .collect();

    Protobuf(DetailMealResponse {
        detail_meals: proto_detail_meals,
        code: 200,
        message: "Get detail meal list success".to_string(),
    })
}
