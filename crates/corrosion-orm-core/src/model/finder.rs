use std::marker::PhantomData;

use crate::{
    CorrosionOrmError, Executor,
    driver::from_row_db::FromRowDb,
    model::{
        CacheModel, cursor_paginator::CursorPaginator, paginator::Paginator, repository::Repo,
    },
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
    T: Send + Unpin + FromRowDb + Repo<E>,
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
    pub fn filter(self, filter: WhereClause) -> Self {
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
    pub fn paginate(self, page_size: usize) -> Paginator<'query, T, E, C>
    where
        T: CacheModel + Clone,
    {
        Paginator::new(self, page_size)
    }
    /// Returns a cursor paginator for this finder with the given page size.
    pub fn cursor_paginate(self, page_size: usize) -> CursorPaginator<'query, T, E, C>
    where
        T: CacheModel + Clone,
    {
        CursorPaginator::new(self, page_size)
    }
    /// Fetches a single row from the query.
    #[cfg(feature = "cache")]
    pub async fn one(self, executor: &mut E) -> Result<T, CorrosionOrmError>
    where
        T: CacheModel + Clone + Repo<E>,
    {
        let cache_scope = crate::model::cache::scope_id(executor);
        let mut ctx = QueryContext::new();
        self.query.to_sql(&mut ctx, executor.get_dialect());

        let mut res = executor.fetch_one::<T>(&mut ctx).await?;
        res.load_relations(executor).await?;
        crate::model::cache::put_entity(cache_scope, &res).await;
        Ok(res)
    }

    /// Fetches all rows from the query.
    #[cfg(feature = "cache")]
    pub async fn all(self, executor: &mut E) -> Result<Vec<T>, CorrosionOrmError>
    where
        T: CacheModel + Clone + Repo<E>,
    {
        let cache_scope = crate::model::cache::scope_id(executor);
        let mut ctx = QueryContext::new();
        self.query.to_sql(&mut ctx, executor.get_dialect());

        let mut res = executor.fetch_all::<T>(&mut ctx).await?;
        for item in &mut res {
            item.load_relations(executor).await?;
            crate::model::cache::put_entity(cache_scope, item).await;
        }
        Ok(res)
    }
    #[cfg(not(feature = "cache"))]
    pub async fn one(self, executor: &mut E) -> Result<T, CorrosionOrmError>
    where
        T: FromRowDb + Repo<E>,
    {
        let mut ctx = QueryContext::new();
        self.query.to_sql(&mut ctx, executor.get_dialect());
        let mut res = executor.fetch_one::<T>(&mut ctx).await?;
        res.load_relations(executor).await?;
        Ok(res)
    }
    #[cfg(not(feature = "cache"))]
    pub async fn all(self, executor: &mut E) -> Result<Vec<T>, CorrosionOrmError>
    where
        T: FromRowDb + Repo<E>,
    {
        let mut ctx = QueryContext::new();
        self.query.to_sql(&mut ctx, executor.get_dialect());
        let mut res = executor.fetch_all::<T>(&mut ctx).await?;
        for item in &mut res {
            item.load_relations(executor).await?;
        }
        Ok(res)
    }
}
