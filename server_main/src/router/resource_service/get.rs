use axum::{Router, extract::State, routing::get};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::resource_service as resource_service_entity;
use interface_types::proto::resource_service::{
    ResourceService as ProtoResourceService, ResourceServiceResponse,
};
use sea_orm::EntityTrait;

use crate::AppState;

/// 创建 resource_service 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_resource_service))
}

/// GET /api/resource_service - 获取所有资源服务列表（所有权限 0-3 都可以访问）
async fn get_resource_service(State(state): State<AppState>) -> Protobuf<ResourceServiceResponse> {
    let db = state.database.clone();

    // 查询所有资源服务
    let resource_services = match resource_service_entity::Entity::find()
        .all(db.as_ref())
        .await
    {
        Ok(n) => n,
        Err(err) => {
            return Protobuf(ResourceServiceResponse {
                resource_services: vec![],
                code: 500,
                message: format!("Database error: {}", err),
            });
        }
    };

    // 转换为 proto 格式并返回
    let proto_resource_services: Vec<ProtoResourceService> = resource_services
        .into_iter()
        .map(|s| ProtoResourceService {
            id: s.id,
            name: s.name.unwrap_or_default(),
            address: s.address.unwrap_or_default(),
            phone: s.phone.unwrap_or_default(),
            latitude: s.latitude.unwrap_or_default(),
            longitude: s.longitude.unwrap_or_default(),
            service_time: s.service_time.unwrap_or_default(),
            boss: s.boss.unwrap_or_default(),
            create_time: s.create_time.and_utc().timestamp(),
        })
        .collect();

    Protobuf(ResourceServiceResponse {
        resource_services: proto_resource_services,
        code: 200,
        message: "Get resource service list success".to_string(),
    })
}
