use corrosion_orm::Model;
use corrosion_orm::prelude::*;

#[derive(Model, Debug, Clone)]
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
    user.save(&mut conn).await?;
    // 4. Find all users via the generated Repo::find()
    let users = User::find().all(&mut conn).await?;
    println!("{:?}", users);
    // 5. Find a user via the generated Repo::find()
    let user = User::find().one(&mut conn).await?;
    println!("{:?}", user);

    Ok(())
}
