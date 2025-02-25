pub use sea_orm_migration::prelude::*;

mod m20250224_015652_create_user_table;
mod m20250224_231932_add_user_username;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250224_015652_create_user_table::Migration),
            Box::new(m20250224_231932_add_user_username::Migration),
        ]
    }
}
