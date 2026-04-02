#[cfg(test)]
mod tests {
    use crate::{User, init_sqlite};
    use corrosion_orm_core::prelude::*;

    #[tokio::test]
    async fn test_delete() {
        let user = User::example();
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await.unwrap();
        user.save(&mut conn).await.unwrap();
        user.delete(&mut conn).await.unwrap();
    }
}
