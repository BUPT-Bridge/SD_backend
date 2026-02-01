pub mod delete;
pub mod get;
pub mod insert;

use axum::Router;

/// 创建 slide_show 路由
///
/// 路由定义：
/// - POST /api/slide_show?index=xxx: 新增 slideshow（仅 Admin 权限）
/// - DELETE /api/slide_show?index=xxx: 删除 slideshow（仅 Admin 权限）
/// - GET /api/slide_show: 获取所有 slideshow 列表（所有权限）
pub fn slide_show_router() -> Router<crate::AppState> {
    get::router()
        .merge(insert::router())
        .merge(delete::router())
}
