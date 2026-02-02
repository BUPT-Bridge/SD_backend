use axum::{
    Router,
    extract::{Query, State},
    routing::get,
};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::health_guide_content as health_guide_content_entity;
use interface_types::proto::health_guide_content::{
    HealthGuideContent as ProtoHealthGuideContent, HealthGuideContentResponse,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;

use crate::AppState;

/// 创建 health_guide_content 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_health_guide_content))
}

/// 查询参数
#[derive(Debug, Deserialize)]
struct HealthGuideContentParams {
    /// 类型一（一级类型 ID）
    type_one: Option<i32>,
    /// 类型二（二级类型名称）
    type_two: Option<String>,
}

/// GET /api/health_guide_content?type_one=xxx&type_two=xxx - 获取健康指南内容（所有权限均可访问）
/// 必须提供 type_one 和 type_two 参数
async fn get_health_guide_content(
    State(state): State<AppState>,
    Query(params): Query<HealthGuideContentParams>,
) -> Protobuf<HealthGuideContentResponse> {
    // 1) 检查必填参数
    let type_one = match params.type_one {
        Some(v) => v,
        None => {
            return Protobuf(HealthGuideContentResponse {
                health_guide_contents: vec![],
                code: 400,
                message: "Missing required parameter: type_one".to_string(),
            });
        }
    };

    let type_two = match params.type_two {
        Some(v) if !v.is_empty() => v,
        _ => {
            return Protobuf(HealthGuideContentResponse {
                health_guide_contents: vec![],
                code: 400,
                message: "Missing required parameter: type_two".to_string(),
            });
        }
    };

    // 2) 查询符合条件的健康指南内容
    let db = state.database.clone();
    let health_guide_contents = match health_guide_content_entity::Entity::find()
        .filter(health_guide_content_entity::Column::TypeOne.eq(type_one))
        .filter(health_guide_content_entity::Column::TypeTwo.eq(type_two))
        .all(db.as_ref())
        .await
    {
        Ok(contents) => contents,
        Err(err) => {
            return Protobuf(HealthGuideContentResponse {
                health_guide_contents: vec![],
                code: 500,
                message: format!("Database error: {}", err),
            });
        }
    };

    // 3) 转换为 proto 格式
    let proto_contents: Vec<ProtoHealthGuideContent> = health_guide_contents
        .into_iter()
        .map(|c| ProtoHealthGuideContent {
            id: c.id,
            type_one: c.type_one.unwrap_or_default(),
            type_two: c.type_two.unwrap_or_default(),
            content: c.content.map(|json| json.to_string()).unwrap_or_default(),
        })
        .collect();

    Protobuf(HealthGuideContentResponse {
        health_guide_contents: proto_contents,
        code: 200,
        message: "Get health guide contents success".to_string(),
    })
}
