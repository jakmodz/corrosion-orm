#[cfg(test)]
mod tests {
    use crate::{User, init_sqlite, user};
    use corrosion_orm_core::prelude::*;
    use corrosion_orm_macros::Model;
    const USER_COUNT: usize = 5;
    async fn init_users(conn: &mut impl Executor, n: usize) -> Result<(), CorrosionOrmError> {
        for i in 1..n + 1 {
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
    #[tokio::test]
    async fn test_find_order_by_asc() -> Result<(), CorrosionOrmError> {
        let db = init_sqlite().await;
        let mut conn = db.acquire_conn().await.unwrap();
        init_users(&mut conn, USER_COUNT).await?;
        let users = User::find()
            .add_order_by(user::COLUMN.id.asc())
            .all(&mut conn)
            .await?;
        assert_eq!(users.first().unwrap().id, 1);
        Ok(())
    }
    #[tokio::test]
    async fn test_find_order_by_desc() -> Result<(), CorrosionOrmError> {
        let db = init_sqlite().await;
        let mut conn = db.acquire_conn().await.unwrap();
        init_users(&mut conn, USER_COUNT).await?;
        let users = User::find()
            .add_order_by(user::COLUMN.id.desc())
            .all(&mut conn)
            .await?;
        assert_eq!(users.first().unwrap().id, USER_COUNT as i32);
        Ok(())
    }
    #[tokio::test]
    async fn test_find_pagination() -> Result<(), CorrosionOrmError> {
        let db = init_sqlite().await;
        let mut conn = db.acquire_conn().await.unwrap();
        init_users(&mut conn, USER_COUNT).await?;
        let mut paginator = User::find().add_order_by(user::COLUMN.id.asc()).paginate(2);
        let page1 = paginator.fetch_page(&mut conn, 0).await?;
        assert_eq!(page1[0].id, 1);
        assert_eq!(page1[1].id, 2);
        let page2 = paginator.fetch_page(&mut conn, 1).await?;
        assert_eq!(page2[0].id, 3);
        assert_eq!(page2[1].id, 4);
        let page3 = paginator.fetch_page(&mut conn, 2).await?;
        assert_eq!(page3.len(), 1);
        assert_eq!(page3[0].id, 5);
        Ok(())
    }
    #[tokio::test]
    async fn test_find_pagination_next() -> Result<(), CorrosionOrmError> {
        let db = init_sqlite().await;
        let mut conn = db.acquire_conn().await.unwrap();
        init_users(&mut conn, USER_COUNT).await?;
        let mut paginator = User::find().add_order_by(user::COLUMN.id.asc()).paginate(2);
        for i in 0..3 {
            if let Some(page) = paginator.fetch_next(&mut conn).await? {
                assert_eq!(page[0].id, (i * 2 + 1));
                if page.len() > 1 {
                    assert_eq!(page[1].id, (i * 2 + 2));
                }
            }
        }
        Ok(())
    }
    #[tokio::test]
    async fn test_find_cursor_pagination() -> Result<(), CorrosionOrmError> {
        let db = init_sqlite().await;
        let mut conn = db.acquire_conn().await.unwrap();
        init_users(&mut conn, USER_COUNT).await?;

        let mut cursor = User::find()
            .add_order_by(user::COLUMN.id.asc())
            .cursor_paginate(2);

        let page1 = cursor
            .fetch_next(
                &mut conn,
                |u| Value::Int(u.id),
                |u| Value::Int(u.id),
                user::Column::id,
                user::Column::id,
            )
            .await?
            .unwrap();
        assert_eq!(page1.len(), 2);
        assert_eq!(page1[0].id, 1);
        assert_eq!(page1[1].id, 2);

        let page2 = cursor
            .fetch_next(
                &mut conn,
                |u| Value::Int(u.id),
                |u| Value::Int(u.id),
                user::Column::id,
                user::Column::id,
            )
            .await?
            .unwrap();
        assert_eq!(page2.len(), 2);
        assert_eq!(page2[0].id, 3);
        assert_eq!(page2[1].id, 4);

        let page3 = cursor
            .fetch_next(
                &mut conn,
                |u| Value::Int(u.id),
                |u| Value::Int(u.id),
                user::Column::id,
                user::Column::id,
            )
            .await?
            .unwrap();
        assert_eq!(page3.len(), 1);
        assert_eq!(page3[0].id, 5);

        let empty = cursor
            .fetch_next(
                &mut conn,
                |u| Value::Int(u.id),
                |u| Value::Int(u.id),
                user::Column::id,
                user::Column::id,
            )
            .await?;
        assert!(empty.is_none());

        Ok(())
    }
    #[derive(Model)]
    struct Post {
        #[PrimaryKey]
        id: i32,
        date: chrono::NaiveDate,
        date_time: chrono::NaiveDateTime,
    }
    const D: chrono::NaiveDate = chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
    const T: chrono::NaiveTime = chrono::NaiveTime::from_hms_milli_opt(12, 34, 56, 789).unwrap();

    #[tokio::test]
    async fn test_find_date() -> Result<(), CorrosionOrmError> {
        let db = init_sqlite().await;
        let mut conn = db.acquire_conn().await.unwrap();
        let mut ctx = QueryContext::from_model(Post::get_schema(), conn.get_dialect());

        conn.execute_query(&mut ctx).await?;

        let post = Post {
            id: 1,
            date: D,
            date_time: chrono::NaiveDateTime::new(D, T),
        };
        post.save(&mut conn).await?;
        let posts = Post::find()
            .filter(post::COLUMN.date.eq(D))
            .all(&mut conn)
            .await?;
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].id, 1);
        assert_eq!(posts[0].date, D);
        Ok(())
    }

    #[tokio::test]
    async fn test_find_datetime() -> Result<(), CorrosionOrmError> {
        let db = init_sqlite().await;
        let mut conn = db.acquire_conn().await.unwrap();
        let mut ctx = QueryContext::from_model(Post::get_schema(), conn.get_dialect());

        conn.execute_query(&mut ctx).await?;
        let date_time = chrono::NaiveDateTime::new(D, T);
        let post = Post {
            id: 1,
            date: D,
            date_time: chrono::NaiveDateTime::new(D, T),
        };
        post.save(&mut conn).await?;
        let posts = Post::find()
            .filter(post::COLUMN.date_time.eq(date_time))
            .all(&mut conn)
            .await?;
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].id, 1);
        assert_eq!(posts[0].date_time, date_time);
        Ok(())
    }
}
