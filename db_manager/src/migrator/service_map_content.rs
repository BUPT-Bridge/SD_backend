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
    // Define how to apply this migration: Create the ServiceMapContent table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ServiceMapContent::Table)
                    .col(
                        ColumnDef::new(ServiceMapContent::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(ServiceMapContent::TypeOne)
                            .integer()  // 社区的id
                    )
                    .col(
                        ColumnDef::new(ServiceMapContent::TypeTwo)
                            .string()  //同ServiceMapType::TypeName即可，用这个来检索
                    )
                    .col(
                        ColumnDef::new(ServiceMapContent::Content)
                            .json()  // 细节的内容
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ServiceMapContent::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum ServiceMapContent {
    Table,
    Id,
    TypeOne,
    TypeTwo,
    Content,
}
