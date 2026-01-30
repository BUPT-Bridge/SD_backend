//! 多媒体文件路由模块
//!
//! 提供多媒体文件的上传和查询功能
//! - POST /api/mutil_media: 上传多媒体文件（multipart/form-data 格式），返回 MediaResponse（包含 UUID 和类型）
//! - GET /api/mutil_media/metadata?uuid=xxx: 获取多媒体文件的元数据（MediaResponse，protobuf 格式）
//! - GET /api/mutil_media/download?uuid=xxx: 下载多媒体文件的二进制数据

pub mod get;
pub mod post;
pub mod utils;

use axum::Router;

/// 创建 mutil_media 路由
///
/// 路由定义：
/// - POST /api/mutil_media: 上传多媒体文件（multipart/form-data 格式）
/// - GET /api/mutil_media/metadata?uuid=xxx: 获取多媒体文件的元数据（protobuf 格式）
/// - GET /api/mutil_media/download?uuid=xxx: 下载多媒体文件的二进制数据
pub fn mutil_media_router() -> Router<crate::AppState> {
    get::router().merge(post::router())
}
