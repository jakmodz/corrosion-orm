use crate::dialect::sql_dialect::SqlDialect;
use crate::query::{create::Create, to_sql::ToSql};
use crate::schema::table::TableSchemaModel;
/// Enum representing a SQL value of various types.
#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Int(i32),
    Int64(i64),
    Float(f64),
    Bool(bool),
    Date(chrono::NaiveDate),
    DateTime(chrono::NaiveDateTime),
    Null,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Int64(a), Value::Int64(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b || (a.is_nan() && b.is_nan()),
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Date(a), Value::Date(b)) => a == b,
            (Value::DateTime(a), Value::DateTime(b)) => a == b,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }
}

impl Eq for Value {}

macro_rules! impl_from_value {
    ($variant:ident, $ty:ty) => {
        impl From<$ty> for Value {
            fn from(v: $ty) -> Self {
                Value::$variant(v as _)
            }
        }
    };
}

impl_from_value!(Int, i32);
impl_from_value!(Int, i8);
impl_from_value!(Int, i16);
impl_from_value!(Int, u8);
impl_from_value!(Int, u16);
impl_from_value!(Int64, i64);
impl_from_value!(Int64, u32);
impl_from_value!(Float, f32);
impl_from_value!(Float, f64);
impl_from_value!(Bool, bool);

impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::String(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::String(v.to_string())
    }
}
impl From<chrono::NaiveDate> for Value {
    fn from(v: chrono::NaiveDate) -> Self {
        Value::Date(v)
    }
}
impl From<chrono::NaiveDateTime> for Value {
    fn from(v: chrono::NaiveDateTime) -> Self {
        Value::DateTime(v)
    }
}
impl<T> From<Option<T>> for Value
where
    Value: From<T>,
{
    fn from(v: Option<T>) -> Self {
        match v {
            Some(inner) => Value::from(inner),
            None => Value::Null,
        }
    }
}

/// Struct representing a query context, holding the SQL string and bind parameters.
pub struct QueryContext {
    pub sql: String,
    pub values: Vec<Value>,
    pub(crate) placeholder_count: usize,
}

impl QueryContext {
    /// Creates a new `QueryContext` with an empty SQL string and no bind parameters.
    pub fn new() -> Self {
        Self {
            sql: String::new(),
            values: Vec::new(),
            placeholder_count: 0,
        }
    }
    #[cfg(feature = "log")]
    pub fn to_debug_sql(&self, dialect: &dyn SqlDialect) -> String {
        let mut sql = self.sql.clone();
        for idx in (1..=self.values.len()).rev() {
            let value = &self.values[idx - 1];
            let placeholder = dialect.bind_param(&idx);
            if let Some(pos) = sql.rfind(&placeholder) {
                let value_str = match value {
                    Value::String(s) => format!("'{}'", s.replace('\'', "''")),
                    Value::Int(i) => i.to_string(),
                    Value::Int64(i) => i.to_string(),
                    Value::Float(f) => f.to_string(),
                    Value::Bool(b) => if *b { "1" } else { "0" }.to_string(),
                    Value::Date(d) => format!("'{}'", d),
                    Value::DateTime(naive_date_time) => format!("'{}'", naive_date_time),
                    Value::Null => "NULL".to_string(),
                };
                sql.replace_range(pos..pos + placeholder.len(), &value_str);
            }
        }
        sql
    }
    /// Pushes a bind parameter to the query context, updating the SQL string and values vector.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to bind.
    /// * `dialect` - The SQL dialect to use for binding.
    ///
    /// # Examples
    ///
    /// ```
    /// use corrosion_orm_core::query::query_type::QueryContext;
    /// use corrosion_orm_core::dialect::sqlite_dialect::SqliteDialect;
    /// use corrosion_orm_core::query::query_type::Value;
    ///
    /// let mut ctx = QueryContext::new();
    /// ctx.push_bind_param(Value::String("foo".to_string()), &SqliteDialect);
    /// ```
    pub fn push_bind_param(&mut self, value: Value, dialect: &dyn SqlDialect) {
        self.placeholder_count += 1;
        self.sql
            .push_str(&dialect.bind_param(&self.placeholder_count));
        self.values.push(value);
    }
    /// Creates a new `QueryContext` from a `TableSchemaModel` using the given SQL dialect.
    ///
    /// # Arguments
    ///
    /// * `model` - The table schema model to generate the DDL from.
    /// * `dialect` - The SQL dialect to use for generating the DDL.
    pub fn from_model(model: TableSchemaModel, dialect: &dyn SqlDialect) -> Self {
        let mut ctx = Self::new();
        let create = Create::new(model);
        create.to_sql(&mut ctx, dialect);
        ctx
    }

    /// Creates a new `QueryContext` from a `TableSchemaModel` using `CREATE TABLE IF NOT EXISTS`.
    pub fn from_model_if_not_exists(model: TableSchemaModel, dialect: &dyn SqlDialect) -> Self {
        let mut ctx = Self::new();
        let create = Create::new(model).if_not_exists();
        create.to_sql(&mut ctx, dialect);
        ctx
    }
}
impl Default for QueryContext {
    fn default() -> Self {
        Self::new()
    }
}

impl From<String> for QueryContext {
    fn from(sql: String) -> Self {
        Self {
            sql,
            values: Vec::new(),
            placeholder_count: 0,
        }
    }
}
impl From<&str> for QueryContext {
    fn from(sql: &str) -> Self {
        Self {
            sql: sql.to_string(),
            values: Vec::new(),
            placeholder_count: 0,
        }
    }
}
impl From<Value> for i32 {
    fn from(v: Value) -> Self {
        match v {
            Value::Int(i) => i,
            Value::Int64(i) => i as i32,
            _ => panic!("Cannot convert Value to i32"),
        }
    }
}

impl From<Value> for i64 {
    fn from(v: Value) -> Self {
        match v {
            Value::Int(i) => i as i64,
            Value::Int64(i) => i,
            _ => panic!("Cannot convert Value to i64"),
        }
    }
}
impl From<Value> for String {
    fn from(v: Value) -> Self {
        match v {
            Value::String(s) => s,
            _ => panic!("Cannot convert Value to String, got {v:?}"),
        }
    }
}

impl From<Value> for chrono::NaiveDateTime {
    fn from(v: Value) -> Self {
        match v {
            Value::DateTime(dt) => dt,
            _ => panic!("Cannot convert Value to NaiveDateTime, got {v:?}"),
        }
    }
}

impl From<Value> for chrono::NaiveDate {
    fn from(v: Value) -> Self {
        match v {
            Value::Date(d) => d,
            _ => panic!("Cannot convert Value to NaiveDate, got {v:?}"),
        }
    }
}

impl From<Value> for bool {
    fn from(v: Value) -> Self {
        match v {
            Value::Bool(b) => b,
            _ => panic!("Cannot convert Value to bool, got {v:?}"),
        }
    }
}

impl From<Value> for f64 {
    fn from(v: Value) -> Self {
        match v {
            Value::Float(f) => f,
            _ => panic!("Cannot convert Value to f64, got {v:?}"),
        }
    }
}

#[cfg(feature = "sqlite")]
mod sqlx_impls {
    use super::Value;
    use sqlx::error::BoxDynError;
    use sqlx::sqlite::{Sqlite, SqliteTypeInfo, SqliteValueRef};
    use sqlx::{Decode, Type, TypeInfo, ValueRef};

    impl Type<Sqlite> for Value {
        fn type_info() -> SqliteTypeInfo {
            <String as Type<Sqlite>>::type_info()
        }

        fn compatible(_ty: &SqliteTypeInfo) -> bool {
            true
        }
    }

    impl<'r> Decode<'r, Sqlite> for Value {
        fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
            if value.is_null() {
                return Ok(Value::Null);
            }

            let type_name = value.type_info().name().to_uppercase();
            match type_name.as_str() {
                "INTEGER" => {
                    let v = <i64 as Decode<Sqlite>>::decode(value)?;
                    if (i32::MIN as i64..=i32::MAX as i64).contains(&v) {
                        Ok(Value::Int(v as i32))
                    } else {
                        Ok(Value::Int64(v))
                    }
                }
                "REAL" => Ok(Value::Float(<f64 as Decode<Sqlite>>::decode(value)?)),
                "TEXT" => {
                    let s = <String as Decode<Sqlite>>::decode(value)?;
                    if let Ok(dt) =
                        chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S%.f")
                    {
                        Ok(Value::DateTime(dt))
                    } else if let Ok(d) = chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d") {
                        Ok(Value::Date(d))
                    } else {
                        Ok(Value::String(s))
                    }
                }
                "BOOLEAN" | "BOOL" => Ok(Value::Bool(<bool as Decode<Sqlite>>::decode(value)?)),
                "DATE" => Ok(Value::Date(<chrono::NaiveDate as Decode<Sqlite>>::decode(
                    value,
                )?)),
                "DATETIME" | "TIMESTAMP" => {
                    Ok(Value::DateTime(<chrono::NaiveDateTime as Decode<
                        Sqlite,
                    >>::decode(value)?))
                }
                "NULL" => Ok(Value::Null),
                "BLOB" => Err("SQLite BLOB values are not supported by Value".into()),
                _ => Err(format!("Unsupported SQLite type for Value: {type_name}").into()),
            }
        }
    }
}
