use axum::{
    Router,
    extract::{Query, State},
    http::HeaderMap,
    routing::delete,
};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::ai_chat as ai_chat_entity;
use interface_types::proto::ai_chat::ChatOperationResponse;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use user_auth::db_exchange::{ExchangeError, token2user};

use crate::AppState;

/// 创建 DELETE 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/session", delete(delete_session))
}

/// 查询参数结构
#[derive(serde::Deserialize)]
pub struct DeleteSessionQuery {
    session_id: String,
}

/// DELETE /api/ai_chat/session?session_id=xxx
/// 删除整个会话（删除该会话下的所有聊天记录）
async fn delete_session(
    State(state): State<AppState>,
    Query(query): Query<DeleteSessionQuery>,
    headers: HeaderMap,
) -> Protobuf<ChatOperationResponse> {
    // 1) 验证 token
    let openid = match extract_openid_from_headers(&headers).await {
        Ok(id) => id,
        Err(resp) => return Protobuf(resp),
    };

    let db = state.database.clone();

    // 2) 验证 session_id
    if query.session_id.is_empty() {
        return Protobuf(ChatOperationResponse {
            success: false,
            code: 400,
            message: "Session ID is required".to_string(),
        });
    }

    // 3) 删除该用户指定会话的所有消息（确保只能删除自己的会话）
    let result = ai_chat_entity::Entity::delete_many()
        .filter(ai_chat_entity::Column::SessionId.eq(query.session_id.clone()))
        .filter(ai_chat_entity::Column::Openid.eq(openid))
        .exec(db.as_ref())
        .await;

    match result {
        Ok(delete_result) => {
            if delete_result.rows_affected == 0 {
                Protobuf(ChatOperationResponse {
                    success: false,
                    code: 404,
                    message: "Session not found or already deleted".to_string(),
                })
            } else {
                Protobuf(ChatOperationResponse {
                    success: true,
                    code: 200,
                    message: format!(
                        "Session deleted successfully, {} messages removed",
                        delete_result.rows_affected
                    ),
                })
            }
        }
        Err(err) => Protobuf(ChatOperationResponse {
            success: false,
            code: 500,
            message: format!("Database error: {}", err),
        }),
    }
}

/// 从 headers 中提取 openid
async fn extract_openid_from_headers(headers: &HeaderMap) -> Result<String, ChatOperationResponse> {
    let token: &str = match headers.get("Authorization") {
        Some(t) => match t.to_str() {
            Ok(s) => s,
            Err(_) => {
                return Err(ChatOperationResponse {
                    success: false,
                    code: 401,
                    message: "Invalid token format".to_string(),
                });
            }
        },
        None => {
            return Err(ChatOperationResponse {
                success: false,
                code: 401,
                message: "Missing token".to_string(),
            });
        }
    };

    let auth_user = match token2user(token) {
        Ok(u) => u,
        Err(err) => {
            let msg = match err {
                ExchangeError::InvalidToken => "Invalid token".to_string(),
                ExchangeError::TokenExpired => "Token expired".to_string(),
                ExchangeError::TokenGenerationError(e) | ExchangeError::OtherError(e) => e,
            };
            return Err(ChatOperationResponse {
                success: false,
                code: 401,
                message: msg,
            });
        }
    };

    Ok(auth_user.open_id)
}
