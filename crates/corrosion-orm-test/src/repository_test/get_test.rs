#[cfg(test)]
mod tests {
    use crate::{init_sqlite, test_entities::User};
    use corrosion_orm_core::prelude::*;

    #[tokio::test]
    async fn test_get_by_id() {
        let driver = init_sqlite().await;

        let user = User::example();
        let mut conn = driver.acquire_conn().await.unwrap();
        user.save(&mut conn).await.unwrap();
        let db_user = User::get_by_id(user.id, &mut conn).await.unwrap();
        assert_eq!(user.id, db_user.id);
    }
    #[tokio::test]
    async fn test_get_by_id_not_found() {
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await.unwrap();
        let result = User::get_by_id(999, &mut conn).await;
        assert!(result.is_err());
    }
    #[tokio::test]
    async fn test_get_all() {
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await.unwrap();
        let user = User::example();
        user.save(&mut conn).await.unwrap();
        let users = User::get_all(&mut conn).await.unwrap();
        assert_eq!(1, users.len());
    }
    #[tokio::test]
    async fn test_get_all_empty() {
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await.unwrap();
        let users = User::get_all(&mut conn).await.unwrap();
        assert!(users.is_empty());
    }
    #[tokio::test]
    async fn test_get_all_multiple() {
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await.unwrap();
        for i in 0..10 {
            let user = User {
                id: i,
                name: format!("user{}", i),
            };
            user.save(&mut conn).await.unwrap();
        }
        let users = User::get_all(&mut conn).await.unwrap();
        assert_eq!(10, users.len());
        for (i, user) in users.iter().enumerate() {
            assert_eq!(i, user.id as usize);
        }
    }

    #[tokio::test]
    async fn test_get_by_id_with_transaction() {
        let user = User::example();
        let driver = init_sqlite().await;

        let mut tx = driver.transaction().await.unwrap();
        user.save(&mut tx).await.unwrap();

        let retrieved_user = User::get_by_id(user.id, &mut tx).await.unwrap();
        assert_eq!(retrieved_user.id, user.id);

        tx.commit().await.unwrap();

        let mut conn = driver.acquire_conn().await.unwrap();
        let db_user = User::get_by_id(user.id, &mut conn).await.unwrap();
        assert_eq!(user.id, db_user.id);
    }

    #[tokio::test]
    async fn test_get_all_with_transaction() {
        let driver = init_sqlite().await;

        let mut tx = driver.transaction().await.unwrap();
        for i in 0..5 {
            let user = User {
                id: i,
                name: format!("user{}", i),
            };
            user.save(&mut tx).await.unwrap();
        }

        let users = User::get_all(&mut tx).await.unwrap();
        assert_eq!(5, users.len());

        tx.commit().await.unwrap();

        let mut conn = driver.acquire_conn().await.unwrap();
        let db_users = User::get_all(&mut conn).await.unwrap();
        assert_eq!(5, db_users.len());
    }

    #[tokio::test]
    async fn test_get_with_transaction_not_found() {
        let driver = init_sqlite().await;
        let mut tx = driver.transaction().await.unwrap();
        let result = User::get_by_id(999, &mut tx).await;
        assert!(result.is_err());
        tx.commit().await.unwrap();
    }

    #[tokio::test]
    async fn test_get_with_transaction_after_rollback() {
        let user = User::example();
        let driver = init_sqlite().await;

        let mut tx = driver.transaction().await.unwrap();
        user.save(&mut tx).await.unwrap();
        tx.rollback().await.unwrap();

        let mut conn = driver.acquire_conn().await.unwrap();
        let result = User::get_by_id(user.id, &mut conn).await;
        assert!(result.is_err());
    }
}
