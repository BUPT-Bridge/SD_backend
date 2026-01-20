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
        manager.create_table(
            Table::create()
                .table(PolicyFile::Table)
                .col(
                    ColumnDef::new(PolicyFile::Id)
                        .integer()
                        .not_null()
                        .primary_key()
                        .auto_increment()
                        .unique_key(),
                )
                .col(ColumnDef::new(PolicyFile::Title).string())
                .col(ColumnDef::new(PolicyFile::Type).string())
                .col(ColumnDef::new(PolicyFile::Index).string())
                .col(
                    ColumnDef::new(PolicyFile::CreateTime)
                        .timestamp()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .to_owned(),
        )
        .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PolicyFile::Table).to_owned())
            .await
    }
}



#[derive(Iden)]
pub enum PolicyFile {
    Table,
    Id,
    Title,
    Type,
    Index,
    CreateTime,
}
