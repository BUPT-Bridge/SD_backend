use axum::{
    Router,
    extract::{Query, State},
    http::HeaderMap,
    routing::get,
};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::ai_chat as ai_chat_entity;
use interface_types::proto::ai_chat::{
    AiChatMessage, AiChatSession, GetSessionListResponse, GetSessionResponse,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use std::collections::HashMap;

use crate::AppState;

/// 创建 GET 路由
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/session", get(get_session_detail))
        .route("/sessions", get(get_session_list))
}

/// 查询参数：会话详情
#[derive(serde::Deserialize)]
pub struct SessionDetailQuery {
    session_id: String,
}

/// GET /api/ai_chat/session?session_id=xxx
/// 获取单个会话详情（包含该会话下的所有消息）
async fn get_session_detail(
    State(state): State<AppState>,
    Query(query): Query<SessionDetailQuery>,
) -> Protobuf<GetSessionResponse> {
    let db = state.database.clone();

    // 验证 session_id
    if query.session_id.is_empty() {
        return Protobuf(GetSessionResponse {
            session_id: String::new(),
            title: String::new(),
            openid: String::new(),
            messages: vec![],
            total_messages: 0,
            created_at: 0,
            last_message_at: 0,
            code: 400,
            message: "Session ID is required".to_string(),
        });
    }

    // 查询该会话的所有消息
    let messages = match ai_chat_entity::Entity::find()
        .filter(ai_chat_entity::Column::SessionId.eq(query.session_id.clone()))
        .order_by_asc(ai_chat_entity::Column::CreatedAt) // 按时间升序，最早的在前
        .all(db.as_ref())
        .await
    {
        Ok(n) => n,
        Err(err) => {
            return Protobuf(GetSessionResponse {
                session_id: query.session_id,
                title: String::new(),
                openid: String::new(),
                messages: vec![],
                total_messages: 0,
                created_at: 0,
                last_message_at: 0,
                code: 500,
                message: format!("Database error: {}", err),
            });
        }
    };

    if messages.is_empty() {
        return Protobuf(GetSessionResponse {
            session_id: query.session_id,
            title: String::new(),
            openid: String::new(),
            messages: vec![],
            total_messages: 0,
            created_at: 0,
            last_message_at: 0,
            code: 404,
            message: "Session not found".to_string(),
        });
    }

    // 提取会话信息
    let first_msg = messages.first().unwrap();
    let last_msg = messages.last().unwrap();
    let session_id = first_msg.session_id.clone();
    let openid = first_msg.openid.clone();
    let title = first_msg
        .title
        .clone()
        .unwrap_or_else(|| "新对话".to_string());
    let created_at = first_msg.created_at.timestamp();
    let last_message_at = last_msg.created_at.timestamp();
    let total_messages = messages.len() as i32;

    // 转换为 proto 消息列表
    let proto_messages: Vec<AiChatMessage> = messages
        .into_iter()
        .map(|m| AiChatMessage {
            id: m.id,
            session_id: m.session_id,
            openid: m.openid,
            title: m.title.unwrap_or_default(),
            user_message: m.user_message.unwrap_or_default(),
            ai_response: m.ai_response.unwrap_or_default(),
            message_type: m.message_type,
            created_at: m.created_at.timestamp(),
            updated_at: m.updated_at.timestamp(),
        })
        .collect();

    Protobuf(GetSessionResponse {
        session_id,
        title,
        openid,
        messages: proto_messages,
        total_messages,
        created_at,
        last_message_at,
        code: 200,
        message: "Get session detail success".to_string(),
    })
}

/// 查询参数：会话列表
#[derive(serde::Deserialize)]
pub struct SessionListQuery {
    page: Option<i32>,
    page_size: Option<i32>,
}

/// GET /api/ai_chat/sessions?page=1&page_size=20
/// 获取用户的会话列表（按会话分组）
async fn get_session_list(
    State(state): State<AppState>,
    Query(query): Query<SessionListQuery>,
    headers: HeaderMap,
) -> Protobuf<GetSessionListResponse> {
    let db = state.database.clone();
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);

    // 从 token 获取用户信息
    let openid = match extract_openid_from_headers(&headers).await {
        Ok(id) => id,
        Err(resp) => return Protobuf(resp),
    };

    // 查询该用户的所有消息，按时间排序
    let messages = match ai_chat_entity::Entity::find()
        .filter(ai_chat_entity::Column::Openid.eq(openid.clone()))
        .order_by_desc(ai_chat_entity::Column::CreatedAt)
        .all(db.as_ref())
        .await
    {
        Ok(n) => n,
        Err(err) => {
            return Protobuf(GetSessionListResponse {
                sessions: vec![],
                total: 0,
                code: 500,
                message: format!("Database error: {}", err),
            });
        }
    };

    // 按 session_id 分组统计
    let mut session_map: HashMap<String, (i32, i64, i64, String)> = HashMap::new();
    for msg in &messages {
        let entry = session_map.entry(msg.session_id.clone()).or_insert((
            0,
            msg.created_at.timestamp(),
            0,
            String::new(),
        ));
        entry.0 += 1; // 消息计数
        entry.2 = msg.created_at.timestamp(); // 最后消息时间
        // 使用第一条消息的 title 作为会话标题
        if entry.3.is_empty() {
            if let Some(title) = &msg.title {
                entry.3 = title.clone();
            }
        }
    }

    // 转换为列表并排序（按最后消息时间降序）
    let mut sessions: Vec<AiChatSession> = session_map
        .into_iter()
        .map(
            |(session_id, (count, created_at, last_msg_at, title))| AiChatSession {
                session_id,
                openid: openid.clone(),
                title: if title.is_empty() {
                    "新对话".to_string()
                } else {
                    title
                },
                message_count: count,
                created_at,
                last_message_at: last_msg_at,
            },
        )
        .collect();

    // 按最后消息时间降序排序
    sessions.sort_by(|a, b| b.last_message_at.cmp(&a.last_message_at));

    let total = sessions.len();

    // 手动分页
    let start = ((page - 1) as usize).min(sessions.len());
    let end = (start + page_size as usize).min(sessions.len());
    let paged_sessions: Vec<AiChatSession> = sessions.drain(start..end).collect();

    Protobuf(GetSessionListResponse {
        sessions: paged_sessions,
        total: total as i32,
        code: 200,
        message: "Get session list success".to_string(),
    })
}

/// 从 headers 中提取 openid
async fn extract_openid_from_headers(
    headers: &HeaderMap,
) -> Result<String, GetSessionListResponse> {
    use user_auth::db_exchange::{ExchangeError, token2user};

    let token: &str = match headers.get("Authorization") {
        Some(t) => match t.to_str() {
            Ok(s) => s,
            Err(_) => {
                return Err(GetSessionListResponse {
                    sessions: vec![],
                    total: 0,
                    code: 401,
                    message: "Invalid token format".to_string(),
                });
            }
        },
        None => {
            return Err(GetSessionListResponse {
                sessions: vec![],
                total: 0,
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
            return Err(GetSessionListResponse {
                sessions: vec![],
                total: 0,
                code: 401,
                message: msg,
            });
        }
    };

    Ok(auth_user.open_id)
}
