use crate::{driver::error::DriverError, error::CorrosionOrmError, query::query_type::Value};

pub struct DbRow {
    pub columns: Vec<(String, Value)>,
}
impl DbRow {
    pub fn new(columns: Vec<(String, Value)>) -> Self {
        Self { columns }
    }

    pub fn try_get<T>(&self, name: &str) -> Result<T, CorrosionOrmError>
    where
        T: TryFrom<Value, Error = String>,
    {
        self.columns
            .iter()
            .find(|(col, _)| col == name)
            .map(|(_, val)| val.clone())
            .ok_or_else(|| {
                CorrosionOrmError::DriverError(DriverError::ColumnNotFound(name.to_string()))
            })
            .and_then(|v| {
                T::try_from(v)
                    .map_err(|e| CorrosionOrmError::DriverError(DriverError::ValueConversion(e)))
            })
    }
    pub fn try_get_optional<T>(&self, name: &str) -> Result<Option<T>, CorrosionOrmError>
    where
        T: TryFrom<Value, Error = String>,
    {
        let val = self
            .columns
            .iter()
            .find(|(col, _)| col == name)
            .map(|(_, val)| val.clone())
            .ok_or_else(|| {
                CorrosionOrmError::DriverError(DriverError::ColumnNotFound(name.to_string()))
            })?;

        match val {
            Value::Null => Ok(None),
            other => Ok(Some(T::try_from(other).map_err(|e| {
                CorrosionOrmError::DriverError(DriverError::ValueConversion(e))
            })?)),
        }
    }
}
