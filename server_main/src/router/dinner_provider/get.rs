use axum::{Router, extract::State, routing::get};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::dinner_provider as dinner_provider_entity;
use interface_types::proto::dinner_provider::{
    DinnerProvider as ProtoDinnerProvider, DinnerProviderResponse,
};
use sea_orm::EntityTrait;

use crate::AppState;

/// 创建 dinner_provider 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_dinner_provider))
}

/// GET /api/dinner_provider - 获取所有供餐点列表（所有权限 0-3 都可以访问）
async fn get_dinner_provider(State(state): State<AppState>) -> Protobuf<DinnerProviderResponse> {
    let db = state.database.clone();

    // 查询所有供餐点
    let dinner_providers = match dinner_provider_entity::Entity::find()
        .all(db.as_ref())
        .await
    {
        Ok(n) => n,
        Err(err) => {
            return Protobuf(DinnerProviderResponse {
                dinner_providers: vec![],
                code: 500,
                message: format!("Database error: {}", err),
            });
        }
    };

    // 转换为 proto 格式并返回
    let proto_dinner_providers: Vec<ProtoDinnerProvider> = dinner_providers
        .into_iter()
        .map(|s| ProtoDinnerProvider {
            id: s.id,
            name: s.name.unwrap_or_default(),
            address: s.address.unwrap_or_default(),
            phone: s.phone.unwrap_or_default(),
            latitude: s.latitude.unwrap_or_default(),
            longitude: s.longitude.unwrap_or_default(),
            service_time: s.service_time.unwrap_or_default(),
            bonus_info: s.bonus_info.unwrap_or_default(),
            meal_style: s.meal_style.unwrap_or_default(),
            create_time: s.create_time.and_utc().timestamp(),
        })
        .collect();

    Protobuf(DinnerProviderResponse {
        dinner_providers: proto_dinner_providers,
        code: 200,
        message: "Get dinner provider list success".to_string(),
    })
}

