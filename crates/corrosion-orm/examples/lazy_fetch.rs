use corrosion_orm::Model;
use corrosion_orm::prelude::*;
use corrosion_orm_core::model::lazy::Lazy;
use corrosion_orm_core::model::lazy_collection::LazyCollection;

//  HasOne: Country has one Capital (lazy)
#[derive(Model, Default, Clone)]
#[Table(name = "capitals")]
pub struct Capital {
    #[PrimaryKey]
    #[Column(name = "id")]
    pub id: i32,
    pub name: String,
    pub population: i32,
}

#[derive(Model, Clone)]
#[Table(name = "countries")]
pub struct Country {
    #[PrimaryKey]
    pub id: i32,
    pub name: String,
    #[HasOne]
    pub capital: Lazy<Capital>,
}

//  BelongsTo: Article belongs to Author (lazy)
#[derive(Model, Default, Clone)]
#[Table(name = "authors")]
pub struct Author {
    #[PrimaryKey]
    pub id: i32,
    pub name: String,
    pub bio: String,
}

#[derive(Model, Clone)]
#[Table(name = "articles")]
pub struct Article {
    #[PrimaryKey]
    pub id: i32,
    pub title: String,
    #[BelongsTo(foreign_key = "author_id", table = "authors")]
    pub author: Lazy<Author>,
}

//  HasMany: Department has many Employees (lazy collection) ─
#[derive(Model, Default, Clone)]
#[Table(name = "employees")]
pub struct Employee {
    #[PrimaryKey]
    pub id: i32,
    pub name: String,
    pub role: String,
    #[Column(name = "department_id")]
    pub department_id: i32,
}

#[derive(Model, Clone)]
#[Table(name = "departments")]
pub struct Department {
    #[PrimaryKey]
    pub id: i32,
    pub name: String,
    #[HasMany(foreign_key = "department_id", table = "employees")]
    pub employees: LazyCollection<Employee, employee::Column>,
}

#[tokio::main]
async fn main() -> Result<(), CorrosionOrmError> {
    let db = corrosion_orm::connect(":memory:").await?;
    let mut conn = db.acquire_conn().await?;

    for schema in &[
        Capital::get_schema(),
        Country::get_schema(),
        Author::get_schema(),
        Article::get_schema(),
        Employee::get_schema(),
        Department::get_schema(),
    ] {
        let mut ctx = QueryContext::from_model(schema.clone(), conn.get_dialect());
        conn.execute_query(&mut ctx).await?;
    }

    //  1. HasOne: save a Country with a pending Capital
    println!(" HasOne (Lazy) ");
    let poland = Country {
        id: 1,
        name: "Poland".into(),
        capital: Lazy::from_entity(Capital {
            id: 10,
            name: "Warsaw".into(),
            population: 1_800_000,
        }),
    };
    poland.save(&mut conn).await?;
    println!("Saved country with pending capital");

    // Fetch back and lazily load the capital
    let mut fetched = Country::get_by_id(1, &mut conn).await?.unwrap();
    let capital = fetched.capital.load(&mut conn).await?;
    println!(
        "Lazy-loaded capital: {} (pop: {})",
        capital.name, capital.population
    );

    //  2. BelongsTo: save an Article referencing an Author
    println!("\n BelongsTo (Lazy) ");
    let article = Article {
        id: 1,
        title: "Rust ORMs Compared".into(),
        author: Lazy::from_entity(Author {
            id: 42,
            name: "Alice".into(),
            bio: "Rustacean since 2019".into(),
        }),
    };
    article.save(&mut conn).await?;
    println!("Saved article with pending author");

    // 3. Fetch back and lazily load the author
    let mut fetched = Article::get_by_id(1, &mut conn).await?.unwrap();
    let author = fetched.author.load(&mut conn).await?;
    println!("Lazy-loaded author: {} — {}", author.name, author.bio);

    // 4. HasMany: save a Department, then load its Employees
    println!("\n HasMany (LazyCollection) ");

    // 5. Save the department (no employees yet)
    let dept = Department {
        id: 1,
        name: "Engineering".into(),
        employees: LazyCollection::new(),
    };
    dept.save(&mut conn).await?;
    println!("Saved department 'Engineering'");

    // 6. Insert employees independently
    for (i, name, role) in [
        (1, "Bob", "Backend"),
        (2, "Carol", "Frontend"),
        (3, "Dave", "DevOps"),
    ] {
        Employee {
            id: i,
            name: name.into(),
            role: role.into(),
            department_id: 1,
        }
        .save(&mut conn)
        .await?;
    }
    println!("Inserted 3 employees into Engineering");

    // 7. Fetch the department and lazily load its employees
    let mut fetched = Department::get_by_id(1, &mut conn).await?.unwrap();
    let employees = fetched.employees.load(&mut conn).await?;
    println!("Lazy-loaded employees:");
    for emp in employees {
        println!("  {} - {} ({})", emp.id, emp.name, emp.role);
    }

    // 8. Calling load again is a no-op — already cached
    let cached = fetched.employees.load(&mut conn).await?;
    println!("Re-load returned {} employees (cached)", cached.len());

    Ok(())
}
