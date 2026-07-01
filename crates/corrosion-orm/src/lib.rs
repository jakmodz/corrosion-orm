pub use corrosion_orm_core::prelude::*;
pub use corrosion_orm_core::*;
pub use corrosion_orm_macros::*;

#[cfg(feature = "sqlite")]
pub async fn connect(url: &str) -> Result<SqliteDriver, CorrosionOrmError> {
    let config = corrosion_orm_core::SqliteConfigBuilder::new()
        .url(url.to_string())
        .build();
    let driver = SqliteDriver::new(config).await?;
    Ok(driver)
}
