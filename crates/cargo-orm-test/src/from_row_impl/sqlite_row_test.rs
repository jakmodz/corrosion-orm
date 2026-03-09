#[cfg(test)]
mod tests {
    use cargo_orm_core::{
        dialect::{sql_dialect::SqlDialect, sqlite_dialect::sqlite::SqliteDialect},
        schema::table::TableSchema,
    };
    use cargo_orm_macros::Model;
    use sqlx::{FromRow, SqlitePool};

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
    #[tokio::test]
    async fn test_from_row_impl() {
        let dialect = SqliteDialect;

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
