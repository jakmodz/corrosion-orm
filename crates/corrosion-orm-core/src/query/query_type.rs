use crate::dialect::sql_dialect::SqlDialect;
use crate::schema::table::TableSchemaModel;

/// Enum representing a SQL value of various types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    String(String),
    Int(i32),
    Int64(i64),
    Bool(bool),
    Date(chrono::NaiveDate),
    DateTime(chrono::NaiveDateTime),
}

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
    T: Default,
{
    fn from(v: Option<T>) -> Self {
        match v {
            Some(inner) => Value::from(inner),
            None => Value::from(T::default()),
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
                    Value::Bool(b) => if *b { "1" } else { "0" }.to_string(),
                    Value::Date(d) => format!("'{}'", d),
                    Value::DateTime(naive_date_time) => format!("'{}'", naive_date_time),
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
        ctx.sql = dialect.generate_ddl(&model).unwrap();
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
