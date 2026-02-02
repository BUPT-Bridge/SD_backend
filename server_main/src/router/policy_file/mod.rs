//! Policy File 路由模块
//!
//! 提供政策文件的 CRUD 接口：
//! - GET /api/policy_file?type=xxx - 获取指定类型的政策文件列表（所有权限均可访问，必须提供 type 参数）
//! - POST /api/policy_file - 创建新的政策文件（仅 Admin 权限，id 和 create_time 由数据库自动处理）
//! - PUT /api/policy_file?id=xxx - 修改指定的政策文件（仅 Admin 权限，通过 id 查找）
//! - DELETE /api/policy_file?id=xxx - 删除指定的政策文件（仅 Admin 权限，通过 id 查找）
//!
//! ## 参数说明
//! - GET 接口：必须提供  参数（字符串类型）
//! - PUT/DELETE 接口：必须提供 uid=1000(zhao) gid=1000(zhao) groups=1000(zhao),4(adm),10(uucp),20(dialout),24(cdrom),25(floppy),27(sudo),29(audio),30(dip),44(video),46(plugdev),116(netdev),998(ollama),999(docker) 参数（整数类型）

mod delete;
mod get;
mod modify;
mod post;

use axum::Router;

use crate::AppState;

/// 创建并返回 policy_file 的完整路由
///
/// 此函数合并了所有 policy_file 相关的子路由：
/// - get::router() - 获取列表（需要 type 参数）
/// - post::router() - 创建
/// - modify::router() - 修改（需要 id 参数）
/// - delete::router() - 删除（需要 id 参数）
pub fn policy_file_router() -> Router<AppState> {
    Router::new()
        .merge(get::router())
        .merge(post::router())
        .merge(modify::router())
        .merge(delete::router())
}
