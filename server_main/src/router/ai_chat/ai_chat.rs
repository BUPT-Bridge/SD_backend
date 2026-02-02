use axum::{Router, extract::State, http::HeaderMap, routing::post};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::ai_chat as ai_chat_entity;
use interface_types::proto::ai_chat::{AiChat as ProtoAiChat, AiChatRequest, AiChatResponse};
use sea_orm::{ActiveModelTrait, Set};
use user_auth::db_exchange::{ExchangeError, token2user};

use crate::AppState;

/// 创建 ai_chat 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", post(insert_ai_chat))
}

/// POST /api/ai_chat - 新增 AI 聊天记录（所有权限 0-3 都可以访问）
async fn insert_ai_chat(
    State(state): State<AppState>,
    headers: HeaderMap,
    Protobuf(payload): Protobuf<AiChatRequest>,
) -> Protobuf<AiChatResponse> {
    // 1) 从 Header 提取 token
    let token: &str = match headers.get("Authorization") {
        Some(t) => match t.to_str() {
            Ok(s) => s,
            Err(_) => {
                return Protobuf(AiChatResponse {
                    ai_chat: None,
                    code: 401,
                    message: "Invalid token format".to_string(),
                });
            }
        },
        None => {
            return Protobuf(AiChatResponse {
                ai_chat: None,
                code: 401,
                message: "Missing token".to_string(),
            });
        }
    };

    // 2) 解析 token，获取用户信息（从 token 中获取 openid）
    let auth_user = match token2user(&token) {
        Ok(u) => u,
        Err(err) => {
            let msg = match err {
                ExchangeError::InvalidToken => "Invalid token".to_string(),
                ExchangeError::TokenExpired => "Token expired".to_string(),
                ExchangeError::TokenGenerationError(e) | ExchangeError::OtherError(e) => e,
            };
            return Protobuf(AiChatResponse {
                ai_chat: None,
                code: 401,
                message: msg,
            });
        }
    };

    // 3) 权限校验：所有权限 0-3 都可以访问
    let user_permission = auth_user.permission.unwrap_or(0);
    if user_permission < 0 || user_permission > 3 {
        return Protobuf(AiChatResponse {
            ai_chat: None,
            code: 403,
            message: "Permission denied: Invalid permission level".to_string(),
        });
    }

    // 4) 创建新的 ActiveModel 并插入（openid 从 token 中获取）
    let db = state.database.clone();
    let new_ai_chat = ai_chat_entity::ActiveModel {
        index: Set(payload.index),
        openid: Set(Some(auth_user.open_id)),
        long_content: Set(payload.long_content),
        ..Default::default()
    };

    // 5) 执行插入
    let inserted_ai_chat = match new_ai_chat.insert(db.as_ref()).await {
        Ok(n) => n,
        Err(err) => {
            return Protobuf(AiChatResponse {
                ai_chat: None,
                code: 500,
                message: format!("Failed to insert ai_chat: {}", err),
            });
        }
    };

    // 6) 返回新增的 ai_chat（不包含 long_content 和 openid）
    Protobuf(AiChatResponse {
        ai_chat: Some(ProtoAiChat {
            id: inserted_ai_chat.id,
            index: inserted_ai_chat.index,
        }),
        code: 200,
        message: "Insert ai_chat success".to_string(),
    })
}
