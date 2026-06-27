use crate::{error::CorrosionOrmError, query::query_type::Value};

pub struct DbRow {
    pub columns: Vec<(String, Value)>,
}
impl DbRow {
    pub fn new(columns: Vec<(String, Value)>) -> Self {
        Self { columns }
    }

    pub fn try_get<T>(&self, name: &str) -> Result<T, CorrosionOrmError>
    where
        T: From<Value>,
    {
        self.columns
            .iter()
            .find(|(col, _)| col == name)
            .map(|(_, val)| val.clone())
            .ok_or_else(|| {
                CorrosionOrmError::DriverError(super::error::DriverError::ColumnNotFound(
                    name.to_string(),
                ))
            })
            .map(|v| T::from(v))
    }
    pub fn try_get_optional<T>(&self, name: &str) -> Result<Option<T>, CorrosionOrmError>
    where
        T: From<Value>,
    {
        let val = self
            .columns
            .iter()
            .find(|(col, _)| col == name)
            .map(|(_, val)| val.clone())
            .ok_or_else(|| {
                CorrosionOrmError::DriverError(super::error::DriverError::ColumnNotFound(
                    name.to_string(),
                ))
            })?;

        match val {
            Value::Null => Ok(None),
            other => Ok(Some(T::from(other))),
        }
    }
}
