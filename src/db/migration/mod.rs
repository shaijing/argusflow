//! Database migrations

pub mod m20240101_000001_create_papers;
pub mod m20240101_000002_create_authors;
pub mod m20240101_000003_create_paper_authors;
pub mod m20240101_000004_create_citations;

use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_papers::Migration),
            Box::new(m20240101_000002_create_authors::Migration),
            Box::new(m20240101_000003_create_paper_authors::Migration),
            Box::new(m20240101_000004_create_citations::Migration),
        ]
    }
}