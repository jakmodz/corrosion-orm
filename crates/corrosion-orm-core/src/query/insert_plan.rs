use std::borrow::Cow;

use crate::query::{insert::Insert, query_type::Value};

#[derive(Debug, Clone)]
pub struct InsertPlan<'a> {
    pub table: &'a str,
    pub columns: Vec<String>,
    pub values: Vec<Value>,
}

impl<'a> InsertPlan<'a> {
    pub fn new(table: &'a str) -> Self {
        Self {
            table,
            columns: Vec::new(),
            values: Vec::new(),
        }
    }
    pub fn to_insert(&self) -> Insert<'a> {
        Insert::new(self.table)
            .columns(
                self.columns
                    .iter()
                    .map(|col| Cow::Owned(col.clone()))
                    .collect(),
            )
            .values(self.values.clone())
    }
}

pub trait InsertPlanGenerator {
    fn generate_insert_plan(&self, values: Vec<Value>) -> InsertPlan<'_>;
}
