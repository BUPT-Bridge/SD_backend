//! Health Guide Type 路由模块
//!
//! 提供健康指南类型的 CRUD 接口：
//! - GET /api/health_guide_type - 获取所有健康指南类型列表（所有权限均可访问）
//! - POST /api/health_guide_type - 创建新的健康指南类型（仅 Admin 权限）
//! - PUT /api/health_guide_type?id=xxx - 修改指定的健康指南类型（仅 Admin 权限）
//! - DELETE /api/health_guide_type?id=xxx - 删除指定的健康指南类型（仅 Admin 权限）

mod delete;
mod get;
mod modify;
mod post;

use axum::Router;

use crate::AppState;

/// 创建并返回 health_guide_type 的完整路由
///
/// 此函数合并了所有 health_guide_type 相关的子路由：
/// - get::router() - 获取列表
/// - post::router() - 创建
/// - modify::router() - 修改
/// - delete::router() - 删除
pub fn health_guide_type_router() -> Router<AppState> {
    Router::new()
        .merge(get::router())
        .merge(post::router())
        .merge(modify::router())
        .merge(delete::router())
}
