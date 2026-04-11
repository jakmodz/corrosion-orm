# Corrosion ORM 🦀⚙️ (Learning Project)

>  **Disclaimer:** This is a toy project created strictly for educational purposes and personal learning. It is **not** intended for production use. If you need a robust, production-ready ORM in Rust, please check out [Diesel](https://diesel.rs/) or [SeaORM](https://www.sea-ql.org/SeaORM/).

**Corrosion ORM** is a custom-built, asynchronous Object-Relational Mapper (ORM) for Rust.

The primary goal of this project is to explore and deeply understand advanced Rust concepts, particularly:
* Writing custom **Procedural Macros** (`syn`, `quote`, `proc-macro2`).
* Managing asynchronous database connections with `tokio`.
* Designing fluent, type-safe API boundaries and Query Builders.

---

##  Features

* **Async by Default:** Built on top of `tokio` to handle asynchronous database operations.
* **Macro-Driven Models:** Uses custom derive macros (e.g., `#[derive(Model)]`) to minimize boilerplate and map Rust structs to database tables.
* **Fluent Query Builder:** A modular query builder for constructing `SELECT`, `INSERT`, `UPDATE`, and `DELETE` statements.
* **SQLite Driver:** Contains a custom driver implementation for SQLite to handle raw database interactions.
* **Connection Pooling:** Implements a basic async connection pool to manage database connections efficiently.
* **Transactions:** Support for wrapping queries in SQL transactions.

##  Project Structure

The project is structured as a Cargo workspace to keep concerns cleanly separated:

* [`corrosion-orm-core`](crates/corrosion-orm-core/): The heart of the ORM. Contains the driver traits, query builder, schema definitions, and the backend implementations.
* [`corrosion-orm-macros`](crates/corrosion-orm-macros/): The procedural macros crate. It parses Rust structs and attributes using `syn` and `deluxe` to generate SQL mapping code.
* [`corrosion-orm-test`](crates/corrosion-orm-test/): Integration tests that ensure the core logic and macro generations work correctly together.

## Example

Here is an example of what the API looks like when using the ORM:

```rust
use corrosion_orm_macros::Model;
use corrosion_orm_core::driver::sqlite_driver::SqliteDriver;
use corrosion_orm_core::driver::connection_pool::ConnectionPool;
use corrosion_orm_core::query::select::Select;

// 1. Define models using the custom derive macro
#[derive(Model, Debug)]
#[corrosion(table_name = "users")]
pub struct User {
    #[corrosion(primary_key)]
    pub id: i32,
    pub username: String,
    pub email: String,
}

#[derive(Model,Debug)]
#[Table(name = "users")]
pub struct User {
    #[Column(name = "id")]
    #[PrimaryKey]
    pub id: i32,
    pub username: String,
    pub email: String
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 2. Initialize a driver
    let config = SqliteConfigBuilder::new()
        .url(String::from(":memory:"))
        .build();
    let driver = SqliteDriver::new(config).await.unwrap();
    let mut conn = driver.acquire_conn().await.unwrap();
    // 3. Query the database using the builder
    let users = User::find()
        .filter(user::COLUMN.username.eq("Bob"))
        .all(&mut conn)
        .await?;

    for user in users {
        println!("Found user: {:?}", user);
    }

    Ok(())
}
