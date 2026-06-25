use crate::migration_registry::{MigrationRegistry, migrationregistry};
use anyhow::Result;
use corrosion_orm::driver::connection::Conn;
use corrosion_orm::{
    CorrosionSqlitePool, Repository, driver::connection_pool::ConnectionGuard, prelude::Executor,
    query::query_type::QueryContext,
};
use corrosion_orm::{Create, ToSql};

use crate::migration::MigrationExecutor;

pub struct SqliteMigrationExecutor<'a> {
    conn: &'a mut ConnectionGuard<CorrosionSqlitePool>,
}

impl<'a> SqliteMigrationExecutor<'a> {
    pub fn new(conn: &'a mut ConnectionGuard<CorrosionSqlitePool>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl MigrationExecutor for SqliteMigrationExecutor<'_> {
    async fn execute_query(&mut self, ctx: &mut QueryContext) -> Result<u64> {
        Ok(self.conn.execute_query(ctx).await?)
    }

    async fn fetch_applied_migration_names(&mut self) -> Result<Vec<String>> {
        let migrations = MigrationRegistry::find()
            .add_order_by(migrationregistry::COLUMN.id.asc())
            .all(self.conn)
            .await?;
        Ok(migrations.into_iter().map(|m| m.name).collect())
    }

    async fn insert_applied_migration(
        &mut self,
        name: &str,
        applied_at: chrono::NaiveDateTime,
    ) -> Result<()> {
        let record = MigrationRegistry::new(name.to_string(), applied_at);
        record.save(self.conn).await?;
        Ok(())
    }

    async fn delete_applied_migration(&mut self, name: &str) -> Result<()> {
        let record = MigrationRegistry::find()
            .filter(migrationregistry::COLUMN.migration_name.eq(name))
            .one(self.conn)
            .await?;
        record.delete(self.conn).await?;
        Ok(())
    }

    async fn execute_create_query(&mut self, create: &Create) -> Result<()> {
        self.conn.begin_transaction().await?;
        let mut ctx = QueryContext::new();
        create.to_sql(&mut ctx, self.conn.get_dialect());
        self.execute_query(&mut ctx).await?;
        self.conn.commit_transaction().await?;
        Ok(())
    }
}
