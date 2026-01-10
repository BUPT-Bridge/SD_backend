// src/migrator/mod.rs

use sea_orm_migration::prelude::*;

mod x;
mod y;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(y::Migration), Box::new(x::Migration)]
    }
}
