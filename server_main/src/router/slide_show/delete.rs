use axum::{
    Router,
    extract::{Query, State},
    http::HeaderMap,
    routing::delete,
};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::slideshow as slideshow_entity;
use interface_types::proto::slideshow::SlideshowResponse;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;
use user_auth::db_exchange::{ExchangeError, token2user};
use user_auth::user_auth::UserPermissionLevel;

use crate::AppState;

/// 创建 slide_show 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", delete(delete_slideshow))
}

/// 查询参数
#[derive(Debug, Deserialize)]
struct SlideShowParams {
    /// slideshow 的索引字符串
    index: String,
}

/// DELETE /api/slide_show?index=xxx - 删除 slideshow（仅 Admin 权限可以访问）
async fn delete_slideshow(
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

    // 3) 权限校验：只有 Admin (permission=3) 才能删除 slideshow
    let user_permission = auth_user.permission.unwrap_or(0);
    if user_permission != UserPermissionLevel::Admin.level() {
        return Protobuf(SlideshowResponse {
            slideshows: vec![],
            code: 403,
            message: "Permission denied: Only Admin can delete slideshow".to_string(),
        });
    }

    // 4) 查找要删除的 slideshow
    let db = state.database.clone();
    let slideshow_to_delete = match slideshow_entity::Entity::find()
        .filter(slideshow_entity::Column::Index.eq(&params.index))
        .one(db.as_ref())
        .await
    {
        Ok(Some(s)) => s,
        Ok(None) => {
            return Protobuf(SlideshowResponse {
                slideshows: vec![],
                code: 404,
                message: format!("Slideshow with index '{}' not found", params.index),
            });
        }
        Err(err) => {
            return Protobuf(SlideshowResponse {
                slideshows: vec![],
                code: 500,
                message: format!("Database error: {}", err),
            });
        }
    };

    // 5) 执行删除
    match slideshow_entity::Entity::delete_by_id(slideshow_to_delete.id)
        .exec(db.as_ref())
        .await
    {
        Ok(_) => {}
        Err(err) => {
            return Protobuf(SlideshowResponse {
                slideshows: vec![],
                code: 500,
                message: format!("Failed to delete slideshow: {}", err),
            });
        }
    };

    // 6) 返回成功响应
    Protobuf(SlideshowResponse {
        slideshows: vec![],
        code: 200,
        message: "Delete slideshow success".to_string(),
    })
}
