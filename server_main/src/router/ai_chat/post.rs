use axum::{Router, extract::State, http::HeaderMap, routing::post};
use axum_extra::protobuf::Protobuf;
use chrono::Utc;
use db_manager::entity::ai_chat as ai_chat_entity;
use interface_types::proto::ai_chat::{AiChatMessage, SaveChatRequest, SaveChatResponse};
use sea_orm::{ActiveModelTrait, Set};
use user_auth::db_exchange::{ExchangeError, token2user};

use crate::AppState;

/// 创建 POST 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/save", post(save_chat_message))
}

/// POST /api/ai_chat/save
/// 保存聊天记录（小程序在获得AI回复后调用）
///
/// # 请求说明：
/// - session_id: 新对话传空字符串，服务器会生成新的 session_id
/// - title: 新对话时必填，作为对话标题；已有对话可留空
/// - user_message: 用户发送的消息（必填）
/// - ai_response: AI回复的内容
/// - message_type: 消息类型（默认TEXT）
async fn save_chat_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    Protobuf(payload): Protobuf<SaveChatRequest>,
) -> Protobuf<SaveChatResponse> {
    // 1) 从 Header 提取 token 并验证
    let openid = match extract_openid_from_headers(&headers).await {
        Ok(id) => id,
        Err(resp) => return Protobuf(resp),
    };

    let db = state.database.clone();

    // 2) 验证消息内容
    if payload.user_message.is_empty() {
        return Protobuf(SaveChatResponse {
            message: None,
            session_id: String::new(),
            code: 400,
            message_info: "User message cannot be empty".to_string(),
        });
    }

    // 3) 处理 session_id：如果是新对话，生成新的 session_id
    let is_new_session = payload.session_id.is_empty();
    let session_id = if is_new_session {
        format!(
            "session_{}_{}",
            openid,
            chrono::Utc::now().timestamp_millis()
        )
    } else {
        payload.session_id
    };

    // 4) 处理 title：统一返回 Option<String>
    let title: Option<String> = if is_new_session {
        // 新对话：生成标题
        if payload.title.is_empty() {
            // 没有提供标题，使用用户消息的前20字
            Some(if payload.user_message.len() > 20 {
                format!("{}...", &payload.user_message[..20])
            } else {
                payload.user_message.clone()
            })
        } else {
            // 使用传入的标题
            Some(payload.title)
        }
    } else {
        // 已有对话：如果传了 title 就更新，否则 None（保持原有）
        if payload.title.is_empty() {
            None
        } else {
            Some(payload.title)
        }
    };

    // 5) 创建新的 ActiveModel
    let now = Utc::now();
    let new_chat = ai_chat_entity::ActiveModel {
        session_id: Set(session_id.clone()),
        openid: Set(openid),
        title: Set(title),
        user_message: Set(Some(payload.user_message)),
        ai_response: Set(Some(payload.ai_response)),
        message_type: Set(payload.message_type as i32),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        ..Default::default()
    };

    // 6) 执行插入
    let inserted = match new_chat.insert(db.as_ref()).await {
        Ok(n) => n,
        Err(err) => {
            return Protobuf(SaveChatResponse {
                message: None,
                session_id: session_id.clone(),
                code: 500,
                message_info: format!("Failed to save chat: {}", err),
            });
        }
    };

    // 7) 返回保存的消息
    Protobuf(SaveChatResponse {
        message: Some(AiChatMessage {
            id: inserted.id,
            session_id: inserted.session_id,
            openid: inserted.openid,
            title: inserted.title.unwrap_or_default(),
            user_message: inserted.user_message.unwrap_or_default(),
            ai_response: inserted.ai_response.unwrap_or_default(),
            message_type: inserted.message_type,
            created_at: inserted.created_at.timestamp(),
            updated_at: inserted.updated_at.timestamp(),
        }),
        session_id: session_id.clone(),
        code: 200,
        message_info: "Save chat message success".to_string(),
    })
}

/// 从 headers 中提取 openid
async fn extract_openid_from_headers(headers: &HeaderMap) -> Result<String, SaveChatResponse> {
    let token: &str = match headers.get("Authorization") {
        Some(t) => match t.to_str() {
            Ok(s) => s,
            Err(_) => {
                return Err(SaveChatResponse {
                    message: None,
                    session_id: String::new(),
                    code: 401,
                    message_info: "Invalid token format".to_string(),
                });
            }
        },
        None => {
            return Err(SaveChatResponse {
                message: None,
                session_id: String::new(),
                code: 401,
                message_info: "Missing token".to_string(),
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
            return Err(SaveChatResponse {
                message: None,
                session_id: String::new(),
                code: 401,
                message_info: msg,
            });
        }
    };

    Ok(auth_user.open_id)
}
