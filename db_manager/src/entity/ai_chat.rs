//! `SeaORM` Entity for AI Chat
//!
//! 表结构：
//! - id: 主键，自增
//! - session_id: 对话会话ID（UUID格式）
//! - openid: 用户微信openid
//! - title: 对话标题
//! - user_message: 用户发送的消息
//! - ai_response: AI回复的内容
//! - message_type: 消息类型（0=文本，1=图片，2=语音）
//! - created_at: 创建时间
//! - updated_at: 更新时间

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(schema_name = "public", table_name = "ai_chat")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    /// 对话会话ID，一个对话多个来回共用
    pub session_id: String,

    /// 用户openid
    pub openid: String,

    /// 对话标题
    pub title: Option<String>,

    /// 用户发送的消息内容
    pub user_message: Option<String>,

    /// AI回复的内容
    pub ai_response: Option<String>,

    /// 消息类型：0=文本, 1=图片, 2=语音
    pub message_type: i32,

    /// 创建时间
    pub created_at: DateTimeWithTimeZone,

    /// 更新时间
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

/// 消息类型枚举
pub mod message_type {
    /// 文本消息
    pub const TEXT: i32 = 0;
    /// 图片消息
    pub const IMAGE: i32 = 1;
    /// 语音消息
    pub const VOICE: i32 = 2;
}
