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
                    .table(User::Table)
                    .col(
                        ColumnDef::new(User::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(User::OpenId)
                            .string()
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(User::Nickname).string())
                    .col(ColumnDef::new(User::Avatar).string())
                    .col(ColumnDef::new(User::Permission).integer().default(1))
                    .col(ColumnDef::new(User::Name).string())
                    .col(ColumnDef::new(User::PhoneNumber).string())
                    .col(ColumnDef::new(User::Address).string())
                    .col(ColumnDef::new(User::Address).string())
                    .col(ColumnDef::new(User::IsImportant).boolean().default(false))
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum User {
    Table,
    Id,
    OpenId,
    Nickname,
    Name,
    PhoneNumber,
    Address,
    Community,
    IsImportant,
    Avatar,
    Permission,
}
