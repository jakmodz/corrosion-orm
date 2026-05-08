#[cfg(test)]
mod tests {
    use crate::User;
    use crate::test_entities::Post;
    use corrosion_orm_core::{
        dialect::{sql_dialect::SqlDialect, sqlite_dialect::sqlite::SqliteDialect},
        prelude::TableSchema,
    };
    #[allow(clippy::disallowed_types)]
    use sqlx::SqlitePool;

    #[test]
    fn test_generate_ddl_sqlite()
    -> Result<(), corrosion_orm_core::schema::table::SchemaValidationError> {
        let dialect = SqliteDialect;
        let schema = User::get_schema();
        let ddl = dialect.generate_ddl(&schema)?;
        insta::assert_snapshot!(ddl);
        Ok(())
    }
    /// Verifies that the SQLite dialect generates the expected DDL for the `Post` has-one relation.
    ///
    /// Generates DDL for `Post::get_schema()` using `SqliteDialect`, asserts the output with
    /// `insta::assert_snapshot!`, and returns `Ok(())` on success.
    ///
    /// # Returns
    ///
    /// `Ok(())` if DDL generation and snapshot assertion succeed, or a
    /// `corrosion_orm_core::schema::table::SchemaValidationError` if schema validation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// let dialect = SqliteDialect;
    /// let schema = Post::get_schema();
    /// let ddl = dialect.generate_ddl(&schema).unwrap();
    /// insta::assert_snapshot!(ddl);
    /// ```
    #[test]
    fn test_generate_ddl_relation_has_one()
    -> Result<(), corrosion_orm_core::schema::table::SchemaValidationError> {
        let dialect = SqliteDialect;
        let schema = Post::get_schema();
        let ddl = dialect.generate_ddl(&schema)?;
        insta::assert_snapshot!(ddl);
        Ok(())
    }

    /// Verifies that the full DDL produced by the SQLite dialect for `User` executes successfully
    /// against an in-memory SQLite database.
    ///
    /// This test generates the complete DDL for `User`, runs it on a transient in-memory SQLite
    /// connection, and performs a simple insert to ensure the table exists and accepts data.
    ///
    /// # Examples
    ///
    /// ```
    /// // generates and runs full DDL for `User` on an in-memory SQLite DB
    /// let dialect = corrosion_orm_core::dialect::sqlite_dialect::sqlite::SqliteDialect;
    /// let schema = crate::User::get_schema();
    /// let ddl = dialect.generate_full_ddl(&schema).unwrap();
    ///
    /// let pool = sqlx::SqlitePool::connect(":memory:").await.unwrap();
    /// sqlx::query(&ddl).execute(&pool).await.unwrap();
    /// sqlx::query("INSERT INTO users (id, name) VALUES (1, 'test')")
    ///     .execute(&pool)
    ///     .await
    ///     .unwrap();
    /// ```
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
