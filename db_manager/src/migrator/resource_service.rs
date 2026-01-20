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
    // Define how to apply this migration: Create the ResourceService table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ResourceService::Table)
                    .col(
                        ColumnDef::new(ResourceService::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(ResourceService::Name).string())
                    .col(ColumnDef::new(ResourceService::Address).string())
                    .col(ColumnDef::new(ResourceService::Phone).string())
                    .col(ColumnDef::new(ResourceService::Latitude).float())
                    .col(ColumnDef::new(ResourceService::Longitude).float())
                    .col(ColumnDef::new(ResourceService::ServiceTime).string())
                    .col(ColumnDef::new(ResourceService::Boss).string())
                    .col(
                        ColumnDef::new(ResourceService::CreateTime)
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
            .drop_table(Table::drop().table(ResourceService::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum ResourceService {
    Table,
    Id,
    Name,
    Address,
    Phone,
    Latitude,
    Longitude,
    ServiceTime,
    Boss,
    CreateTime,
}
