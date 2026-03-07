#[cfg(test)]
mod tests {
    use cargo_orm_macros::Model;
    use sqlx::SqlitePool;

    #[derive(Model)]
    #[Table(name = "users")]
    struct User {
        #[Column(name = "id")]
        #[PrimaryKey]
        #[allow(unused)]
        id: i32,
        #[Column(name = "name", unique, nullable)]
        name: String,
    }

    #[test]
    fn test_generate_ddl_sqlite() -> Result<(), cargo_orm_core::schema::table::SchemaValidationError>
    {
        use cargo_orm_core::dialect::{
            sql_dialect::SqlDialect, sqlite_dialect::sqlite::SqliteDialect,
        };
        use cargo_orm_core::schema::table::TableSchema;
        let dialect = SqliteDialect;
        let schema = User::get_schema();
        let ddl = dialect.generate_ddl(&schema)?;

        insta::assert_snapshot!(ddl);
        Ok(())
    }
    #[tokio::test]
    async fn test_generate_full_ddl_sqlite_connection() {
        use cargo_orm_core::dialect::{
            sql_dialect::SqlDialect, sqlite_dialect::sqlite::SqliteDialect,
        };

        use cargo_orm_core::schema::table::TableSchema;
        let dialect = SqliteDialect;
        let schema = User::get_schema();
        let ddl = dialect.generate_full_ddl(&schema).unwrap();

        let pool = SqlitePool::connect(":memory:").await.unwrap();
        sqlx::query(&ddl).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO users (id, name) VALUES (1, 'test')")
            .execute(&pool)
            .await
            .unwrap();
    }
}
