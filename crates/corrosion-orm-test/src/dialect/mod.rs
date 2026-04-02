#[cfg(test)]
mod tests {
    use crate::User;
    #[allow(clippy::disallowed_types)]
    use sqlx::SqlitePool;

    #[test]
    fn test_generate_ddl_sqlite()
    -> Result<(), corrosion_orm_core::schema::table::SchemaValidationError> {
        use corrosion_orm_core::dialect::{
            sql_dialect::SqlDialect, sqlite_dialect::sqlite::SqliteDialect,
        };
        use corrosion_orm_core::schema::table::TableSchema;
        let dialect = SqliteDialect;
        let schema = User::get_schema();
        let ddl = dialect.generate_ddl(&schema)?;

        insta::assert_snapshot!(ddl);
        Ok(())
    }
    #[tokio::test]
    async fn test_generate_full_ddl_sqlite_connection() {
        use corrosion_orm_core::dialect::{
            sql_dialect::SqlDialect, sqlite_dialect::sqlite::SqliteDialect,
        };

        use corrosion_orm_core::schema::table::TableSchema;
        let dialect = SqliteDialect;
        let schema = User::get_schema();
        let ddl = dialect.generate_full_ddl(&schema).unwrap();

        #[allow(clippy::disallowed_types)]
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        sqlx::query(&ddl).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO users (id, name) VALUES (1, 'test')")
            .execute(&pool)
            .await
            .unwrap();
    }
}
