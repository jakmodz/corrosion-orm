//! Cursor-based pagination for database queries.
//!
//! This module provides the [`CursorPaginator`] struct, which implements efficient
//! cursor-based pagination using keyset pagination (also known as "seek-method" or
//! "keyset filtering"). This pagination strategy is ideal for high-performance APIs
//! and datasets that change frequently.
//!
//! # Understanding Cursor-Based Pagination
//!
//! Unlike offset-based pagination that uses `OFFSET` to skip rows, cursor-based
//! pagination uses the values of the last row to seek to the next set of records.
//! This approach offers several advantages:
//! - **Consistent O(1) performance** - Each page takes the same time to fetch
//! - **Handles real-time updates** - New/deleted rows don't cause pagination inconsistencies
//! - **Scalability** - Works efficiently with large datasets and deep pagination
//!
//! # When to Use Cursor-Based Pagination
//!
//! Cursor-based pagination with [`CursorPaginator`] is best suited for:
//! - **Large result sets** (100K+ rows)
//! - **High-performance APIs** requiring consistent response times
//! - **Real-time data** that changes during pagination
//! - **Deep pagination** (fetching page 1000+)
//! - **Infinite scroll patterns** in web/mobile applications
//! - **Streaming scenarios** where rows are continuously added
//!
//! # When NOT to Use Cursor-Based Pagination
//!
//! Cursor-based pagination is **not ideal** for:
//! - **Jumping to arbitrary pages** (you must paginate sequentially)
//! - **Small datasets** where simpler offset pagination is sufficient
//! - **User interfaces** requiring page numbers (1, 2, 3, etc.)
//! - **Accessing pages in random order**
//!
//! In these cases, consider using [`crate::model::Paginator`] instead.
//!
//! # Sort and Unique Columns
//!
//! Cursor pagination requires two columns:
//!
//! 1. **Sort Column** - The column used to order results. Can be non-unique.
//!    - Examples: `created_at`, `score`, `priority`
//!    - Used to determine result ordering
//!
//! 2. **Unique Column** - A column that uniquely identifies each row.
//!    - Examples: `id`, `uuid`
//!    - Must be unique and immutable
//!    - Used to break ties when sort column values are equal
//!    - Ensures reproducible pagination even with duplicate sort values
//!
//! Together, these columns form a composite sort key that:
//! - Uniquely identifies each row: `(sort_col, unique_col)`
//! - Ensures pagination doesn't skip or repeat rows
//! - Allows efficient index-based seeking
//!
//! # Performance Notes
//!
//! Cursor pagination typically delivers:
//! - **Constant time per page** regardless of total dataset size
//! - **Efficient database index usage** with proper column indexing
//! - **Reduced memory overhead** compared to offset-based approaches
//!
//! For optimal performance:
//! - Index the sort column: `CREATE INDEX idx_col_sorted ON table(sort_col, unique_col)`
//! - Use immutable unique columns (typically auto-increment IDs)

use crate::{
    CorrosionOrmError, Executor, model::Finder, prelude::Value, query::WhereClause,
    types::ColumnTrait,
};

pub struct CursorPaginator<'query, T, E: Executor, C: ColumnTrait> {
    finder: Finder<'query, T, E, C>,
    page_size: usize,
    last_row: Option<T>,
}

impl<'query, T, E: Executor, C: ColumnTrait> CursorPaginator<'query, T, E, C>
where
    T: Send + Unpin + Clone + for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>,
{
    pub fn new(finder: Finder<'query, T, E, C>, page_size: usize) -> Self {
        Self {
            finder,
            page_size,
            last_row: None,
        }
    }

    /// Fetches the next page based on the last row retrieved.
    pub async fn fetch_next(
        &mut self,
        db: &mut E,
        get_sort_val: impl Fn(&T) -> Value,
        get_unique_val: impl Fn(&T) -> Value,
        sort_col: C,
        unique_col: C,
    ) -> Result<Option<Vec<T>>, CorrosionOrmError> {
        let mut query = self.finder.clone().limit(self.page_size);

        if let Some(last) = &self.last_row {
            let sort_val = get_sort_val(last);
            let unique_val = get_unique_val(last);

            query = query.filter(WhereClause::seek_after(
                sort_col, sort_val, unique_col, unique_val,
            ));
        }

        let results = query.all(db).await?;

        if results.is_empty() {
            Ok(None)
        } else {
            self.last_row = results.last().cloned();
            Ok(Some(results))
        }
    }
}
