pub mod delete;
pub mod get;
pub mod insert;
pub mod modify;

use axum::Router;

/// 创建 medical_service 路由
///
/// 路由定义：
/// - GET /api/medical_service: 获取所有医疗服务列表（所有权限 0-3 都可以访问）
/// - POST /api/medical_service: 新增医疗服务（仅 Admin 权限）
/// - DELETE /api/medical_service?id=xxx: 删除医疗服务（仅 Admin 权限，id 通过查询参数传递）
/// - PUT /api/medical_service?id=xxx: 修改医疗服务（仅 Admin 权限，id 通过查询参数传递，其他字段通过 proto body 传递）
pub fn medical_service_router() -> Router<crate::AppState> {
    get::router()
        .merge(insert::router())
        .merge(delete::router())
        .merge(modify::router())
}

