//! Health Guide Content 路由模块
//!
//! 提供健康指南内容的 CRUD 接口：
//! - GET /api/health_guide_content?type_one=xxx&type_two=xxx - 获取健康指南内容（所有权限均可访问，必须提供 type_one 和 type_two 参数）
//! - POST /api/health_guide_content - 创建新的健康指南内容（仅 Admin 权限）
//! - PUT /api/health_guide_content?type_one=xxx&type_two=xxx - 修改指定的健康指南内容（仅 Admin 权限，通过 type_one 和 type_two 筛选）
//! - DELETE /api/health_guide_content?type_one=xxx&type_two=xxx - 删除指定的健康指南内容（仅 Admin 权限，通过 type_one 和 type_two 筛选）
//!
//! ## 参数说明
//! - `type_one`: 一级类型 ID（整数类型，匹配 HealthGuideType 的 id）
//! - `type_two`: 二级类型名称（字符串类型）
//!
//! 注意：GET、PUT、DELETE 接口的 type_one 和 type_two 参数为必填项，缺失将返回 400 错误

mod delete;
mod get;
mod modify;
mod post;

use axum::Router;

use crate::AppState;

/// 创建并返回 health_guide_content 的完整路由
///
/// 此函数合并了所有 health_guide_content 相关的子路由：
/// - get::router() - 获取内容（需要 type_one 和 type_two 参数）
/// - post::router() - 创建内容
/// - modify::router() - 修改内容（需要 type_one 和 type_two 参数）
/// - delete::router() - 删除内容（需要 type_one 和 type_two 参数）
pub fn health_guide_content_router() -> Router<AppState> {
    Router::new()
        .merge(get::router())
        .merge(post::router())
        .merge(modify::router())
        .merge(delete::router())
}

