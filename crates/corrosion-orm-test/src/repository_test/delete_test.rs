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
        assert!(User::get_by_id(1, &mut conn).await.is_err());
    }
    #[tokio::test]
    async fn test_delete_not_found() {
        let user = User::example();
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await.unwrap();
        user.delete(&mut conn).await.unwrap();
    }
    #[tokio::test]
    async fn test_delete_verifies_removed() {
        let driver = init_sqlite().await;
        let user = User::example();

        let mut conn = driver.acquire_conn().await.unwrap();
        user.save(&mut conn).await.unwrap();
        drop(conn);

        let mut conn = driver.acquire_conn().await.unwrap();
        user.delete(&mut conn).await.unwrap();
        drop(conn);

        let mut conn = driver.acquire_conn().await.unwrap();
        let result = User::get_by_id(1, &mut conn).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_with_transaction() {
        let user = User::example();
        let driver = init_sqlite().await;

        let mut tx = driver.transaction().await.unwrap();
        user.save(&mut tx).await.unwrap();

        let retrieved_user = User::get_by_id(1, &mut tx).await.unwrap();
        assert_eq!(retrieved_user.id, 1);

        retrieved_user.delete(&mut tx).await.unwrap();
        tx.commit().await.unwrap();

        let mut conn = driver.acquire_conn().await.unwrap();
        assert!(User::get_by_id(1, &mut conn).await.is_err());
    }
    #[tokio::test]
    async fn test_delete_with_transaction_not_found() {
        let driver = init_sqlite().await;
        let mut tx = driver.transaction().await.unwrap();
        let res = User::get_by_id(1, &mut tx).await;
        assert!(res.is_err());
        tx.commit().await.unwrap();
    }
    #[tokio::test]
    async fn test_delete_with_transaction_rollback() {
        let user = User::example();
        let driver = init_sqlite().await;
        let mut tx = driver.transaction().await.unwrap();
        user.save(&mut tx).await.unwrap();
        tx.rollback().await.unwrap();
        let mut conn = driver.acquire_conn().await.unwrap();
        assert!(User::get_by_id(1, &mut conn).await.is_err());
    }
}
