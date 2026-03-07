#[cfg(test)]
mod tests{    
    use cargo_orm_macros::Model;
    
    #[derive(Model)]
    #[Table(name = "test_table")]
    struct User {
        #[Column(name = "id")]
        #[PrimaryKey]
        #[allow(unused)]
        id: i32,
        #[Column(name = "name", unique, nullable)]
        name: String,
    }

    #[test]
    fn test_generate_ddl_sqlite()->Result<(), cargo_orm_core::schema::table::SchemaValidationError> {
        use cargo_orm_core::dialect::{sql_dialect::SqlDialect, sqlite_dialect::sqlite::SqliteDialect};
        use cargo_orm_core::{schema::table::TableSchema};
        let dialect = SqliteDialect;
        let schema = User::get_schema();
        let  ddl = dialect.generate_ddl(&schema)?;
        
        insta::assert_snapshot!(ddl);
        Ok(())
    }
}
