use anyhow::Result;
use async_trait::async_trait;
use corrosion_orm::{Create, query::query_type::QueryContext};

#[async_trait]
pub trait MigrationExecutor: Send {
    async fn execute_query(&mut self, ctx: &mut QueryContext) -> Result<u64>;
    async fn fetch_applied_migration_names(&mut self) -> Result<Vec<String>>;
    async fn insert_applied_migration(
        &mut self,
        name: &str,
        applied_at: chrono::NaiveDateTime,
    ) -> Result<()>;
    async fn delete_applied_migration(&mut self, name: &str) -> Result<()>;
    async fn execute_create_query(&mut self, create: &Create) -> Result<()>;
}

#[async_trait]
pub trait MigrationTrait: Send + Sync {
    fn name(&self) -> &'static str;
    async fn up(&self, db: &mut dyn MigrationExecutor) -> Result<()>;
    async fn down(&self, db: &mut dyn MigrationExecutor) -> Result<()>;
}

pub trait MigratorTrait {
    fn migrations() -> Vec<Box<dyn MigrationTrait>>;
}
