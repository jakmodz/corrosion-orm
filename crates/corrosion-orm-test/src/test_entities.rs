use corrosion_orm_core::{SqliteConfigBuilder, SqliteDriver, dialect::sql_dialect::SqlDialect};
use corrosion_orm_macros::Model;
#[allow(dead_code)]
pub(crate) struct MockSqliteDialect;
impl SqlDialect for MockSqliteDialect {
    fn cast_type(&self, _: &corrosion_orm_core::types::column_type::SqlType) -> String {
        String::new()
    }

    /// Provide the SQLite bind-parameter placeholder used in prepared statements.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let dialect = MockSqliteDialect;
    /// let placeholder = dialect.bind_param(&0);
    /// assert_eq!(placeholder, "?");
    /// ```
    ///
    /// # Returns
    ///
    /// The string `"?"` representing the SQLite parameter placeholder.
    fn bind_param(&self, _count: &usize) -> String {
        "?".to_string()
    }
}
#[derive(Model, Clone, Debug, Default)]
#[Table(name = "users")]
#[Index(name = "idx_users_id", fields = ["id"], unique = true)]
pub struct User {
    #[Column(name = "id")]
    #[PrimaryKey]
    #[allow(unused)]
    pub id: i32,
    #[Column(name = "name", unique, nullable)]
    #[allow(unused)]
    pub name: String,
}

#[derive(Model, Clone)]
pub struct Post {
    #[Column(name = "id")]
    #[PrimaryKey]
    pub id: i32,
    #[Column(name = "teacher_id")]
    pub teacher_id: i64,
    #[BelongsTo(foreign_key = "user_id", table = "users")]
    pub user: User,
}

#[derive(Model, Default)]
pub struct Teacher {
    #[Column(name = "id")]
    #[PrimaryKey]
    pub id: i64,
    #[Column(name = "name")]
    pub name: String,
    #[HasMany(foreign_key = "teacher_id", table = "Post")]
    pub posts: Vec<Post>,
}
impl User {
    /// Creates an example `User` populated with sample values.
    ///
    /// The returned value has `id = 1` and `name = "Bob"`.
    ///
    /// # Examples
    ///
    /// ```
    /// use corrosion_orm_test::test_entities::User;
    /// let u = User::example();
    /// assert_eq!(u.id, 1);
    /// assert_eq!(u.name, "Bob");
    /// ```
    #[allow(dead_code)]
    pub fn example() -> Self {
        Self {
            id: 1,
            name: String::from("Bob"),
        }
    }
}
/// Initializes an in-memory SQLite driver and creates tables for the test models.
///
/// The function configures logging for tests, constructs an in-memory SQLite configuration,
/// creates a `SqliteDriver`, and executes the schema creation queries for `User`, `Post`,
/// and `Teacher`.
///
/// # Examples
///
/// ```no_run
/// use corrosion_orm_test::test_entities::init_sqlite;
///
/// #[tokio::test]
/// async fn init_sqlite_example() {
///     let _driver = init_sqlite().await;
/// }
/// ```
///
/// # Returns
///
/// An initialized `SqliteDriver` connected to an in-memory database with schemas for
/// `User`, `Post`, and `Teacher` created.
#[allow(dead_code)]
pub async fn init_sqlite() -> SqliteDriver {
    use corrosion_orm_core::SqliteDriver;
    use corrosion_orm_core::prelude::*;

    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::max())
        .filter_module("sqlx", log::LevelFilter::Off)
        .try_init();

    let config = SqliteConfigBuilder::new()
        .url(String::from(":memory:"))
        .build();
    let driver = SqliteDriver::new(config).await.unwrap();
    let mut ctx = QueryContext::from_model(
        User::get_schema(),
        driver.acquire_conn().await.unwrap().get_dialect(),
    );

    driver
        .acquire_conn()
        .await
        .unwrap()
        .execute_query(&mut ctx)
        .await
        .unwrap();

    let mut ctx = QueryContext::from_model(
        Post::get_schema(),
        driver.acquire_conn().await.unwrap().get_dialect(),
    );

    driver
        .acquire_conn()
        .await
        .unwrap()
        .execute_query(&mut ctx)
        .await
        .unwrap();

    let mut ctx = QueryContext::from_model(
        Teacher::get_schema(),
        driver.acquire_conn().await.unwrap().get_dialect(),
    );

    driver
        .acquire_conn()
        .await
        .unwrap()
        .execute_query(&mut ctx)
        .await
        .unwrap();
    driver
}
