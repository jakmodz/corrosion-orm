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
    // 1. Connect to a file-based SQLite database.
    //    sqlx does not create the file by default, so we create it first.
    //    We need 2 connections in the pool: one for regular queries,
    //    one for the transaction.
    let db_file = "./transaction_example.db";
    std::fs::File::create(db_file).expect("failed to create database file");
    let db_path = format!("sqlite:{}", db_file);
    println!("Connecting to database: {}", db_path);

    let config = corrosion_orm::SqliteConfigBuilder::new()
        .url(db_path)
        .max_connections(2) // need 2: one for conn, one for tx
        .build();
    let db = corrosion_orm::SqliteDriver::new(config).await?;
    let mut conn = db.acquire_conn().await?;

    // 2. Create the table
    let mut ctx = QueryContext::from_model(User::get_schema(), conn.get_dialect());
    conn.execute_query(&mut ctx).await?;

    // 3. Transaction: save inside, verify invisible outside
    let user = User {
        id: 1,
        name: "John Doe".into(),
    };

    let mut tx = db.transaction().await?;
    user.save(&mut tx).await?;

    // 4.  The regular connection cannot see uncommitted data
    if User::get_by_id(1, &mut conn).await?.is_none() {
        println!("User not visible outside the transaction (uncommitted)");
    }

    // 5. Commit — now the data is visible everywhere
    tx.commit().await?;

    if let Some(user) = User::get_by_id(1, &mut conn).await? {
        println!("User visible after commit: {:?}", user);
    }

    let _ = std::fs::remove_file(db_file);
    Ok(())
}
