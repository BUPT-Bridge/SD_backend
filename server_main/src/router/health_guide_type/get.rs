use axum::{Router, extract::State, routing::get};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::health_guide_type as health_guide_type_entity;
use interface_types::proto::health_guide_type::{
    HealthGuideType as ProtoHealthGuideType, HealthGuideTypeResponse,
};
use sea_orm::EntityTrait;

use crate::AppState;

/// 创建 health_guide_type 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_health_guide_type))
}

/// GET /api/health_guide_type - 获取所有健康指南类型（所有权限均可访问）
async fn get_health_guide_type(State(state): State<AppState>) -> Protobuf<HealthGuideTypeResponse> {
    let db = state.database.clone();

    // 查询所有健康指南类型
    let health_guide_types = match health_guide_type_entity::Entity::find()
        .all(db.as_ref())
        .await
    {
        Ok(types) => types,
        Err(err) => {
            return Protobuf(HealthGuideTypeResponse {
                health_guide_types: vec![],
                code: 500,
                message: format!("Database error: {}", err),
            });
        }
    };

    // 转换为 proto 格式
    let proto_types: Vec<ProtoHealthGuideType> = health_guide_types
        .into_iter()
        .map(|t| ProtoHealthGuideType {
            id: t.id,
            type_name: t.type_name.unwrap_or_default(),
            icon: t.icon.unwrap_or_default(),
            type_sum: t.type_sum.unwrap_or_default(),
            type_one: t.type_one.map(|json| json.to_string()).unwrap_or_default(),
        })
        .collect();

    Protobuf(HealthGuideTypeResponse {
        health_guide_types: proto_types,
        code: 200,
        message: "Get health guide types success".to_string(),
    })
}
