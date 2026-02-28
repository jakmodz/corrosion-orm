#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SqlType {
    /// Represents an integer value.
    Integer,
    /// Represents a decimal value.
    Float,
    /// Represents a double value.
    Double,
    /// Represents a character value with a fixed length.
    Char(usize),
    /// Represents a string value with a maximum length.
    Varchar(usize),
    /// Represents a text value with no maximum length.
    Text,
    /// Represents a boolean value.
    Boolean,
    /// Represents a date value.
    Date,
    /// Represents a timestamp value.
    Timestamp,
    /// Special variant for column defined with a specific SQL type.
    Custom(String),
}
/// Trait for casting rust types to sql types
pub trait ToSqlType {
    fn to_sql_type(&self) -> SqlType;
}

impl ToSqlType for String {
    fn to_sql_type(&self) -> SqlType {
        if self.len() > 255 {
            return SqlType::Text;
        }
        SqlType::Varchar(255)
    }
}
impl ToSqlType for i32 {
    fn to_sql_type(&self) -> SqlType {
        SqlType::Integer
    }
}
impl ToSqlType for bool {
    fn to_sql_type(&self) -> SqlType {
        SqlType::Boolean
    }
}
impl ToSqlType for chrono::NaiveDate {
    fn to_sql_type(&self) -> SqlType {
        SqlType::Date
    }
}
impl ToSqlType for chrono::NaiveDateTime {
    fn to_sql_type(&self) -> SqlType {
        SqlType::Timestamp
    }
}
impl ToSqlType for u16 {
    fn to_sql_type(&self) -> SqlType {
        SqlType::Integer
    }
}
impl ToSqlType for u32 {
    fn to_sql_type(&self) -> SqlType {
        SqlType::Integer
    }
}
impl ToSqlType for u64 {
    fn to_sql_type(&self) -> SqlType {
        SqlType::Integer
    }
}
impl ToSqlType for i64 {
    fn to_sql_type(&self) -> SqlType {
        SqlType::Integer
    }
}
impl ToSqlType for u8 {
    fn to_sql_type(&self) -> SqlType {
        SqlType::Integer
    }
}
impl ToSqlType for i8 {
    fn to_sql_type(&self) -> SqlType {
        SqlType::Integer
    }
}
impl ToSqlType for u128 {
    fn to_sql_type(&self) -> SqlType {
        SqlType::Integer
    }
}
impl ToSqlType for i128 {
    fn to_sql_type(&self) -> SqlType {
        SqlType::Integer
    }
}
impl ToSqlType for f32 {
    fn to_sql_type(&self) -> SqlType {
        SqlType::Float
    }
}
impl ToSqlType for f64 {
    fn to_sql_type(&self) -> SqlType {
        SqlType::Double
    }
}
impl<T: ToSqlType + Default> ToSqlType for Option<T> {
    fn to_sql_type(&self) -> SqlType {
        T::default().to_sql_type()
    }
}
