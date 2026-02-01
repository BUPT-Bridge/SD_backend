pub mod delete;
pub mod get;
pub mod insert;
pub mod modify;

use axum::Router;

/// 创建 community_service 路由
///
/// 路由定义：
/// - GET /api/community_service: 获取所有社区服务列表（所有权限 0-3 都可以访问）
/// - POST /api/community_service: 新增社区服务（仅 Admin 权限）
/// - DELETE /api/community_service?id=xxx: 删除社区服务（仅 Admin 权限，id 通过查询参数传递）
/// - PUT /api/community_service?id=xxx: 修改社区服务（仅 Admin 权限，id 通过查询参数传递，其他字段通过 proto body 传递）
pub fn community_service_router() -> Router<crate::AppState> {
    get::router()
        .merge(insert::router())
        .merge(delete::router())
        .merge(modify::router())
}
