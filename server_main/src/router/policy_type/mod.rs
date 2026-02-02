//! Policy Type 路由模块
//!
//! 提供政策类型的 CRUD 接口：
//! - GET /api/policy_type - 获取所有政策类型列表（所有权限均可访问）
//! - POST /api/policy_type - 创建新的政策类型（仅 Admin 权限）
//! - PUT /api/policy_type?id=xxx - 修改指定的政策类型（仅 Admin 权限）
//! - DELETE /api/policy_type?id=xxx - 删除指定的政策类型（仅 Admin 权限）

mod delete;
mod get;
mod modify;
mod post;

use axum::Router;

use crate::AppState;

/// 创建并返回 policy_type 的完整路由
///
/// 此函数合并了所有 policy_type 相关的子路由：
/// - get::router() - 获取列表
/// - post::router() - 创建
/// - modify::router() - 修改
/// - delete::router() - 删除
pub fn policy_type_router() -> Router<AppState> {
    Router::new()
        .merge(get::router())
        .merge(post::router())
        .merge(modify::router())
        .merge(delete::router())
}
