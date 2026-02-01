use axum::{Router, extract::State, http::HeaderMap, routing::post};
use axum_extra::protobuf::Protobuf;
use db_manager::entity::feedback as feedback_entity;
use interface_types::proto::feedback::{
    Feedback as ProtoFeedback, FeedbackRequest, FeedbackResponse,
};
use sea_orm::{ActiveModelTrait, Set};
use user_auth::db_exchange::{ExchangeError, token2user};

use crate::AppState;

/// 创建 feedback 路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", post(insert_feedback))
}

/// POST /api/feedback - 新增反馈（所有权限 0-3 都可以访问）
async fn insert_feedback(
    State(state): State<AppState>,
    headers: HeaderMap,
    Protobuf(payload): Protobuf<FeedbackRequest>,
) -> Protobuf<FeedbackResponse> {
    // 1) 从 Header 提取 token
    let token: &str = match headers.get("Authorization") {
        Some(t) => match t.to_str() {
            Ok(s) => s,
            Err(_) => {
                return Protobuf(FeedbackResponse {
                    feedback: None,
                    code: 401,
                    message: "Invalid token format".to_string(),
                });
            }
        },
        None => {
            return Protobuf(FeedbackResponse {
                feedback: None,
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
            return Protobuf(FeedbackResponse {
                feedback: None,
                code: 401,
                message: msg,
            });
        }
    };

    // 3) 权限校验：所有权限 0-3 都可以访问
    let user_permission = auth_user.permission.unwrap_or(0);
    if user_permission < 0 || user_permission > 3 {
        return Protobuf(FeedbackResponse {
            feedback: None,
            code: 403,
            message: "Permission denied: Invalid permission level".to_string(),
        });
    }

    // 4) 创建新的 ActiveModel 并插入
    let db = state.database.clone();
    let new_feedback = feedback_entity::ActiveModel {
        r#type: Set(if payload.r#type.is_empty() {
            None
        } else {
            Some(payload.r#type)
        }),
        content: Set(if payload.content.is_empty() {
            None
        } else {
            Some(payload.content)
        }),
        phone: Set(payload.phone),
        ..Default::default()
    };

    // 5) 执行插入
    let inserted_feedback = match new_feedback.insert(db.as_ref()).await {
        Ok(n) => n,
        Err(err) => {
            return Protobuf(FeedbackResponse {
                feedback: None,
                code: 500,
                message: format!("Failed to insert feedback: {}", err),
            });
        }
    };

    // 6) 返回新增的 feedback
    Protobuf(FeedbackResponse {
        feedback: Some(ProtoFeedback {
            id: inserted_feedback.id,
            r#type: inserted_feedback.r#type.unwrap_or_default(),
            content: inserted_feedback.content.unwrap_or_default(),
            phone: inserted_feedback.phone,
            created_time: inserted_feedback.created_time.timestamp(),
        }),
        code: 200,
        message: "Insert feedback success".to_string(),
    })
}
