use corrosion_orm::Model;
use corrosion_orm::Validate;
use corrosion_orm::prelude::*;

// 1. Standalone validation (no database)
// Derive `Validate` on any struct to get `self.validate()?`.
#[derive(Validate, Debug)]
struct SignupForm {
    #[NotNull]
    #[Size(min = 3, max = 20)]
    pub username: String,

    #[Email]
    pub email: String,

    #[Pattern(pattern = r"^\d{10}$")]
    pub phone: String,
}

// 2. Model + Validation (validate before saving)
#[derive(Model, Validate, Debug, Clone)]
#[Table(name = "users")]
pub struct User {
    #[PrimaryKey]
    pub id: i64,

    #[NotNull]
    #[Size(min = 1)]
    pub name: String,

    #[Email]
    pub email: String,
}

#[tokio::main]
async fn main() -> Result<(), CorrosionOrmError> {
    // Standalone validation
    println!(" Validating a signup form ");

    let valid_form = SignupForm {
        username: "alice42".into(),
        email: "alice@example.com".into(),
        phone: "1234567890".into(),
    };
    match valid_form.validate() {
        Ok(()) => println!("Valid form: all fields passed"),
        Err(e) => println!("Unexpected error: {e}"),
    }
    // Each validator spots a different problem:
    let bad_form = SignupForm {
        username: "x".into(),         // too short
        email: "not-an-email".into(), // invalid email
        phone: "123".into(),          // not 10 digits
    };
    match bad_form.validate() {
        Ok(()) => println!("Valid (unexpected)"),
        Err(e) => println!("Validation failed: {e}"),
    }

    // Model validation before save
    println!("\n Model + Validate ");
    let db = corrosion_orm::connect(":memory:").await?;
    let mut conn = db.acquire_conn().await?;

    // Create the table
    let mut ctx = QueryContext::from_model(User::get_schema(), conn.get_dialect());
    conn.execute_query(&mut ctx).await?;

    // Validate before saving
    let user = User {
        id: 1,
        name: "Bob".into(),
        email: "bob@example.com".into(),
    };
    user.validate()?; // would return Err if name was empty or email was invalid
    user.save(&mut conn).await?;
    println!("Validated and saved: {user:?}");

    // Invalid model: email is wrong
    let bad_user = User {
        id: 2,
        name: "Eve".into(),
        email: "bad_email".into(),
    };
    match bad_user.validate() {
        Ok(()) => println!("(unexpected)"),
        Err(e) => println!("Rejected before save: {e}"),
    }

    // Validation stops at the first error
    println!("\n Validation stops at the first error ");
    // validate() checks fields in declaration order and returns the first failure.
    let form = SignupForm {
        username: "".into(), // fails NotNull first → other fields aren't checked
        email: "bad".into(),
        phone: "12".into(),
    };
    match form.validate() {
        Err(e) => println!("First error caught: {e}"),
        Ok(()) => println!("All good (unexpected)"),
    }

    Ok(())
}
