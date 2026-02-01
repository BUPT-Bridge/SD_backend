use axum::{Router, extract::State, routing::get};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::slideshow as slideshow_entity;
use interface_types::proto::slideshow::{Slideshow as ProtoSlideshow, SlideshowResponse};
use sea_orm::EntityTrait;

use crate::AppState;

/// 创建 slide_show 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_slideshow))
}

/// GET /api/slide_show - 获取所有 slideshow（所有权限 0-3 都可以访问）
async fn get_slideshow(State(state): State<AppState>) -> Protobuf<SlideshowResponse> {
    let db = state.database.clone();

    // 查询所有 slideshow
    let slideshows = match slideshow_entity::Entity::find().all(db.as_ref()).await {
        Ok(n) => n,
        Err(err) => {
            return Protobuf(SlideshowResponse {
                slideshows: vec![],
                code: 500,
                message: format!("Database error: {}", err),
            });
        }
    };

    // 转换为 proto 格式并返回
    let proto_slideshows: Vec<ProtoSlideshow> = slideshows
        .into_iter()
        .map(|s| ProtoSlideshow {
            id: s.id,
            index: s.index.unwrap_or_default(),
            create_time: s.create_time.and_utc().timestamp(),
        })
        .collect();

    Protobuf(SlideshowResponse {
        slideshows: proto_slideshows,
        code: 200,
        message: "Get slideshow list success".to_string(),
    })
}
