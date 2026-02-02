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
    // Define how to apply this migration: Create the ServiceMapType table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ServiceMapType::Table)
                    .col(
                        ColumnDef::new(ServiceMapType::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(ServiceMapType::CommunityName)
                            .string()
                    )
                    .col(
                        ColumnDef::new(ServiceMapType::TypeSum)
                            .integer()
                    )
                    .col(
                        ColumnDef::new(ServiceMapType::TypeName)
                            .json()
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ServiceMapType::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum ServiceMapType {
    Table,
    Id,
    CommunityName,
    TypeSum,
    TypeName,
}
