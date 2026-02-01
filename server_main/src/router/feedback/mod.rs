pub mod export;
pub mod insert;

use axum::Router;

/// 创建 feedback 路由
///
/// 路由定义：
/// - POST /api/feedback: 新增反馈（所有权限 0-3 都可以访问）
/// - GET /api/feedback/export: 导出反馈（所有权限 0-3 都可以访问）
pub fn feedback_router() -> Router<crate::AppState> {
    insert::router().merge(export::router())
}
