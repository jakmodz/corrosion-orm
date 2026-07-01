use corrosion_orm::Model;
use corrosion_orm::prelude::*;

// HasOne: Country has one Capital (eager)
#[derive(Model, Default, Clone)]
#[Table(name = "capitals")]
pub struct Capital {
    #[PrimaryKey]
    #[Column(name = "id")]
    pub id: i32,
    pub name: String,
    pub population: i32,
}

#[derive(Model)]
#[Table(name = "countries")]
pub struct Country {
    #[PrimaryKey]
    pub id: i32,
    pub name: String,
    #[HasOne]
    pub capital: Capital,
}

//  BelongsTo: Article belongs to Author (eager)

#[derive(Model, Default, Clone)]
#[Table(name = "authors")]
pub struct Author {
    #[PrimaryKey]
    pub id: i32,
    pub name: String,
    pub bio: String,
}

#[derive(Model)]
#[Table(name = "articles")]
pub struct Article {
    #[PrimaryKey]
    pub id: i32,
    pub title: String,
    #[BelongsTo(foreign_key = "author_id", table = "authors")]
    pub author: Author,
}

//  HasMany: Department has many Employees (eager)

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

#[derive(Model)]
#[Table(name = "departments")]
pub struct Department {
    #[PrimaryKey]
    pub id: i32,
    pub name: String,
    #[HasMany(foreign_key = "department_id", table = "employees")]
    pub employees: Vec<Employee>, // eager: Vec<T>, all children loaded automatically
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

    //  1. HasOne: save a Country with its Capital embedded directly
    println!(" HasOne (Eager) ");
    let poland = Country {
        id: 1,
        name: "Poland".into(),
        capital: Capital {
            id: 10,
            name: "Warsaw".into(),
            population: 1_800_000,
        },
    };
    poland.save(&mut conn).await?;
    println!("Saved country with its capital (cascade save)");

    // Fetch back — capital is already loaded, no .load() needed
    let fetched = Country::get_by_id(1, &mut conn).await?.unwrap();
    println!(
        "Eager-loaded capital: {} (pop: {})",
        fetched.capital.name, fetched.capital.population
    );

    //  2. BelongsTo: save an Article with its Author embedded directly
    println!("\n BelongsTo (Eager) ");
    let article = Article {
        id: 1,
        title: "Rust ORMs Compared".into(),
        author: Author {
            id: 42,
            name: "Alice".into(),
            bio: "Rustacean since 2019".into(),
        },
    };
    article.save(&mut conn).await?;
    println!("Saved article with its author (cascade save)");

    // Fetch back — author is already loaded, no .load() needed
    let fetched = Article::get_by_id(1, &mut conn).await?.unwrap();
    println!(
        "Eager-loaded author: {} — {}",
        fetched.author.name, fetched.author.bio
    );

    //  3. HasMany: save a Department with its Employees embedded directly
    println!("\n HasMany (Eager) ");

    // Save the department together with all employees (cascade save)
    let dept = Department {
        id: 1,
        name: "Engineering".into(),
        employees: vec![
            Employee {
                id: 1,
                name: "Bob".into(),
                role: "Backend".into(),
                department_id: 1,
            },
            Employee {
                id: 2,
                name: "Carol".into(),
                role: "Frontend".into(),
                department_id: 1,
            },
            Employee {
                id: 3,
                name: "Dave".into(),
                role: "DevOps".into(),
                department_id: 1,
            },
        ],
    };
    dept.save(&mut conn).await?;
    println!("Saved department 'Engineering' with 3 employees (cascade save)");

    // Fetch back — employees are already loaded, no .load() needed
    let fetched = Department::get_by_id(1, &mut conn).await?.unwrap();
    println!("Eager-loaded employees:");
    for emp in &fetched.employees {
        println!("  {} - {} ({})", emp.id, emp.name, emp.role);
    }

    // Unlike lazy loading, there is no caching concern — data is always present.
    println!(
        "Department '{}' has {} employees (always loaded)",
        fetched.name,
        fetched.employees.len()
    );

    Ok(())
}
