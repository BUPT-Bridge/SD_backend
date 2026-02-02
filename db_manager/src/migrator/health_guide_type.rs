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
    // Define how to apply this migration: Create the HealthGuideType table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(HealthGuideType::Table)
                    .col(
                        ColumnDef::new(HealthGuideType::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(HealthGuideType::TypeName).string())
                    .col(ColumnDef::new(HealthGuideType::Icon).integer())
                    .col(ColumnDef::new(HealthGuideType::TypeSum).integer())
                    .col(ColumnDef::new(HealthGuideType::TypeOne).json())
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(HealthGuideType::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum HealthGuideType {
    Table,
    Id,
    TypeName,
    Icon,
    TypeSum,
    TypeOne,
}
