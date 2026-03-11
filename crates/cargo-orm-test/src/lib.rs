pub mod dialect;
pub mod from_row_impl;
mod sqlite_tests;

mod tests {
    use cargo_orm_core::{schema::table::TableSchema, types::column_type::SqlType};
    use cargo_orm_macros::Model;

    #[derive(Model)]
    #[Table(name = "users")]
    struct User {
        #[Column(name = "id")]
        #[PrimaryKey]
        id: i32,
        #[Column(name = "username", unique, nullable)]
        username: String,
        #[Column(name = "email", unique = false, nullable)]
        email: Option<String>,
    }

    #[test]
    fn test_table_name() {
        assert_eq!(User::get_table_name(), "users");
    }

    #[test]
    fn test_table_schema() {
        let schema = User::get_schema();
        assert_eq!(schema.name, "users");
        assert_eq!(schema.fields[0].name, "username");
        assert_eq!(schema.fields[0].sql_type, SqlType::Varchar(255));
        assert_eq!(schema.fields[1].name, "email");
        assert_eq!(schema.fields[1].sql_type, SqlType::Varchar(255));
    }
}
