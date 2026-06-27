use crate::{
    dialect::sql_dialect::SqlDialect,
    types::{column_type::SqlType, generation_strategy::GenerationType},
};

pub struct SqliteDialect;

impl SqlDialect for SqliteDialect {
    fn cast_type(&self, sql_type: &SqlType) -> String {
        match sql_type {
            SqlType::Integer => "INTEGER",
            SqlType::Float => "REAL",
            SqlType::Double => "REAL",
            SqlType::Char(_) => "TEXT",
            SqlType::Varchar(_) => "TEXT",
            SqlType::Text => "TEXT",
            SqlType::Boolean => "INTEGER",
            SqlType::Date => "DATE",
            SqlType::Timestamp => "TIMESTAMP",
            SqlType::Custom(_) => unreachable!(),
        }
        .to_string()
    }

    fn bind_param(&self, _count: &usize) -> String {
        "?".to_string()
    }

    fn cast_generation_type(&self, generation_type: &GenerationType) -> String {
        match generation_type {
            GenerationType::AutoIncrement => " AUTOINCREMENT".to_string(),
        }
    }

    fn generate_empty_insert(&self, table_name: &str) -> String {
        format!("INSERT INTO {} DEFAULT VALUES;\n", table_name)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    const DIALECT: SqliteDialect = SqliteDialect;

    #[test]
    fn test_cast_type_integer() {
        assert_eq!(DIALECT.cast_type(&SqlType::Integer), "INTEGER");
    }
    #[test]
    fn test_cast_type_float() {
        assert_eq!(DIALECT.cast_type(&SqlType::Float), "REAL");
    }
    #[test]
    fn test_cast_type_double() {
        assert_eq!(DIALECT.cast_type(&SqlType::Double), "REAL");
    }
    #[test]
    fn test_cast_type_char() {
        assert_eq!(DIALECT.cast_type(&SqlType::Char(10)), "TEXT");
    }
    #[test]
    fn test_cast_type_varchar() {
        assert_eq!(DIALECT.cast_type(&SqlType::Varchar(10)), "TEXT");
    }
    #[test]
    fn test_cast_type_text() {
        assert_eq!(DIALECT.cast_type(&SqlType::Text), "TEXT");
    }
}
