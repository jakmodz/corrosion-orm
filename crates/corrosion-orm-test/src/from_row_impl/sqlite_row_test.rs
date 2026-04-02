#[cfg(test)]
mod tests {
    use crate::User;
    use corrosion_orm_core::{
        dialect::{sql_dialect::SqlDialect, sqlite_dialect::sqlite::SqliteDialect},
        schema::table::TableSchema,
    };
    #[allow(clippy::disallowed_types)]
    use sqlx::{FromRow, SqlitePool};
    #[tokio::test]
    async fn test_from_row_impl() {
        let dialect = SqliteDialect;
        #[allow(clippy::disallowed_types)]
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let ddl = dialect.generate_ddl(&User::get_schema()).unwrap();

        sqlx::query(ddl.as_str()).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO users (id, name) VALUES (1, 'test')")
            .execute(&pool)
            .await
            .unwrap();
        let row = sqlx::query("SELECT * FROM users")
            .fetch_one(&pool)
            .await
            .unwrap();
        let user = User::from_row(&row).unwrap();
        assert_eq!(user.id, 1);
        assert_eq!(user.name, "test");
    }
}
