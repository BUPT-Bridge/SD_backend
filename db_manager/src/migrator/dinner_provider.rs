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
    // Define how to apply this migration: Create the DinnerProvider table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DinnerProvider::Table)
                    .col(
                        ColumnDef::new(DinnerProvider::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(DinnerProvider::Name).string())
                    .col(ColumnDef::new(DinnerProvider::Address).string())
                    .col(ColumnDef::new(DinnerProvider::Phone).string())
                    .col(ColumnDef::new(DinnerProvider::Latitude).float())
                    .col(ColumnDef::new(DinnerProvider::Longitude).float())
                    .col(ColumnDef::new(DinnerProvider::ServiceTime).string())
                    .col(ColumnDef::new(DinnerProvider::BonusInfo).string())
                    .col(ColumnDef::new(DinnerProvider::MealStyle).string())
                    .col(
                        ColumnDef::new(DinnerProvider::CreateTime)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the MedicalService table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DinnerProvider::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum DinnerProvider {
    Table,
    Id,
    Name,
    Address,
    Phone,
    Latitude,
    Longitude,
    ServiceTime,
    BonusInfo,
    MealStyle,
    CreateTime,
}
