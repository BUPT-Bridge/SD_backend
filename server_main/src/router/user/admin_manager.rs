use axum::{Router, extract::State, http::HeaderMap, routing::get};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::user as user_entity;
use interface_types::proto::user::{
    AdminManagedUser as ProtoAdminManagedUser, AdminManagerResponse,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use user_auth::db_exchange::{ExchangeError, token2user};
use user_auth::user_auth::UserPermissionLevel;

use crate::AppState;

/// 创建 admin_manager 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/admin_manager", get(admin_manager))
}

/// GET /api/user/admin_manager
/// 仅 Admin 权限（permission=3）可以获取所有 permission 为 2 和 3 的用户信息
async fn admin_manager(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Protobuf<AdminManagerResponse> {
    // 1) 从 Header 提取 token
    let token: &str = match headers.get("Authorization") {
        Some(t) => match t.to_str() {
            Ok(s) => s,
            Err(_) => {
                return Protobuf(AdminManagerResponse {
                    users: vec![],
                    code: 401,
                    message: "Invalid token format".to_string(),
                });
            }
        },
        None => {
            return Protobuf(AdminManagerResponse {
                users: vec![],
                code: 401,
                message: "Missing token".to_string(),
            });
        }
    };

    // 2) 解析 token，获取用户信息
    let auth_user = match token2user(&token) {
        Ok(u) => u,
        Err(err) => {
            let msg = match err {
                ExchangeError::InvalidToken => "Invalid token".to_string(),
                ExchangeError::TokenExpired => "Token expired".to_string(),
                ExchangeError::TokenGenerationError(e) | ExchangeError::OtherError(e) => e,
            };
            return Protobuf(AdminManagerResponse {
                users: vec![],
                code: 401,
                message: msg,
            });
        }
    };

    // 3) 权限校验：只有 Admin (permission=3) 才能访问
    let user_permission = auth_user.permission.unwrap_or(0);
    if user_permission != UserPermissionLevel::Admin.level() {
        return Protobuf(AdminManagerResponse {
            users: vec![],
            code: 403,
            message: "Permission denied: Only Admin can access this endpoint".to_string(),
        });
    }

    // 4) 查询数据库中所有 permission 为 2 或 3 的用户
    let db = state.database.clone();
    let users = match user_entity::Entity::find()
        .filter(
            user_entity::Column::Permission
                .eq(UserPermissionLevel::Provider.level())
                .or(user_entity::Column::Permission.eq(UserPermissionLevel::Admin.level())),
        )
        .all(db.as_ref())
        .await
    {
        Ok(users) => users,
        Err(err) => {
            return Protobuf(AdminManagerResponse {
                users: vec![],
                code: 500,
                message: format!("Database error: {}", err),
            });
        }
    };

    // 5) 将数据库模型转换为 Proto 模型
    let proto_users: Vec<ProtoAdminManagedUser> = users
        .into_iter()
        .map(|user| ProtoAdminManagedUser {
            open_id: Some(user.open_id),
            nickname: user.nickname,
            name: user.name,
            phone_number: user.phone_number,
            address: user.address,
            is_important: user.is_important.map(|b| b.to_string()),
            avatar: user.avatar,
            permission: user.permission.map(|p| p.to_string()),
        })
        .collect();

    Protobuf(AdminManagerResponse {
        users: proto_users,
        code: 200,
        message: "Get admin managed users success".to_string(),
    })
}
