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
    // Define how to apply this migration: Create the HealthGuideContent table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(HealthGuideContent::Table)
                    .col(
                        ColumnDef::new(HealthGuideContent::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(HealthGuideContent::TypeOne)
                            .integer()  //匹配一级的type,即id
                    )
                    .col(
                        ColumnDef::new(HealthGuideContent::Content)
                            .json()
                    )
                    .col(
                        ColumnDef::new(HealthGuideContent::TypeTwo) //二级Type的string
                            .string()  
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(HealthGuideContent::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum HealthGuideContent {
    Table,
    Id,
    TypeOne,
    TypeTwo,
    Content,
}
