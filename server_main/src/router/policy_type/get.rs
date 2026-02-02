use axum::{Router, extract::State, routing::get};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::policy_type as policy_type_entity;
use interface_types::proto::policy_type::{
    PolicyType as ProtoPolicyType, PolicyTypeResponse,
};
use sea_orm::EntityTrait;

use crate::AppState;

/// 创建 policy_type 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_policy_type))
}

/// GET /api/policy_type - 获取所有政策类型（所有权限均可访问）
async fn get_policy_type(State(state): State<AppState>) -> Protobuf<PolicyTypeResponse> {
    let db = state.database.clone();

    // 查询所有政策类型
    let policy_types = match policy_type_entity::Entity::find()
        .all(db.as_ref())
        .await
    {
        Ok(types) => types,
        Err(err) => {
            return Protobuf(PolicyTypeResponse {
                policy_types: vec![],
                code: 500,
                message: format!("Database error: {}", err),
            });
        }
    };

    // 转换为 proto 格式
    let proto_types: Vec<ProtoPolicyType> = policy_types
        .into_iter()
        .map(|t| ProtoPolicyType {
            id: t.id,
            r#type: t.r#type.unwrap_or_default(),
        })
        .collect();

    Protobuf(PolicyTypeResponse {
        policy_types: proto_types,
        code: 200,
        message: "Get policy types success".to_string(),
    })
}
