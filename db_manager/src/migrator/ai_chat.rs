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
    // Define how to apply this migration: Create the Bakery table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AiChat::Table)
                    .col(
                        ColumnDef::new(AiChat::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(AiChat::Index)
                            .string()
                    )
                    .col(
                        ColumnDef::new(AiChat::Openid)
                            .string()
                    )
                    .col(
                        ColumnDef::new(AiChat::LongContent)
                            .string()
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AiChat::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum AiChat {
    Table,
    Id,
    Index, // api返回的对话索引
    Openid,
    LongContent,
}
