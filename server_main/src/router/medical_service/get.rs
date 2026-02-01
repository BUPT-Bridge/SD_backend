use axum::{Router, extract::State, routing::get};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::medical_service as medical_service_entity;
use interface_types::proto::medical_service::{
    MedicalService as ProtoMedicalService, MedicalServiceResponse,
};
use sea_orm::EntityTrait;

use crate::AppState;

/// 创建 medical_service 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_medical_service))
}

/// GET /api/medical_service - 获取所有医疗服务列表（所有权限 0-3 都可以访问）
async fn get_medical_service(State(state): State<AppState>) -> Protobuf<MedicalServiceResponse> {
    let db = state.database.clone();

    // 查询所有医疗服务
    let medical_services = match medical_service_entity::Entity::find()
        .all(db.as_ref())
        .await
    {
        Ok(n) => n,
        Err(err) => {
            return Protobuf(MedicalServiceResponse {
                medical_services: vec![],
                code: 500,
                message: format!("Database error: {}", err),
            });
        }
    };

    // 转换为 proto 格式并返回
    let proto_medical_services: Vec<ProtoMedicalService> = medical_services
        .into_iter()
        .map(|s| ProtoMedicalService {
            id: s.id,
            name: s.name.unwrap_or_default(),
            address: s.address.unwrap_or_default(),
            phone: s.phone.unwrap_or_default(),
            latitude: s.latitude.unwrap_or_default(),
            longitude: s.longitude.unwrap_or_default(),
            service_time: s.service_time.unwrap_or_default(),
            create_time: s.create_time.and_utc().timestamp(),
        })
        .collect();

    Protobuf(MedicalServiceResponse {
        medical_services: proto_medical_services,
        code: 200,
        message: "Get medical service list success".to_string(),
    })
}
