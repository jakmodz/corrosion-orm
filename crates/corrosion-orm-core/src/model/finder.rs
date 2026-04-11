use sqlx::FromRow;
use std::marker::PhantomData;

use crate::{
    CorrosionOrmError, Executor,
    prelude::QueryContext,
    query::{Select, ToSql, WhereClause},
};

#[derive(Clone)]
/// Query result set from a database query for find() method.
///
/// This struct is used to execute a database query and return the results for find() method.
pub struct Finder<'query, T, E: Executor> {
    pub(crate) query: Select<'query>,
    _entity: PhantomData<T>,
    _executor: PhantomData<E>,
}

impl<'query, T, E: Executor> Finder<'query, T, E>
where
    T: Send + Unpin + for<'r> FromRow<'r, sqlx::sqlite::SqliteRow>,
{
    pub fn new(query: Select<'query>) -> Self {
        Self {
            query,
            _entity: PhantomData,
            _executor: PhantomData,
        }
    }
    pub fn limit(self, limit: usize) -> Self {
        Self {
            query: self.query.limit(limit),
            _entity: PhantomData,
            _executor: PhantomData,
        }
    }
    pub fn filter(self, filter: WhereClause<'query>) -> Self {
        Self {
            query: self.query.where_clause(filter),
            _entity: PhantomData,
            _executor: PhantomData,
        }
    }
    pub async fn one(self, executor: &mut E) -> Result<T, CorrosionOrmError> {
        let mut ctx = QueryContext::new();
        self.query.to_sql(&mut ctx, executor.get_dialect());
        let res = executor.fetch_one(&mut ctx).await?;
        Ok(res)
    }
    pub async fn all(self, executor: &mut E) -> Result<Vec<T>, CorrosionOrmError> {
        let mut ctx = QueryContext::new();
        self.query.to_sql(&mut ctx, executor.get_dialect());
        let res = executor.fetch_all(&mut ctx).await?;
        Ok(res)
    }
}
