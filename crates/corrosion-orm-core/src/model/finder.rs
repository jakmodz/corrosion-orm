use sqlx::FromRow;
use std::marker::PhantomData;

use crate::{
    CorrosionOrmError, Executor,
    model::{cursor_paginator::CursorPaginator, paginator::Paginator},
    prelude::QueryContext,
    query::{Select, ToSql, WhereClause, order_by::OrderBy},
    types::ColumnTrait,
};

/// Query result set from a database query for find() method.
///
/// This struct is now generic over C: ColumnTrait to ensure model-level type safety.
pub struct Finder<'query, T, E: Executor, C: ColumnTrait> {
    pub(crate) query: Select<'query, C>,
    _entity: PhantomData<T>,
    _executor: PhantomData<E>,
}
impl<'query, T, E: Executor, C: ColumnTrait> Clone for Finder<'query, T, E, C>
where
    Select<'query, C>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            query: self.query.clone(),
            _entity: PhantomData,
            _executor: PhantomData,
        }
    }
}
impl<'query, T, E: Executor, C: ColumnTrait> Finder<'query, T, E, C>
where
    T: Send + Unpin + for<'r> FromRow<'r, sqlx::sqlite::SqliteRow>,
{
    pub fn new(query: Select<'query, C>) -> Self {
        Self {
            query,
            _entity: PhantomData,
            _executor: PhantomData,
        }
    }

    /// Limits the number of rows returned by the query.
    pub fn limit(self, limit: usize) -> Self {
        Self {
            query: self.query.limit(limit),
            _entity: PhantomData,
            _executor: PhantomData,
        }
    }

    /// Filters the query using the given [`WhereClause`].
    ///
    /// Because Finder is generic over C, it now enforces that 'filter'
    /// must be a WhereClause bound to the same ColumnTrait (C).
    pub fn filter(self, filter: WhereClause<C>) -> Self {
        Self {
            query: self.query.where_clause(filter),
            _entity: PhantomData,
            _executor: PhantomData,
        }
    }
    /// Adds an order to order clause in the query.
    pub fn add_order_by(self, order_by: OrderBy<C>) -> Self {
        Self {
            query: self.query.add_order_by(order_by),
            _entity: PhantomData,
            _executor: PhantomData,
        }
    }
    /// Sets the offset for the query.
    pub fn offset(self, offset: usize) -> Self {
        Self {
            query: self.query.offset(offset),
            _entity: PhantomData,
            _executor: PhantomData,
        }
    }
    /// Returns a paginator for this finder with the given page size.
    pub fn paginate(self, page_size: usize) -> Paginator<'query, T, E, C> {
        Paginator::new(self, page_size)
    }
    /// Returns a cursor paginator for this finder with the given page size.
    pub fn cursor_paginate(self, page_size: usize) -> CursorPaginator<'query, T, E, C>
    where
        T: Clone,
    {
        CursorPaginator::new(self, page_size)
    }
    /// Fetches a single row from the query.
    pub async fn one(self, executor: &mut E) -> Result<T, CorrosionOrmError> {
        let mut ctx = QueryContext::new();
        self.query.to_sql(&mut ctx, executor.get_dialect());
        let res = executor.fetch_one(&mut ctx).await?;
        Ok(res)
    }

    /// Fetches all rows from the query.
    pub async fn all(self, executor: &mut E) -> Result<Vec<T>, CorrosionOrmError> {
        let mut ctx = QueryContext::new();
        self.query.to_sql(&mut ctx, executor.get_dialect());
        let res = executor.fetch_all(&mut ctx).await?;
        Ok(res)
    }
}
