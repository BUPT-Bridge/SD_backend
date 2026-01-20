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
    // Define how to apply this migration: Create the DetailMeal table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DetailMeal::Table)
                    .col(
                        ColumnDef::new(DetailMeal::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(DetailMeal::Type)
                            .string()
                    )
                    .col(
                        ColumnDef::new(DetailMeal::DateTime)
                            .string()
                    )
                    .col(
                        ColumnDef::new(DetailMeal::MealInfo)
                            .json()
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DetailMeal::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum DetailMeal {
    Table,
    Id,
    Type, //早餐中餐或晚餐
    DateTime,
    MealInfo, //直接存入json
}
