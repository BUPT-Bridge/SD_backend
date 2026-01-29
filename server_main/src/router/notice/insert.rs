use axum::{
    Router,
    extract::State,
    http::HeaderMap,
    routing::{get, post},
};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::notice as notice_entity;
use interface_types::proto::notice::{Notice as ProtoNotice, NoticeRequest, NoticeResponse};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use user_auth::db_exchange::{ExchangeError, token2user};
use user_auth::user_auth::UserPermissionLevel;

use crate::AppState;

/// 创建 notice 路由
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/insert", get(get_notice))
        .route("/insert", post(insert_notice))
}

/// GET /api/notice - 获取 notice（所有权限 0-3 都可以访问）
/// 返回数据库中的最后一个 notice
async fn get_notice(State(state): State<AppState>) -> Protobuf<NoticeResponse> {
    let db = state.database.clone();

    // 查询所有 notice
    let notices = match notice_entity::Entity::find().all(db.as_ref()).await {
        Ok(n) => n,
        Err(err) => {
            return Protobuf(NoticeResponse {
                notice: None,
                code: 500,
                message: format!("Database error: {}", err),
            });
        }
    };

    // 返回最后一个 notice
    if let Some(last_notice) = notices.last() {
        Protobuf(NoticeResponse {
            notice: Some(ProtoNotice {
                id: last_notice.id,
                content: last_notice.content.clone().unwrap_or_default(),
            }),
            code: 200,
            message: "Get notice success".to_string(),
        })
    } else {
        Protobuf(NoticeResponse {
            notice: None,
            code: 404,
            message: "No notice found".to_string(),
        })
    }
}

/// POST /api/notice - 新增 notice（仅 Admin 权限可以访问）
async fn insert_notice(
    State(state): State<AppState>,
    headers: HeaderMap,
    Protobuf(payload): Protobuf<NoticeRequest>,
) -> Protobuf<NoticeResponse> {
    // 1) 从 Header 提取 token
    let token: &str = match headers.get("Authorization") {
        Some(t) => match t.to_str() {
            Ok(s) => s,
            Err(_) => {
                return Protobuf(NoticeResponse {
                    notice: None,
                    code: 401,
                    message: "Invalid token format".to_string(),
                });
            }
        },
        None => {
            return Protobuf(NoticeResponse {
                notice: None,
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
            return Protobuf(NoticeResponse {
                notice: None,
                code: 401,
                message: msg,
            });
        }
    };

    // 3) 权限校验：只有 Admin (permission=3) 才能新增 notice
    let user_permission = auth_user.permission.unwrap_or(0);
    if user_permission != UserPermissionLevel::Admin.level() {
        return Protobuf(NoticeResponse {
            notice: None,
            code: 403,
            message: "Permission denied: Only Admin can insert notice".to_string(),
        });
    }

    // 4) 创建新的 ActiveModel 并插入
    let db = state.database.clone();
    let new_notice = notice_entity::ActiveModel {
        content: Set(Some(payload.content)),
        ..Default::default()
    };

    // 5) 执行插入
    let inserted_notice = match new_notice.insert(db.as_ref()).await {
        Ok(n) => n,
        Err(err) => {
            return Protobuf(NoticeResponse {
                notice: None,
                code: 500,
                message: format!("Failed to insert notice: {}", err),
            });
        }
    };

    // 6) 返回新增的 notice
    Protobuf(NoticeResponse {
        notice: Some(ProtoNotice {
            id: inserted_notice.id,
            content: inserted_notice.content.unwrap_or_default(),
        }),
        code: 200,
        message: "Insert notice success".to_string(),
    })
}
