pub mod delete;
pub mod get;
pub mod insert;
pub mod modify;

use axum::Router;

/// 创建 detail_meal 路由
///
/// 路由定义：
/// - GET /api/detail_meal: 获取明细餐列表（所有权限 0-3 都可以访问，支持查询参数筛选）
/// - POST /api/detail_meal: 新增明细餐（仅 Provider/Admin 权限）
/// - PUT /api/detail_meal?id=xxx: 修改明细餐（仅 Provider/Admin 权限，id 通过查询参数传递）
/// - DELETE /api/detail_meal?id=xxx: 删除明细餐（仅 Provider/Admin 权限，id 通过查询参数传递）
pub fn detail_meal_router() -> Router<crate::AppState> {
    get::router()
        .merge(insert::router())
        .merge(modify::router())
        .merge(delete::router())
}
