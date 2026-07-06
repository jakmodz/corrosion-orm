#[cfg(test)]
mod tests {

    use crate::{
        init_sqlite,
        test_entities::{Post, Teacher, User},
    };
    use corrosion_orm_core::{
        SqliteDriver,
        model::{lazy::Lazy, lazy_collection::LazyCollection},
        prelude::*,
    };
    use corrosion_orm_macros::Model;

    /// Helper to count rows in users table
    async fn count_users(conn: &mut impl Executor) -> Result<i64, CorrosionOrmError> {
        let users = User::get_all(conn).await?;
        Ok(users.len() as i64)
    }

    /// Helper to count rows in posts table
    async fn count_posts(conn: &mut impl Executor) -> Result<i64, CorrosionOrmError> {
        let posts = Post::get_all(conn).await?;
        Ok(posts.len() as i64)
    }

    /// Helper to count rows in teachers table
    async fn count_teachers(conn: &mut impl Executor) -> Result<i64, CorrosionOrmError> {
        let teachers = Teacher::get_all(conn).await?;
        Ok(teachers.len() as i64)
    }

    /// Creates a test user with specified ID and name
    fn create_user(id: i32, name: &str) -> User {
        User {
            id,
            name: name.to_string(),
        }
    }

    /// Creates a test post with specified ID, teacher_id, and related user
    fn create_post(id: i32, teacher_id: i64, user_id: i32, user_name: &str) -> Post {
        Post {
            id,
            teacher_id,
            user: create_user(user_id, user_name),
        }
    }

    /// Creates a test teacher with posts
    fn create_teacher_with_posts(id: i64, name: &str, posts: Vec<Post>) -> Teacher {
        Teacher {
            id,
            name: name.to_string(),
            posts,
        }
    }

    #[tokio::test]
    async fn test_belongsto_basic_save() -> Result<(), CorrosionOrmError> {
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await?;

        let post = create_post(1, 1, 1, "John Doe");
        post.save(&mut conn).await?;

        let saved_post = Post::get_by_id(1, &mut conn).await?.unwrap();
        assert_eq!(saved_post.id, 1);
        assert_eq!(saved_post.teacher_id, 1);

        let saved_user = User::get_by_id(1, &mut conn).await?.unwrap();
        assert_eq!(saved_user.name, "John Doe");

        Ok(())
    }

    #[tokio::test]
    async fn test_belongsto_fetch_with_lazy_loading() -> Result<(), CorrosionOrmError> {
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await?;

        let post = create_post(1, 1, 1, "Jane Smith");
        post.save(&mut conn).await?;
        drop(conn);

        let mut conn = driver.acquire_conn().await?;
        let fetched_post = Post::get_by_id(1, &mut conn).await?.unwrap();

        assert_eq!(fetched_post.user.id, 1);
        assert_eq!(fetched_post.user.name, "Jane Smith");

        Ok(())
    }

    #[tokio::test]
    async fn test_belongsto_get_all_with_relations() -> Result<(), CorrosionOrmError> {
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await?;

        let post1 = create_post(1, 1, 1, "User One");
        let post2 = create_post(2, 1, 2, "User Two");
        let post3 = create_post(3, 1, 3, "User Three");

        post1.save(&mut conn).await?;
        post2.save(&mut conn).await?;
        post3.save(&mut conn).await?;

        let posts = Post::get_all(&mut conn).await?;
        assert_eq!(posts.len(), 3);

        assert_eq!(posts[0].user.name, "User One");
        assert_eq!(posts[1].user.name, "User Two");
        assert_eq!(posts[2].user.name, "User Three");

        Ok(())
    }

    #[tokio::test]
    async fn test_belongsto_multiple_entities_same_relation() -> Result<(), CorrosionOrmError> {
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await?;

        let user = create_user(1, "Shared User");

        let post1 = Post {
            id: 1,
            teacher_id: 1,
            user: user.clone(),
        };

        let post2 = Post {
            id: 2,
            teacher_id: 1,
            user: user.clone(),
        };

        let post3 = Post {
            id: 3,
            teacher_id: 1,
            user: user.clone(),
        };

        post1.save(&mut conn).await?;
        post2.save(&mut conn).await?;
        post3.save(&mut conn).await?;

        let all_posts = Post::get_all(&mut conn).await?;
        assert_eq!(all_posts.len(), 3);

        assert_eq!(all_posts[0].user.id, 1);
        assert_eq!(all_posts[1].user.id, 1);
        assert_eq!(all_posts[2].user.id, 1);

        let user_count = count_users(&mut conn).await?;
        assert_eq!(user_count, 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_belongsto_update_relation() -> Result<(), CorrosionOrmError> {
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await?;

        let post = create_post(1, 1, 1, "Original User");
        post.save(&mut conn).await?;
        drop(conn);

        let mut conn = driver.acquire_conn().await?;
        let mut retrieved_post = Post::get_by_id(1, &mut conn).await?.unwrap();
        assert_eq!(retrieved_post.user.name, "Original User");

        retrieved_post.user.name = "Updated User".to_string();
        retrieved_post.save(&mut conn).await?;
        drop(conn);

        let mut conn = driver.acquire_conn().await?;
        let updated_post = Post::get_by_id(1, &mut conn).await?.unwrap();
        assert_eq!(updated_post.user.name, "Updated User");

        let updated_user = User::get_by_id(1, &mut conn).await?.unwrap();
        assert_eq!(updated_user.name, "Updated User");

        Ok(())
    }

    #[tokio::test]
    async fn test_belongsto_cascade_delete_after_parent() -> Result<(), CorrosionOrmError> {
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await?;

        let post = create_post(1, 1, 1, "To Be Deleted");
        post.save(&mut conn).await?;

        let user_count_before = count_users(&mut conn).await?;
        assert_eq!(user_count_before, 1);

        post.delete(&mut conn).await?;

        let post_result = Post::get_by_id(1, &mut conn).await?;
        assert!(post_result.is_none());

        let user_result = User::get_by_id(1, &mut conn).await?;
        assert!(user_result.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_belongsto_foreign_key_constraint() -> Result<(), CorrosionOrmError> {
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await?;

        let post = create_post(1, 5, 10, "Test User");
        post.save(&mut conn).await?;

        let saved_post = Post::get_by_id(1, &mut conn).await?.unwrap();

        assert_eq!(saved_post.teacher_id, 5);
        assert_eq!(saved_post.user.id, 10);

        Ok(())
    }

    #[tokio::test]
    async fn test_belongsto_requires_related_entity() -> Result<(), CorrosionOrmError> {
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await?;

        let post = create_post(1, 1, 1, "Valid User");
        post.save(&mut conn).await?;

        let posts = Post::get_all(&mut conn).await?;
        assert!(!posts.is_empty());
        assert_eq!(posts[0].user.id, 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_belongsto_with_default_user() -> Result<(), CorrosionOrmError> {
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await?;

        let user = User::default();
        let post = Post {
            id: 1,
            teacher_id: 1,
            user,
        };

        post.save(&mut conn).await?;
        let saved_post = Post::get_by_id(1, &mut conn).await?.unwrap();
        assert_eq!(saved_post.id, 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_belongsto_consistency_multiple_calls() -> Result<(), CorrosionOrmError> {
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await?;

        let posts = vec![
            create_post(1, 1, 1, "Post 1 User"),
            create_post(2, 1, 2, "Post 2 User"),
        ];

        for post in posts {
            post.save(&mut conn).await?;
        }

        let first_fetch = Post::get_all(&mut conn).await?;
        let second_fetch = Post::get_all(&mut conn).await?;

        assert_eq!(first_fetch.len(), second_fetch.len());
        assert_eq!(first_fetch[0].user.name, second_fetch[0].user.name);
        assert_eq!(first_fetch[1].user.name, second_fetch[1].user.name);

        Ok(())
    }

    #[tokio::test]
    async fn test_hasmany_basic_save() -> Result<(), CorrosionOrmError> {
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await?;

        let posts = vec![
            create_post(1, 1, 1, "Post One"),
            create_post(2, 1, 2, "Post Two"),
            create_post(3, 1, 3, "Post Three"),
        ];

        let teacher = create_teacher_with_posts(1, "Teacher One", posts);
        teacher.save(&mut conn).await?;

        let saved_teacher = Teacher::get_by_id(1, &mut conn).await?.unwrap();
        assert_eq!(saved_teacher.id, 1);
        assert_eq!(saved_teacher.name, "Teacher One");

        let post_count = count_posts(&mut conn).await?;
        assert_eq!(post_count, 3);

        Ok(())
    }

    #[tokio::test]
    async fn test_hasmany_lazy_loading() -> Result<(), CorrosionOrmError> {
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await?;

        let posts = vec![
            create_post(1, 1, 1, "Post One"),
            create_post(2, 1, 2, "Post Two"),
        ];

        let teacher = create_teacher_with_posts(1, "Teacher One", posts);
        teacher.save(&mut conn).await?;
        drop(conn);

        let mut conn = driver.acquire_conn().await?;
        let fetched_teacher = Teacher::get_by_id(1, &mut conn).await?.unwrap();

        assert_eq!(fetched_teacher.posts.len(), 2);
        assert_eq!(fetched_teacher.posts[0].id, 1);
        assert_eq!(fetched_teacher.posts[1].id, 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_hasmany_get_all() -> Result<(), CorrosionOrmError> {
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await?;

        let posts1 = vec![
            create_post(1, 1, 1, "Teacher1 Post1"),
            create_post(2, 1, 2, "Teacher1 Post2"),
        ];
        let teacher1 = create_teacher_with_posts(1, "Teacher One", posts1);
        teacher1.save(&mut conn).await?;

        let posts2 = vec![
            create_post(3, 2, 3, "Teacher2 Post1"),
            create_post(4, 2, 4, "Teacher2 Post2"),
            create_post(5, 2, 5, "Teacher2 Post3"),
        ];
        let teacher2 = create_teacher_with_posts(2, "Teacher Two", posts2);
        teacher2.save(&mut conn).await?;

        let all_teachers = Teacher::get_all(&mut conn).await?;
        assert_eq!(all_teachers.len(), 2);
        assert_eq!(all_teachers[0].posts.len(), 2);
        assert_eq!(all_teachers[1].posts.len(), 3);

        Ok(())
    }

    #[tokio::test]
    async fn test_hasmany_empty_collection() -> Result<(), CorrosionOrmError> {
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await?;

        let teacher = create_teacher_with_posts(1, "Teacher No Posts", Vec::new());
        teacher.save(&mut conn).await?;

        let saved_teacher = Teacher::get_by_id(1, &mut conn).await?.unwrap();
        assert_eq!(saved_teacher.posts.len(), 0);
        assert!(saved_teacher.posts.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_hasmany_multiple_parents_isolation() -> Result<(), CorrosionOrmError> {
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await?;

        // Teacher 1 with 2 posts
        let teacher1_posts = vec![create_post(1, 1, 1, "T1P1"), create_post(2, 1, 2, "T1P2")];
        let teacher1 = create_teacher_with_posts(1, "Teacher Alpha", teacher1_posts);
        teacher1.save(&mut conn).await?;

        let teacher2_posts = vec![
            create_post(3, 2, 3, "T2P1"),
            create_post(4, 2, 4, "T2P2"),
            create_post(5, 2, 5, "T2P3"),
        ];
        let teacher2 = create_teacher_with_posts(2, "Teacher Beta", teacher2_posts);
        teacher2.save(&mut conn).await?;

        let teacher3_posts = vec![create_post(6, 3, 6, "T3P1")];
        let teacher3 = create_teacher_with_posts(3, "Teacher Gamma", teacher3_posts);
        teacher3.save(&mut conn).await?;

        let t1 = Teacher::get_by_id(1, &mut conn).await?.unwrap();
        let t2 = Teacher::get_by_id(2, &mut conn).await?.unwrap();
        let t3 = Teacher::get_by_id(3, &mut conn).await?.unwrap();

        assert_eq!(t1.posts.len(), 2);
        assert_eq!(t2.posts.len(), 3);
        assert_eq!(t3.posts.len(), 1);

        for post in &t1.posts {
            assert_eq!(post.teacher_id, 1);
        }
        for post in &t2.posts {
            assert_eq!(post.teacher_id, 2);
        }
        for post in &t3.posts {
            assert_eq!(post.teacher_id, 3);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_hasmany_add_more_children() -> Result<(), CorrosionOrmError> {
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await?;

        let initial_posts = vec![
            create_post(1, 1, 1, "Initial Post 1"),
            create_post(2, 1, 2, "Initial Post 2"),
        ];
        let teacher = create_teacher_with_posts(1, "Growing Teacher", initial_posts);
        teacher.save(&mut conn).await?;

        let mut retrieved = Teacher::get_by_id(1, &mut conn).await?.unwrap();
        assert_eq!(retrieved.posts.len(), 2);

        retrieved.posts.push(create_post(3, 1, 3, "New Post 3"));
        retrieved.posts.push(create_post(4, 1, 4, "New Post 4"));
        retrieved.save(&mut conn).await?;
        drop(conn);

        let mut conn = driver.acquire_conn().await?;
        let final_teacher = Teacher::get_by_id(1, &mut conn).await?.unwrap();
        assert_eq!(final_teacher.posts.len(), 4);

        let total_posts = count_posts(&mut conn).await?;
        assert_eq!(total_posts, 4);

        Ok(())
    }

    #[tokio::test]
    async fn test_hasmany_update_child() -> Result<(), CorrosionOrmError> {
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await?;

        let posts = vec![
            create_post(1, 1, 1, "Post One"),
            create_post(2, 1, 2, "Post Two"),
        ];

        let teacher = create_teacher_with_posts(1, "Teacher", posts);
        teacher.save(&mut conn).await?;

        let mut retrieved = Teacher::get_by_id(1, &mut conn).await?.unwrap();
        retrieved.posts[0].user.name = "Updated Post One User".to_string();
        retrieved.save(&mut conn).await?;
        drop(conn);

        let mut conn = driver.acquire_conn().await?;
        let final_teacher = Teacher::get_by_id(1, &mut conn).await?.unwrap();
        assert_eq!(final_teacher.posts[0].user.name, "Updated Post One User");

        Ok(())
    }

    #[tokio::test]
    async fn test_hasmany_cascade_delete_before_parent() -> Result<(), CorrosionOrmError> {
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await?;

        let posts = vec![
            create_post(1, 1, 1, "Child 1"),
            create_post(2, 1, 2, "Child 2"),
            create_post(3, 1, 3, "Child 3"),
        ];

        let teacher = create_teacher_with_posts(1, "To Delete", posts);
        teacher.save(&mut conn).await?;

        let post_count_before = count_posts(&mut conn).await?;
        assert_eq!(post_count_before, 3);

        teacher.delete(&mut conn).await?;

        let teacher_result = Teacher::get_by_id(1, &mut conn).await?;
        assert!(teacher_result.is_none());

        let post_count_after = count_posts(&mut conn).await?;
        assert_eq!(post_count_after, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_hasmany_delete_specific_child() -> Result<(), CorrosionOrmError> {
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await?;

        let posts = vec![
            create_post(1, 1, 1, "Keep This"),
            create_post(2, 1, 2, "Delete This"),
            create_post(3, 1, 3, "Keep This Too"),
        ];

        let teacher = create_teacher_with_posts(1, "Teacher", posts);
        teacher.save(&mut conn).await?;

        let post_to_delete = create_post(2, 1, 2, "Delete This");
        post_to_delete.delete(&mut conn).await?;

        let post_result = Post::get_by_id(2, &mut conn).await?;
        assert!(post_result.is_none());

        let teacher_result = Teacher::get_by_id(1, &mut conn).await?;
        assert!(teacher_result.is_some());

        let final_posts = count_posts(&mut conn).await?;
        assert_eq!(final_posts, 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_hasmany_large_collection() -> Result<(), CorrosionOrmError> {
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await?;

        let mut posts = Vec::new();
        for i in 1..=50 {
            posts.push(create_post(i, 1, i, &format!("Post {}", i)));
        }

        let teacher = create_teacher_with_posts(1, "Busy Teacher", posts);
        teacher.save(&mut conn).await?;

        let retrieved = Teacher::get_by_id(1, &mut conn).await?.unwrap();
        assert_eq!(retrieved.posts.len(), 50);

        let post_count = count_posts(&mut conn).await?;
        assert_eq!(post_count, 50);

        Ok(())
    }

    #[tokio::test]
    async fn test_hasmany_nested_relations() -> Result<(), CorrosionOrmError> {
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await?;

        let posts = vec![
            Post {
                id: 1,
                teacher_id: 1,
                user: create_user(1, "User For Post 1"),
            },
            Post {
                id: 2,
                teacher_id: 1,
                user: create_user(2, "User For Post 2"),
            },
            Post {
                id: 3,
                teacher_id: 1,
                user: create_user(3, "User For Post 3"),
            },
        ];

        let teacher = create_teacher_with_posts(1, "Teacher With Nested Relations", posts);
        teacher.save(&mut conn).await?;

        let retrieved = Teacher::get_by_id(1, &mut conn).await?.unwrap();
        assert_eq!(retrieved.posts.len(), 3);

        for post in &retrieved.posts {
            assert!(post.user.id > 0);
            assert!(!post.user.name.is_empty());
            assert_eq!(post.teacher_id, 1);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_hasmany_filter_query_generation() -> Result<(), CorrosionOrmError> {
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await?;

        let t1_posts = vec![create_post(1, 1, 1, "T1 Post")];
        let t2_posts = vec![create_post(2, 2, 2, "T2 Post")];

        create_teacher_with_posts(1, "Teacher 1", t1_posts)
            .save(&mut conn)
            .await?;
        create_teacher_with_posts(2, "Teacher 2", t2_posts)
            .save(&mut conn)
            .await?;

        let t1 = Teacher::get_by_id(1, &mut conn).await?.unwrap();
        assert_eq!(t1.posts.len(), 1);
        assert_eq!(t1.posts[0].teacher_id, 1);

        let t2 = Teacher::get_by_id(2, &mut conn).await?.unwrap();
        assert_eq!(t2.posts.len(), 1);
        assert_eq!(t2.posts[0].teacher_id, 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_complex_nested_relations() -> Result<(), CorrosionOrmError> {
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await?;

        let posts = vec![
            Post {
                id: 1,
                teacher_id: 1,
                user: create_user(1, "Author One"),
            },
            Post {
                id: 2,
                teacher_id: 1,
                user: create_user(2, "Author Two"),
            },
        ];

        let teacher = create_teacher_with_posts(1, "Complex Teacher", posts);
        teacher.save(&mut conn).await?;

        let retrieved = Teacher::get_by_id(1, &mut conn).await?.unwrap();

        assert_eq!(retrieved.name, "Complex Teacher");
        assert_eq!(retrieved.posts.len(), 2);

        for post in &retrieved.posts {
            assert_eq!(post.teacher_id, 1);
            assert!(!post.user.name.is_empty());
            assert!(post.user.id > 0);
        }

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_transaction_commit_with_relations() -> Result<(), CorrosionOrmError> {
        let driver = init_sqlite().await;

        let mut tx = driver.transaction().await?;

        let posts = vec![
            create_post(1, 1, 1, "Tx Post 1"),
            create_post(2, 1, 2, "Tx Post 2"),
        ];
        let teacher = create_teacher_with_posts(1, "Tx Teacher", posts);
        teacher.save(&mut tx).await?;

        tx.commit().await?;

        let mut conn = driver.acquire_conn().await?;
        let retrieved = Teacher::get_by_id(1, &mut conn).await?.unwrap();
        assert_eq!(retrieved.posts.len(), 2);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_transaction_rollback_with_relations() -> Result<(), CorrosionOrmError> {
        let driver = init_sqlite().await;

        let mut tx = driver.transaction().await?;

        let posts = vec![
            create_post(1, 1, 1, "Rollback Post 1"),
            create_post(2, 1, 2, "Rollback Post 2"),
        ];
        let teacher = create_teacher_with_posts(1, "Rollback Teacher", posts);
        teacher.save(&mut tx).await?;

        tx.rollback().await?;

        let mut conn = driver.acquire_conn().await?;
        let result = Teacher::get_by_id(1, &mut conn).await?;
        assert!(result.is_none());

        let post_count = count_posts(&mut conn).await?;
        assert_eq!(post_count, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_multiple_operations_sequence() -> Result<(), CorrosionOrmError> {
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await?;

        let posts = vec![
            create_post(1, 1, 1, "Multi Op Post 1"),
            create_post(2, 1, 2, "Multi Op Post 2"),
        ];
        let teacher = create_teacher_with_posts(1, "Multi Op Teacher", posts);
        teacher.save(&mut conn).await?;

        assert_eq!(count_teachers(&mut conn).await?, 1);
        assert_eq!(count_posts(&mut conn).await?, 2);

        let mut retrieved = Teacher::get_by_id(1, &mut conn).await?.unwrap();
        retrieved.name = "Updated Multi Op Teacher".to_string();
        retrieved.save(&mut conn).await?;

        let updated = Teacher::get_by_id(1, &mut conn).await?.unwrap();
        assert_eq!(updated.name, "Updated Multi Op Teacher");

        updated.delete(&mut conn).await?;

        assert_eq!(count_teachers(&mut conn).await?, 0);
        assert_eq!(count_posts(&mut conn).await?, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_consistency_multiple_manipulations() -> Result<(), CorrosionOrmError> {
        let driver = init_sqlite().await;
        let mut conn = driver.acquire_conn().await?;

        let initial_posts = vec![create_post(1, 1, 1, "Consistency Post 1")];
        let teacher = create_teacher_with_posts(1, "Consistency Teacher", initial_posts);
        teacher.save(&mut conn).await?;

        let mut t1 = Teacher::get_by_id(1, &mut conn).await?.unwrap();
        assert_eq!(t1.posts.len(), 1);
        t1.posts.push(create_post(2, 1, 2, "Consistency Post 2"));
        t1.save(&mut conn).await?;

        let mut t2 = Teacher::get_by_id(1, &mut conn).await?.unwrap();
        assert_eq!(t2.posts.len(), 2);
        t2.posts.push(create_post(3, 1, 3, "Consistency Post 3"));
        t2.save(&mut conn).await?;

        let final_teacher = Teacher::get_by_id(1, &mut conn).await?.unwrap();
        assert_eq!(final_teacher.posts.len(), 3);
        assert_eq!(count_posts(&mut conn).await?, 3);

        Ok(())
    }
    #[derive(Model, Clone)]
    #[Table(name = "countries")]
    #[Index(name = "idx_countries_id", fields = ["id"], unique = true)]
    pub struct Country {
        #[Column(name = "id")]
        #[PrimaryKey]
        #[allow(unused)]
        pub id: i32,
        #[HasOne]
        pub capital: Lazy<Capital>,
    }

    #[derive(Model, Default, Clone)]
    pub struct Capital {
        #[Column(name = "id")]
        #[PrimaryKey]
        pub id: i32,
        pub name: String,
    }
    async fn init_schemas(schemas: Vec<TableSchemaModel>, driver: &mut SqliteDriver) {
        for schema in schemas {
            let mut ctx = QueryContext::from_model(
                schema,
                driver.acquire_conn().await.unwrap().get_dialect(),
            );
            driver
                .acquire_conn()
                .await
                .unwrap()
                .execute_query(&mut ctx)
                .await
                .unwrap();
        }
    }
    #[tokio::test]
    async fn test_lazy_fetch() -> Result<(), CorrosionOrmError> {
        let mut driver = init_sqlite().await;
        init_schemas(
            vec![Capital::get_schema(), Country::get_schema()],
            &mut driver,
        )
        .await;
        let mut conn = driver.acquire_conn().await?;
        let country = Country {
            id: 1,
            capital: Lazy::from_entity(Capital {
                id: 2,
                name: String::from("Warsaw"),
            }),
        };
        country.save(&mut conn).await?;
        let mut fetched_country = Country::get_by_id(1, &mut conn).await?.unwrap();
        assert_eq!(
            fetched_country.capital.load(&mut conn).await?.name,
            String::from("Warsaw")
        );
        Ok(())
    }
    #[derive(Model, Clone)]
    #[Table(name = "articles_lazy")]
    pub struct ArticleLazy {
        #[Column(name = "id")]
        #[PrimaryKey]
        pub id: i32,
        #[BelongsTo(foreign_key = "author_id", table = "authors_lazy")]
        pub author: Lazy<AuthorLazy>,
    }

    #[derive(Model, Default, Clone)]
    #[Table(name = "authors_lazy")]
    pub struct AuthorLazy {
        #[Column(name = "id")]
        #[PrimaryKey]
        pub id: i32,
        #[Column(name = "name")]
        pub name: String,
    }

    #[derive(Model, Clone)]
    #[Table(name = "departments_lazy")]
    pub struct DepartmentLazy {
        #[Column(name = "id")]
        #[PrimaryKey]
        pub id: i32,
        #[HasMany(foreign_key = "department_id", table = "employees_lazy")]
        pub employees: LazyCollection<EmployeeLazy, employeelazy::Column>,
    }

    #[derive(Model, Default, Clone)]
    #[Table(name = "employees_lazy")]
    pub struct EmployeeLazy {
        #[Column(name = "id")]
        #[PrimaryKey]
        pub id: i32,
        #[Column(name = "department_id")]
        pub department_id: i32,
        #[Column(name = "name")]
        pub name: String,
    }

    #[tokio::test]
    async fn test_lazy_fetch_belongs_to() -> Result<(), CorrosionOrmError> {
        let mut driver = init_sqlite().await;
        init_schemas(
            vec![AuthorLazy::get_schema(), ArticleLazy::get_schema()],
            &mut driver,
        )
        .await;

        let mut conn = driver.acquire_conn().await?;

        let article = ArticleLazy {
            id: 1,
            author: Lazy::from_entity(AuthorLazy {
                id: 10,
                name: "Alice".to_string(),
            }),
        };

        article.save(&mut conn).await?;

        let mut fetched = ArticleLazy::get_by_id(1, &mut conn).await?.unwrap();
        let author = fetched.author.load(&mut conn).await?;

        assert_eq!(author.id, 10);
        assert_eq!(author.name, "Alice");

        Ok(())
    }

    #[tokio::test]
    async fn test_lazy_fetch_has_many() -> Result<(), CorrosionOrmError> {
        let schema = DepartmentLazy::get_schema();
        let rel = schema
            .relations
            .iter()
            .find(|r| r.relation_name == "employees")
            .unwrap();
        assert!(matches!(
            rel.relation_type,
            corrosion_orm_core::schema::relation::RelationType::HasMany
        ));
        assert!(!rel.is_eager, "employees relation should be lazy");

        let mut driver = init_sqlite().await;
        init_schemas(
            vec![EmployeeLazy::get_schema(), DepartmentLazy::get_schema()],
            &mut driver,
        )
        .await;

        let mut conn = driver.acquire_conn().await?;

        let department = DepartmentLazy {
            id: 1,
            employees: LazyCollection::new(),
        };
        department.save(&mut conn).await?;

        EmployeeLazy {
            id: 1,
            department_id: 1,
            name: "Emp One".to_string(),
        }
        .save(&mut conn)
        .await?;

        EmployeeLazy {
            id: 2,
            department_id: 1,
            name: "Emp Two".to_string(),
        }
        .save(&mut conn)
        .await?;

        let mut fetched = DepartmentLazy::get_by_id(1, &mut conn).await?.unwrap();
        let employees = fetched.employees.load(&mut conn).await?;

        assert_eq!(employees.len(), 2);
        assert!(employees.iter().any(|e| e.id == 1 && e.department_id == 1));
        assert!(employees.iter().any(|e| e.id == 2 && e.department_id == 1));

        Ok(())
    }
}
