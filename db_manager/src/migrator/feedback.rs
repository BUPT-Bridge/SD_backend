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
    // Define how to apply this migration: Create the Feedback table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Feedback::Table)
                    .col(
                        ColumnDef::new(Feedback::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Feedback::Type)
                            .string()
                    )
                    .col(
                        ColumnDef::new(Feedback::Content)
                            .string()
                    )
                    .col(ColumnDef::new(Feedback::CreatedTime)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Feedback::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Feedback {
    Table,
    Id,
    Type,
    Content,
    Phone,
    CreatedTime,
}
