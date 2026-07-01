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

    // 3. Insert some sample data
    for i in 1..=100 {
        let user = User {
            id: i as i64,
            name: format!("User{}", i),
        };
        user.save(&mut conn).await?;
    }

    // 4. Cursor-based pagination: fetch 10 users per page
    let mut paginator = User::find()
        .add_order_by(user::COLUMN.id.asc())
        .cursor_paginate(10);

    let mut page = 1;
    while let Some(users) = paginator
        .fetch_next(
            &mut conn,
            |u| u.id.into(),
            |u| u.id.into(),
            user::COLUMN.id.column,
            user::COLUMN.id.column,
        )
        .await?
    {
        println!("--- Page {} ---", page);
        for user in &users {
            println!("  id: {}, name: {}", user.id, user.name);
        }
        page += 1;
    }

    Ok(())
}
