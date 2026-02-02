use axum::{Router, extract::State, routing::get};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::service_map_type as service_map_type_entity;
use interface_types::proto::service_map_type::{
    ServiceMapType as ProtoServiceMapType, ServiceMapTypeResponse,
};
use sea_orm::EntityTrait;

use crate::AppState;

/// 创建 service_map_type 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_service_map_type))
}

/// GET /api/service_map_type - 获取所有服务地图类型（所有权限均可访问）
async fn get_service_map_type(State(state): State<AppState>) -> Protobuf<ServiceMapTypeResponse> {
    let db = state.database.clone();

    // 查询所有服务地图类型
    let service_map_types = match service_map_type_entity::Entity::find()
        .all(db.as_ref())
        .await
    {
        Ok(types) => types,
        Err(err) => {
            return Protobuf(ServiceMapTypeResponse {
                service_map_types: vec![],
                code: 500,
                message: format!("Database error: {}", err),
            });
        }
    };

    // 转换为 proto 格式
    let proto_types: Vec<ProtoServiceMapType> = service_map_types
        .into_iter()
        .map(|t| ProtoServiceMapType {
            id: t.id,
            community_name: t.community_name.unwrap_or_default(),
            type_sum: t.type_sum.unwrap_or_default(),
            type_name: t.type_name.map(|json| json.to_string()).unwrap_or_default(),
        })
        .collect();

    Protobuf(ServiceMapTypeResponse {
        service_map_types: proto_types,
        code: 200,
        message: "Get service map types success".to_string(),
    })
}
