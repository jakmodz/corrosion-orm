use corrosion_orm::model::snapshot::{AppRegistry, ModelRegistry};

include!(concat!(env!("OUT_DIR"), "/migration_modules.rs"));
mod migrator;

pub struct MyAppRegistry;

impl AppRegistry for MyAppRegistry {
    fn app_registry() -> ModelRegistry {
        ModelRegistry::new()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let registry = MyAppRegistry::app_registry();
    corrosion_orm_migration::cli::run_cli::<migrator::Migrator>(&registry).await
}
