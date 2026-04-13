//! Offset-based pagination for database queries.
//!
//! This module provides the [`Paginator`] struct, which implements traditional
//! offset-based pagination using page numbers and SQL `OFFSET` clauses. This is the
//! classic pagination strategy familiar to most web developers.
//!
//! # Understanding Offset-Based Pagination
//!
//! Offset-based pagination works by:
//! 1. Taking a `page` number (0-indexed)
//! 2. Calculating offset = `page × page_size`
//! 3. Using `OFFSET offset LIMIT page_size` in the SQL query
//! 4. Returning the results for that page
//!
//! # When to Use Offset-Based Pagination
//!
//! Offset-based pagination with [`Paginator`] is best suited for:
//! - **Small to medium datasets** (< 100,000 rows)
//! - **Admin panels and dashboards** with manageable record counts
//! - **Traditional user interfaces** showing page numbers (1, 2, 3, ...)
//! - **Allowing arbitrary page access** (jump to page 5 directly)
//! - **Relatively static data** that doesn't change frequently
//! - **Simple implementations** where ease-of-use is valued
//!
//! # When NOT to Use Offset-Based Pagination
//!
//! Offset-based pagination is **not ideal** for:
//! - **Large datasets** (millions of rows) - Later pages become very slow
//! - **High-performance APIs** - OFFSET performance degrades with page depth
//! - **Real-time data** with frequent inserts/deletes - Can cause duplicate/missing rows
//! - **Infinite scroll patterns** - Better served by cursor pagination
//! - **Deep pagination** (page 1000+) - Extremely slow as database skips many rows
//!
//! In these cases, consider using [`crate::model::CursorPaginator`] instead.
//!
//! # Performance Characteristics
//!
//! - **Early pages** (0-10): Fast, ~O(n) where n = page_size
//! - **Middle pages** (100-1000): Noticeably slower, ~O(offset + page_size)
//! - **Late pages** (10000+): Very slow, database must scan/skip many rows
//!
//! This is because the database must process every row up to the offset point,
//! even though most are discarded.
//!
//! # Example: Basic Usage
//!
//! ```ignore
//! use corrosion_orm_core::model::Paginator;
//!
//! let mut paginator = Paginator::new(
//!     User::find().order_by(user::COLUMN.CreatedAt.desc()),
//!     20 // 20 items per page
//! );
//!
//! // Jump to page 5
//! let page_5 = paginator.fetch_page(&mut db, 5).await?;
//! for user in page_5 {
//!     println!("{}: {}", user.id, user.email);
//! }
//! ```
//!
//! # Example: Sequential Navigation
//!
//! ```ignore
//! use corrosion_orm_core::model::Paginator;
//!
//! let mut paginator = Paginator::new(
//!     Post::find().order_by(post::COLUMN.CreatedAt.desc()),
//!     50
//! );
//!
//! // Iterate through pages sequentially
//! while let Some(posts) = paginator.fetch_next(&mut db).await? {
//!     for post in posts {
//!         println!("Post: {}", post.title);
//!     }
//! }
//! ```
//!
//! # See Also
//!
//! - [`crate::model::CursorPaginator`] - Cursor-based pagination for large datasets,
//!   real-time data, and consistent O(1) performance
use crate::{CorrosionOrmError, Executor, model::Finder, types::ColumnTrait};

pub struct Paginator<'query, T, E: Executor, C: ColumnTrait> {
    finder: Finder<'query, T, E, C>,
    page_size: usize,
    current_page: usize,
}

impl<'query, T, E: Executor, C: ColumnTrait> Paginator<'query, T, E, C>
where
    T: Send + Unpin + for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>,
{
    pub fn new(finder: Finder<'query, T, E, C>, page_size: usize) -> Self {
        Self {
            finder,
            page_size,
            current_page: 0,
        }
    }
    /// Fetch the page of results from the database.
    ///
    /// page parameter is index of page to fetch.
    pub async fn fetch_page(
        &mut self,
        db: &mut E,
        page: usize,
    ) -> Result<Vec<T>, CorrosionOrmError> {
        self.current_page = page;
        let offset = self.current_page * self.page_size;
        let res = self
            .finder
            .clone()
            .offset(offset)
            .limit(self.page_size)
            .all(db)
            .await?;
        Ok(res)
    }
    /// Fetches the next page of results from the database.
    ///
    /// Returns `None` if there are no more pages.
    pub async fn fetch_next(&mut self, db: &mut E) -> Result<Option<Vec<T>>, CorrosionOrmError> {
        let res = self.fetch_page(db, self.current_page).await?;
        if res.is_empty() {
            Ok(None)
        } else {
            self.current_page += 1;
            Ok(Some(res))
        }
    }
}
