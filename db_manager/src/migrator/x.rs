// src/migrator/m20220602_000002_create_chef_table.rs (create new file)

use sea_orm_migration::prelude::*;

use super::y::Bakery;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "create_chef_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Chef table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Chef::Table)
                    .col(
                        ColumnDef::new(Chef::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Chef::Name).string().not_null())
                    .col(ColumnDef::new(Chef::ContactDetails).json())
                    .col(ColumnDef::new(Chef::BakeryId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-chef-bakery_id")
                            .from(Chef::Table, Chef::BakeryId)
                            .to(Bakery::Table, Bakery::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Chef table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Chef::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Chef {
    Table,
    Id,
    Name,
    ContactDetails,
    BakeryId,
}
