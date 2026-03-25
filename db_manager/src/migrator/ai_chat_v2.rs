use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        let path = file!();
        std::path::Path::new(path)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. 如果旧表存在，重命名备份（保留历史数据）
        let has_old_table = manager
            .has_table(AiChatOld::Table.to_string().as_str())
            .await?;

        if has_old_table {
            manager
                .rename_table(
                    Table::rename()
                        .table(AiChatOld::Table, "ai_chat_backup")
                        .to_owned(),
                )
                .await?;
        }

        // 2. 创建新表
        manager
            .create_table(
                Table::create()
                    .table(AiChat::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AiChat::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(AiChat::SessionId)
                            .string_len(64)
                            .not_null()
                            .comment("对话会话ID，一个对话多个来回共用"),
                    )
                    .col(
                        ColumnDef::new(AiChat::Openid)
                            .string_len(128)
                            .not_null()
                            .comment("用户openid"),
                    )
                    .col(
                        ColumnDef::new(AiChat::Title)
                            .string_len(100)
                            .null()
                            .comment("对话标题"),
                    )
                    .col(
                        ColumnDef::new(AiChat::UserMessage)
                            .text()
                            .null()
                            .comment("用户发送的消息内容"),
                    )
                    .col(
                        ColumnDef::new(AiChat::AiResponse)
                            .text()
                            .null()
                            .comment("AI回复的内容"),
                    )
                    .col(
                        ColumnDef::new(AiChat::MessageType)
                            .integer()
                            .not_null()
                            .default(0)
                            .comment("消息类型：0=文本, 1=图片, 2=语音"),
                    )
                    .col(
                        ColumnDef::new(AiChat::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp())
                            .comment("创建时间"),
                    )
                    .col(
                        ColumnDef::new(AiChat::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp())
                            .comment("更新时间"),
                    )
                    .to_owned(),
            )
            .await?;

        // 3. 单独创建索引（使用独立的 create_index 调用）
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_aichat_session_id")
                    .table(AiChat::Table)
                    .col(AiChat::SessionId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_aichat_openid")
                    .table(AiChat::Table)
                    .col(AiChat::Openid)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_aichat_created_at")
                    .table(AiChat::Table)
                    .col(AiChat::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. 删除索引
        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .name("idx_aichat_session_id")
                    .table(AiChat::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .name("idx_aichat_openid")
                    .table(AiChat::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .name("idx_aichat_created_at")
                    .table(AiChat::Table)
                    .to_owned(),
            )
            .await?;

        // 2. 删除新表
        manager
            .drop_table(Table::drop().if_exists().table(AiChat::Table).to_owned())
            .await?;

        // 3. 如果存在备份表，恢复旧表
        let has_backup = manager.has_table("ai_chat_backup").await?;
        if has_backup {
            manager
                .rename_table(
                    Table::rename()
                        .table("ai_chat_backup", "ai_chat")
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }
}

/// 新表结构
#[derive(Iden)]
pub enum AiChat {
    Table,
    Id,
    SessionId,
    Openid,
    Title,
    UserMessage,
    AiResponse,
    MessageType,
    CreatedAt,
    UpdatedAt,
}

/// 旧表结构（用于检测和备份）
#[derive(Iden)]
pub enum AiChatOld {
    Table,
}
