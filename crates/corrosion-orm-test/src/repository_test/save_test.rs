#[cfg(test)]
mod tests {
    use crate::{init_sqlite, test_entities::User};
    use corrosion_orm_core::prelude::*;

    #[tokio::test]
    async fn test_save() {
        let user = User::example();
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await.unwrap();
        user.save(&mut conn).await.unwrap();
        let _saved_user = user.save(&mut conn).await.unwrap();
    }
}
