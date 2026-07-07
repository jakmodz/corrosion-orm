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

    // 4. Find users using composed filtering
    let users = User::find()
        .filter(
            WhereClause::eq(user::Column::id, 1).and(WhereClause::eq(user::Column::name, "John")),
        )
        .all(&mut conn)
        .await?;
    println!("Found users: {:?}", users);

    Ok(())
}
