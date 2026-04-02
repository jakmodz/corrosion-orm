use corrosion_orm_core::{SqliteConfigBuilder, SqliteDriver, dialect::sql_dialect::SqlDialect};
use corrosion_orm_macros::Model;
#[allow(dead_code)]
pub(crate) struct MockSqliteDialect;
impl SqlDialect for MockSqliteDialect {
    fn cast_type(&self, _: &corrosion_orm_core::types::column_type::SqlType) -> String {
        String::new()
    }

    fn bind_param(&self, _count: &usize) -> String {
        "?".to_string()
    }
}
#[derive(Model, Clone, Debug)]
#[Table(name = "users")]
pub struct User {
    #[Column(name = "id")]
    #[PrimaryKey]
    #[allow(unused)]
    pub id: i32,
    #[Column(name = "name", unique, nullable)]
    #[allow(unused)]
    pub name: String,
}
impl User {
    #[allow(dead_code)]
    pub(crate) fn example() -> Self {
        Self {
            id: 1,
            name: String::from("Bob"),
        }
    }
}
#[allow(dead_code)]
pub(crate) async fn init_sqlite() -> SqliteDriver {
    use corrosion_orm_core::SqliteDriver;
    use corrosion_orm_core::prelude::*;
    let _ = env_logger::builder().is_test(true).try_init();

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
    driver
}
