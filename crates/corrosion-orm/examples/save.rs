use corrosion_orm::Model;
use corrosion_orm::prelude::*;

#[derive(Model, Clone, Debug)]
#[Table(name = "users")]
pub struct User {
    #[PrimaryKey]
    pub id: i64,
    pub name: String,
}

#[tokio::main]
async fn main() -> Result<(), CorrosionOrmError> {
    // 1. Connect to an in-memory SQLite database
    let db = corrosion_orm::connect(":memory:").await?;
    let mut conn = db.acquire_conn().await?;

    // 2. Create the "users" table from the model's schema
    let mut ctx = QueryContext::from_model(User::get_schema(), conn.get_dialect());
    conn.execute_query(&mut ctx).await?;

    // 3. Insert a user via the generated Repo::save()
    let user = User {
        id: 1,
        name: "John".to_string(),
    };
    let saved = user.save(&mut conn).await?;
    println!("Saved user: {:?}", saved);

    // 4. Find users using the generated Finder / query builder
    let users = User::find()
        .filter(user::COLUMN.name.eq("John"))
        .all(&mut conn)
        .await?;

    for u in &users {
        println!("Found user: {:?}", u);
    }

    // 5. Fetch by id
    if let Some(user) = User::get_by_id(1, &mut conn).await? {
        println!("Got by id: {:?}", user);
    }

    Ok(())
}
