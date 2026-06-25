use anyhow::Result;
use async_trait::async_trait;
use corrosion_orm::query::query_type::QueryContext;
use corrosion_orm_migration::{MigrationExecutor, MigrationTrait};

pub struct GeneratedMigration;

#[async_trait]
impl MigrationTrait for GeneratedMigration {
    fn name(&self) -> &'static str {
        "{{migration_name}}"
    }

    async fn up(&self, _db: &mut dyn MigrationExecutor) -> Result<()> {
        {
            {{up_code}}
        }
        Ok(())
    }

    async fn down(&self, _db: &mut dyn MigrationExecutor) -> Result<()> {
        {
            {{down_code}}
        }
        Ok(())
    }
}
