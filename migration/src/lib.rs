#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        // Migration-free architecture: Automatic Entity-First Schema Synchronization replaces manual migration files.
        // Migrator retains an empty list for full Loco framework compatibility.
        vec![]
    }
}
