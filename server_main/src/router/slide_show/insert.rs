use axum::{
    Router,
    extract::{Query, State},
    http::HeaderMap,
    routing::post,
};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::slideshow as slideshow_entity;
use interface_types::proto::slideshow::SlideshowResponse;
use sea_orm::{ActiveModelTrait, Set};
use serde::Deserialize;
use user_auth::db_exchange::{ExchangeError, token2user};
use user_auth::user_auth::UserPermissionLevel;

use crate::AppState;

/// 创建 slide_show 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", post(insert_slideshow))
}

/// 查询参数
#[derive(Debug, Deserialize)]
struct SlideShowParams {
    /// slideshow 的索引字符串
    index: String,
}

/// POST /api/slide_show?index=xxx - 新增 slideshow（仅 Admin 权限可以访问）
async fn insert_slideshow(
    State(state): State<AppState>,
    Query(params): Query<SlideShowParams>,
    headers: HeaderMap,
) -> Protobuf<SlideshowResponse> {
    // 1) 从 Header 提取 token
    let token: &str = match headers.get("Authorization") {
        Some(t) => match t.to_str() {
            Ok(s) => s,
            Err(_) => {
                return Protobuf(SlideshowResponse {
                    slideshows: vec![],
                    code: 401,
                    message: "Invalid token format".to_string(),
                });
            }
        },
        None => {
            return Protobuf(SlideshowResponse {
                slideshows: vec![],
                code: 401,
                message: "Missing token".to_string(),
            });
        }
    };

    // 2) 解析 token，获取用户信息
    let auth_user = match token2user(&token) {
        Ok(u) => u,
        Err(err) => {
            let msg = match err {
                ExchangeError::InvalidToken => "Invalid token".to_string(),
                ExchangeError::TokenExpired => "Token expired".to_string(),
                ExchangeError::TokenGenerationError(e) | ExchangeError::OtherError(e) => e,
            };
            return Protobuf(SlideshowResponse {
                slideshows: vec![],
                code: 401,
                message: msg,
            });
        }
    };

    // 3) 权限校验：只有 Admin (permission=3) 才能新增 slideshow
    let user_permission = auth_user.permission.unwrap_or(0);
    if user_permission != UserPermissionLevel::Admin.level() {
        return Protobuf(SlideshowResponse {
            slideshows: vec![],
            code: 403,
            message: "Permission denied: Only Admin can insert slideshow".to_string(),
        });
    }

    // 4) 创建新的 ActiveModel 并插入
    let db = state.database.clone();
    let new_slideshow = slideshow_entity::ActiveModel {
        index: Set(Some(params.index)),
        ..Default::default()
    };

    // 5) 执行插入
    let inserted_slideshow = match new_slideshow.insert(db.as_ref()).await {
        Ok(n) => n,
        Err(err) => {
            return Protobuf(SlideshowResponse {
                slideshows: vec![],
                code: 500,
                message: format!("Failed to insert slideshow: {}", err),
            });
        }
    };

    // 6) 返回新增的 slideshow
    use interface_types::proto::slideshow::Slideshow as ProtoSlideshow;
    Protobuf(SlideshowResponse {
        slideshows: vec![ProtoSlideshow {
            id: inserted_slideshow.id,
            index: inserted_slideshow.index.unwrap_or_default(),
            create_time: inserted_slideshow.create_time.and_utc().timestamp(),
        }],
        code: 200,
        message: "Insert slideshow success".to_string(),
    })
}
