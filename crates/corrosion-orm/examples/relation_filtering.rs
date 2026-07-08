use corrosion_orm::Model;
use corrosion_orm::prelude::*;

#[derive(Model, Clone, Debug, Default)]
#[Table(name = "users")]
pub struct User {
    #[PrimaryKey]
    pub id: i64,
    pub name: String,
}

#[derive(Model, Clone)]
pub struct Post {
    #[PrimaryKey]
    pub id: i64,
    #[Column(name = "title")]
    pub title: String,
    #[BelongsTo(foreign_key = "user_id", table = "users")]
    pub author: User,
}

#[tokio::main]
async fn main() -> Result<(), CorrosionOrmError> {
    // 1. Connect to an in-memory SQLite database
    let db = corrosion_orm::connect(":memory:").await?;
    let mut conn = db.acquire_conn().await?;

    // 2. Create tables for both models
    let mut ctx = QueryContext::from_model(User::get_schema(), conn.get_dialect());
    conn.execute_query(&mut ctx).await?;
    let mut ctx = QueryContext::from_model(Post::get_schema(), conn.get_dialect());
    conn.execute_query(&mut ctx).await?;

    // 3. Insert some users
    for i in 1..=3 {
        let user = User {
            id: i,
            name: format!("Author{}", i),
        };
        user.save(&mut conn).await?;
    }

    // 4. Insert posts, each linked to a user
    for i in 1..=6 {
        let author_id = ((i - 1) % 3) + 1;
        let post = Post {
            id: i,
            title: format!("Post {}", i),
            author: User {
                id: author_id,
                name: format!("Author{}", author_id),
            },
        };
        post.save(&mut conn).await?;
    }

    // 5. Find all posts whose author's name is "Author1" (filtering across the relation)
    let posts = Post::find()
        .filter(user::COLUMN.name.eq("Author1"))
        .all(&mut conn)
        .await?;
    println!("Posts by Author1 (filter on users.name):");
    for post in &posts {
        println!(
            "  id: {}, title: {}, author_id: {}",
            post.id, post.title, post.author.id
        );
    }

    // 6. Find posts whose author id is 2
    let posts = Post::find()
        .filter(user::COLUMN.id.eq(2))
        .all(&mut conn)
        .await?;
    println!("\nPosts by author id = 2 (filter on users.id):");
    for post in &posts {
        println!("  id: {}, title: {}", post.id, post.title);
    }

    // 7. Find posts whose author name starts with "Author"
    let posts = Post::find()
        .filter(user::COLUMN.name.starts_with("Author"))
        .all(&mut conn)
        .await?;
    println!("\nAll posts (author name starts with 'Author'):");
    for post in &posts {
        println!(
            "  id: {}, title: {}, author_id: {}",
            post.id, post.title, post.author.id
        );
    }

    Ok(())
}
