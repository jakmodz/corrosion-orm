mod delete_test;
mod find_test;
mod get_test;
mod relation_test;
mod save_test;

#[cfg(test)]
mod tests {
    use crate::{
        init_sqlite,
        test_entities::{Post, User},
    };
    use corrosion_orm_core::CorrosionOrmError;
    use corrosion_orm_core::prelude::*;
    #[tokio::test]
    async fn test_post_save() -> Result<(), CorrosionOrmError> {
        let post = Post {
            id: 1,
            teacher_id: 1,
            user: User {
                id: 1,
                name: "Test User".to_string(),
            },
        };
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await?;
        post.save(&mut conn).await?;
        Ok(())
    }
}
