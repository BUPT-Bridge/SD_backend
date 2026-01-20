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
    // Define how to apply this migration: Create the CommunityService table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CommunityService::Table)
                    .col(
                        ColumnDef::new(CommunityService::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(CommunityService::Name).string())
                    .col(ColumnDef::new(CommunityService::Address).string())
                    .col(ColumnDef::new(CommunityService::Phone).string())
                    .col(ColumnDef::new(CommunityService::Latitude).float())
                    .col(ColumnDef::new(CommunityService::Longitude).float())
                    .col(ColumnDef::new(CommunityService::CreateTime).timestamp().not_null().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CommunityService::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum CommunityService {
    Table,
    Id,
    Name,
    Address,
    Phone,
    Latitude,
    Longitude,
    CreateTime,
}
