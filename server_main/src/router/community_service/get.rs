use axum::{Router, extract::State, routing::get};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::community_service as community_service_entity;
use interface_types::proto::community_service::{
    CommunityService as ProtoCommunityService, CommunityServiceResponse,
};
use sea_orm::EntityTrait;

use crate::AppState;

/// 创建 community_service 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_community_service))
}

/// GET /api/community_service - 获取所有社区服务列表（所有权限 0-3 都可以访问）
async fn get_community_service(
    State(state): State<AppState>,
) -> Protobuf<CommunityServiceResponse> {
    let db = state.database.clone();

    // 查询所有社区服务
    let community_services = match community_service_entity::Entity::find()
        .all(db.as_ref())
        .await
    {
        Ok(n) => n,
        Err(err) => {
            return Protobuf(CommunityServiceResponse {
                community_services: vec![],
                code: 500,
                message: format!("Database error: {}", err),
            });
        }
    };

    // 转换为 proto 格式并返回
    let proto_community_services: Vec<ProtoCommunityService> = community_services
        .into_iter()
        .map(|s| ProtoCommunityService {
            id: s.id,
            name: s.name.unwrap_or_default(),
            address: s.address.unwrap_or_default(),
            phone: s.phone.unwrap_or_default(),
            latitude: s.latitude.unwrap_or_default(),
            longitude: s.longitude.unwrap_or_default(),
            create_time: s.create_time.and_utc().timestamp(),
        })
        .collect();

    Protobuf(CommunityServiceResponse {
        community_services: proto_community_services,
        code: 200,
        message: "Get community service list success".to_string(),
    })
}
