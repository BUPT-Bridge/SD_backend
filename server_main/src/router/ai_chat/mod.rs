pub mod delete;
pub mod get;
pub mod post;

use axum::Router;

/// 创建 ai_chat 路由
///
/// 路由定义：
/// - GET  /api/ai_chat/session?session_id=xxx: 获取单个会话详情（包含所有消息）
/// - GET  /api/ai_chat/sessions?page=1&page_size=20: 获取会话列表
/// - POST /api/ai_chat/save: 保存聊天记录（需要token）
/// - DELETE /api/ai_chat/session?session_id=xxx: 删除整个会话（需要token）
pub fn ai_chat_router() -> Router<crate::AppState> {
    get::router().merge(post::router()).merge(delete::router())
}
