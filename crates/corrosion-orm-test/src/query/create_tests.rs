#[cfg(test)]
mod tests {
    use crate::User;
    use corrosion_orm_core::{
        dialect::sqlite_dialect::sqlite::SqliteDialect,
        prelude::{Create, QueryContext, TableSchema, ToSql},
    };

    fn render_create(create: Create) -> String {
        let mut ctx = QueryContext::new();
        create.to_sql(&mut ctx, &SqliteDialect);
        ctx.sql
    }

    #[test]
    fn test_create_from_schema() {
        let schema = User::get_schema();
        let sql = render_create(Create::new(schema));

        insta::assert_snapshot!(sql, @r"
        CREATE TABLE users (
            id INTEGER PRIMARY KEY,
            name TEXT NULL UNIQUE
        );
        CREATE UNIQUE INDEX IF NOT EXISTS idx_users_id ON users (id);
        ");
    }

    #[test]
    fn test_create_if_not_exists() {
        let schema = User::get_schema();
        let sql = render_create(Create::new(schema).if_not_exists());

        insta::assert_snapshot!(sql, @r"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            name TEXT NULL UNIQUE
        );
        CREATE UNIQUE INDEX IF NOT EXISTS idx_users_id ON users (id);
        ");
    }

    #[test]
    fn test_create_from_impl() {
        let schema = User::get_schema();
        let sql = render_create(Create::from(schema));

        insta::assert_snapshot!(sql, @r"
        CREATE TABLE users (
            id INTEGER PRIMARY KEY,
            name TEXT NULL UNIQUE
        );
        CREATE UNIQUE INDEX IF NOT EXISTS idx_users_id ON users (id);
        ");
    }
}
