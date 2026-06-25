#[cfg(test)]
mod tests {
    use corrosion_orm_core::{
        dialect::sqlite_dialect::sqlite::SqliteDialect,
        query::alter::{Alter, AlterAction},
        query::query_type::QueryContext,
        query::to_sql::ToSql,
        schema::table::ColumnSchemaModel,
        types::column_type::SqlType,
    };

    fn render_alter(alter: Alter) -> String {
        let mut ctx = QueryContext::new();
        alter.to_sql(&mut ctx, &SqliteDialect);
        ctx.sql
    }

    #[test]
    fn test_alter_add_column() {
        let col = ColumnSchemaModel {
            name: "email".to_string(),
            is_nullable: false,
            is_unique: true,
            sql_type: SqlType::Varchar(255),
            generation_type: None,
        };

        let sql = render_alter(Alter::new("users", AlterAction::AddColumn { column: col }));

        insta::assert_snapshot!(sql, @r"
        ALTER TABLE users ADD     email TEXT NOT NULL UNIQUE
        ");
    }

    #[test]
    fn test_alter_drop_column() {
        let sql = render_alter(Alter::new(
            "users",
            AlterAction::DropColumn {
                name: "age".to_string(),
            },
        ));

        insta::assert_snapshot!(sql, @r"
        ALTER TABLE users DROP COLUMN age
        ");
    }

    #[test]
    fn test_alter_rename_column() {
        let sql = render_alter(Alter::new(
            "users",
            AlterAction::RenameColumn {
                from: "username".to_string(),
                to: "display_name".to_string(),
            },
        ));

        insta::assert_snapshot!(sql, @r"
        ALTER TABLE users RENAME COLUMN username TO display_name
        ");
    }

    #[test]
    fn test_alter_rename_table() {
        let sql = render_alter(Alter::new(
            "users",
            AlterAction::RenameTable {
                to: "accounts".to_string(),
            },
        ));

        insta::assert_snapshot!(sql, @r"
        ALTER TABLE users RENAME TO accounts
        ");
    }
}
