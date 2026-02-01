pub mod delete;
pub mod get;
pub mod insert;
pub mod modify;

use axum::Router;

/// 创建 dinner_provider 路由
///
/// 路由定义：
/// - GET /api/dinner_provider: 获取所有供餐点列表（所有权限 0-3 都可以访问）
/// - POST /api/dinner_provider: 新增供餐点（仅 Admin 权限）
/// - DELETE /api/dinner_provider?id=xxx: 删除供餐点（仅 Admin 权限，id 通过查询参数传递）
/// - PUT /api/dinner_provider?id=xxx: 修改供餐点（仅 Manager/Admin 权限，id 通过查询参数传递，其他字段通过 proto body 传递）
pub fn dinner_provider_router() -> Router<crate::AppState> {
    get::router()
        .merge(insert::router())
        .merge(delete::router())
        .merge(modify::router())
}
