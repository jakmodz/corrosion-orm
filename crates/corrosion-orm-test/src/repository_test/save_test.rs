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
    #[tokio::test]
    async fn test_save_verifies_saved() {
        let user = User::example();
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await.unwrap();
        user.save(&mut conn).await.unwrap();
        drop(conn);
        let mut conn = driver.acquire_conn().await.unwrap();
        let result = User::get_by_id(1, &mut conn).await;
        assert!(result.is_ok());
    }
    #[tokio::test]
    async fn test_save_update() -> Result<(), CorrosionOrmError> {
        let user = User::example();
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await.unwrap();
        user.save(&mut conn).await.unwrap();
        drop(conn);
        let mut conn = driver.acquire_conn().await.unwrap();
        let mut updated_user = User::example();
        updated_user.name = String::from("Alice");
        updated_user.save(&mut conn).await.unwrap();
        let result = User::get_by_id(1, &mut conn).await?.unwrap();
        assert_eq!(result.name, "Alice");
        Ok(())
    }
    #[tokio::test]
    async fn test_save_with_transaction_commit() -> Result<(), CorrosionOrmError> {
        let user = User::example();
        let driver = init_sqlite().await;

        let mut tx = driver.transaction().await.unwrap();
        user.save(&mut tx).await.unwrap();
        tx.commit().await.unwrap();

        let mut conn = driver.acquire_conn().await.unwrap();
        let result = User::get_by_id(1, &mut conn).await?.unwrap();
        assert_eq!(result.id, 1);
        Ok(())
    }

    #[tokio::test]
    async fn test_save_with_transaction_rollback() -> Result<(), CorrosionOrmError> {
        let user = User::example();
        let driver = init_sqlite().await;

        let mut tx = driver.transaction().await.unwrap();
        user.save(&mut tx).await.unwrap();
        tx.rollback().await.unwrap();

        let mut conn = driver.acquire_conn().await.unwrap();
        let result = User::get_by_id(1, &mut conn).await?;
        assert!(result.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn test_save_update_with_transaction_commit() -> Result<(), CorrosionOrmError> {
        let user = User::example();
        let driver = init_sqlite().await;

        let mut conn = driver.acquire_conn().await.unwrap();
        user.save(&mut conn).await.unwrap();
        drop(conn);

        let mut tx = driver.transaction().await.unwrap();
        let mut updated_user = User::get_by_id(1, &mut tx).await?.unwrap();
        updated_user.name = String::from("Alice");
        updated_user.save(&mut tx).await.unwrap();
        tx.commit().await.unwrap();

        let mut conn = driver.acquire_conn().await.unwrap();
        let result = User::get_by_id(1, &mut conn).await?.unwrap();
        assert_eq!(result.name, "Alice");
        Ok(())
    }

    #[tokio::test]
    async fn test_save_multiple_with_transaction_commit() -> Result<(), CorrosionOrmError> {
        let driver = init_sqlite().await;

        let mut tx = driver.transaction().await.unwrap();

        let user1 = User {
            id: 1,
            name: String::from("Alice"),
        };
        user1.save(&mut tx).await.unwrap();

        let user2 = User {
            id: 2,
            name: String::from("Bob"),
        };
        user2.save(&mut tx).await.unwrap();

        let user3 = User {
            id: 3,
            name: String::from("Charlie"),
        };
        user3.save(&mut tx).await.unwrap();

        tx.commit().await.unwrap();

        let mut conn = driver.acquire_conn().await.unwrap();
        let result1 = User::get_by_id(1, &mut conn).await;
        let result2 = User::get_by_id(2, &mut conn).await;
        let result3 = User::get_by_id(3, &mut conn).await;

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result3.is_ok());
        assert_eq!(result1?.unwrap().name, "Alice");
        assert_eq!(result2?.unwrap().name, "Bob");
        assert_eq!(result3?.unwrap().name, "Charlie");
        Ok(())
    }
}
