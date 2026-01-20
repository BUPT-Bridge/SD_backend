// src/migrator/mod.rs

use sea_orm_migration::prelude::*;

pub mod ai_chat;
pub mod community_service;
pub mod detail_meal;
pub mod dinner_provider;
pub mod feedback;
pub mod health_guide_content;
pub mod health_guide_type;
pub mod medical_service;
pub mod mutil_media;
pub mod notice;
pub mod policy_file;
pub mod policy_type;
pub mod resource_service;
pub mod service_map_content;
pub mod service_map_type;
pub mod slideshow;
pub mod user;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(community_service::Migration),
            Box::new(dinner_provider::Migration),
            Box::new(medical_service::Migration),
            Box::new(notice::Migration),
            Box::new(policy_file::Migration),
            Box::new(policy_type::Migration),
            Box::new(resource_service::Migration),
            Box::new(slideshow::Migration),
            Box::new(detail_meal::Migration),
            Box::new(health_guide_type::Migration),
            Box::new(health_guide_content::Migration),
            Box::new(service_map_type::Migration),
            Box::new(service_map_content::Migration),
            Box::new(feedback::Migration),
            Box::new(user::Migration),
        ]
    }
}
