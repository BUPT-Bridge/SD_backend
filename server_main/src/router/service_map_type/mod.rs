//! Service Map Type 路由模块
//!
//! 提供服务地图类型的 CRUD 接口：
//! - GET /api/service_map_type - 获取所有服务地图类型列表（所有权限均可访问）
//! - POST /api/service_map_type - 创建新的服务地图类型（仅 Admin 权限）
//! - PUT /api/service_map_type?id=xxx - 修改指定的服务地图类型（仅 Admin 权限）
//! - DELETE /api/service_map_type?id=xxx - 删除指定的服务地图类型（仅 Admin 权限）

mod delete;
mod get;
mod modify;
mod post;

use axum::Router;

use crate::AppState;

/// 创建并返回 service_map_type 的完整路由
///
/// 此函数合并了所有 service_map_type 相关的子路由：
/// - get::router() - 获取列表
/// - post::router() - 创建
/// - modify::router() - 修改
/// - delete::router() - 删除
pub fn service_map_type_router() -> Router<AppState> {
    Router::new()
        .merge(get::router())
        .merge(post::router())
        .merge(modify::router())
        .merge(delete::router())
}
