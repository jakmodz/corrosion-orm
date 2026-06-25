use corrosion_orm_migration::{MigrationTrait, MigratorTrait};

pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        include!(concat!(env!("OUT_DIR"), "/migration_list.rs"))
    }
}
