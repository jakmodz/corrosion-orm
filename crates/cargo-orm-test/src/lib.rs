#[cfg(test)]
mod tests {
    use cargo_orm_macros::Model;
    use cargo_orm_core::{schema::table::TableSchema, types::colum_type::SqlType};
    
    
    #[derive(Model)]
    #[table(name = "users")]
    struct User {
        _id: i32,
        _name: String,
    } 
    
    #[test]
    fn test_table_name(){
        assert_eq!(User::get_table_name(),"users");
    }
    
    #[test]
    fn test_table_schema(){
        let schema = User::get_schema();
        assert_eq!(schema.name,"users");
        assert_eq!(schema.fields.len(),2);
        assert_eq!(schema.fields[0].name,"_id");
        assert_eq!(schema.fields[0].sql_type,SqlType::Integer);
        assert_eq!(schema.fields[1].name,"_name");
        assert_eq!(schema.fields[1].sql_type,SqlType::Varchar(255));
    }
}
