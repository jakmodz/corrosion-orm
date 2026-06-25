use corrosion_orm::{Model, sqlx::types::chrono};

#[derive(Model)]
#[Table(name = "__migration_registry")]
pub struct MigrationRegistry {
    #[Column(generation_strategy = {auto_increment})]
    #[PrimaryKey]
    pub id: i64,
    #[Column(name = "migration_name")]
    pub name: String,
    #[Column(name = "applied_at")]
    pub applied_at: chrono::NaiveDateTime,
}

impl MigrationRegistry {
    pub fn new(name: String, applied_at: chrono::NaiveDateTime) -> Self {
        Self {
            id: 0,
            name,
            applied_at,
        }
    }
}
