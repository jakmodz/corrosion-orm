#[cfg(test)]
mod tests {
    use crate::{User, init_sqlite, user};
    use corrosion_orm_core::prelude::*;
    const USER_COUNT: usize = 5;
    async fn init_users(conn: &mut impl Executor, n: usize) -> Result<(), CorrosionOrmError> {
        for i in 0..n {
            let user = User {
                id: i as i32,
                name: format!("user{}", i),
            };
            user.save(conn).await?;
        }
        Ok(())
    }
    #[tokio::test]
    async fn test_find() -> Result<(), CorrosionOrmError> {
        let mut user = User::example();
        user.id = 99;
        let db = init_sqlite().await;
        let mut conn = db.acquire_conn().await.unwrap();
        init_users(&mut conn, USER_COUNT).await?;
        user.save(&mut conn).await.unwrap();

        let users = User::find()
            .filter(user::COLUMN.name.starts_with("user"))
            .all(&mut conn)
            .await?;
        assert_eq!(users.len(), USER_COUNT);
        Ok(())
    }
    #[tokio::test]
    async fn test_find_by_id() -> Result<(), CorrosionOrmError> {
        let mut user = User::example();
        user.id = 99;
        let db = init_sqlite().await;
        let mut conn = db.acquire_conn().await.unwrap();
        init_users(&mut conn, USER_COUNT).await?;
        user.save(&mut conn).await.unwrap();

        let users = User::find()
            .filter(user::COLUMN.id.eq(99))
            .all(&mut conn)
            .await?;
        assert_eq!(users.len(), 1);
        Ok(())
    }
    #[tokio::test]
    async fn test_find_id_between() -> Result<(), CorrosionOrmError> {
        let db = init_sqlite().await;
        let mut conn = db.acquire_conn().await.unwrap();
        init_users(&mut conn, USER_COUNT).await?;
        let users = User::find()
            .filter(user::COLUMN.id.between(1, 3))
            .all(&mut conn)
            .await?;
        assert_eq!(users.len(), 3);
        Ok(())
    }
    #[tokio::test]
    async fn test_find_one() -> Result<(), CorrosionOrmError> {
        let db = init_sqlite().await;
        let mut conn = db.acquire_conn().await.unwrap();
        init_users(&mut conn, USER_COUNT).await?;
        let user = User::find()
            .filter(user::COLUMN.id.eq(1))
            .one(&mut conn)
            .await?;
        assert_eq!(user.id, 1);
        Ok(())
    }
    #[tokio::test]
    async fn test_find_one_not_found() -> Result<(), CorrosionOrmError> {
        let db = init_sqlite().await;
        let mut conn = db.acquire_conn().await.unwrap();
        init_users(&mut conn, USER_COUNT).await?;
        let res = User::find()
            .filter(user::COLUMN.id.eq(99))
            .one(&mut conn)
            .await;
        assert!(res.is_err());
        Ok(())
    }
    #[tokio::test]
    async fn test_find_limit() -> Result<(), CorrosionOrmError> {
        let db = init_sqlite().await;
        let mut conn = db.acquire_conn().await.unwrap();
        init_users(&mut conn, USER_COUNT).await?;
        let users = User::find().limit(2).all(&mut conn).await?;
        assert_eq!(users.len(), 2);
        Ok(())
    }
}
