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
