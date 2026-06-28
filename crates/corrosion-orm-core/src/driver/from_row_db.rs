use crate::{driver::db_row::DbRow, error::CorrosionOrmError};

pub trait FromRowDb: Sized {
    fn from_row(row: &DbRow) -> Result<Self, CorrosionOrmError>;
}
