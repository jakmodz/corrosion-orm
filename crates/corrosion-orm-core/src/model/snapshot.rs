use crate::schema::table::{TableSchema, TableSchemaModel};
use chrono::Local;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ModelsSnapshot {
    crated_at: chrono::DateTime<Local>,
    models: Vec<TableSchemaModel>,
}

impl ModelsSnapshot {
    pub fn new(models: Vec<TableSchemaModel>) -> Self {
        Self {
            crated_at: chrono::Local::now(),
            models,
        }
    }

    pub fn created_at(&self) -> &chrono::DateTime<Local> {
        &self.crated_at
    }

    pub fn models(&self) -> &[TableSchemaModel] {
        &self.models
    }
}
pub struct ModelRegistry {
    factories: Vec<fn() -> TableSchemaModel>,
}
impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelRegistry {
    pub fn new() -> Self {
        Self {
            factories: Vec::new(),
        }
    }

    pub fn register<M: TableSchema + 'static>(mut self) -> Self {
        self.factories.push(M::get_schema);
        self
    }

    pub fn snapshot(&self) -> ModelsSnapshot {
        let models = self.factories.iter().map(|f| f()).collect();
        ModelsSnapshot::new(models)
    }
}
pub trait AppRegistry {
    fn app_registry() -> ModelRegistry;
}

#[cfg(test)]
mod tests {
    use super::{AppRegistry, ModelRegistry};
    use crate::schema::table::{TableSchema, TableSchemaModel};

    struct UsersModel;
    struct PostsModel;

    impl TableSchema for UsersModel {
        fn get_table_name() -> &'static str {
            "users"
        }

        fn get_schema() -> TableSchemaModel {
            TableSchemaModel::new("users".to_string())
        }
    }

    impl TableSchema for PostsModel {
        fn get_table_name() -> &'static str {
            "posts"
        }

        fn get_schema() -> TableSchemaModel {
            TableSchemaModel::new("posts".to_string())
        }
    }

    struct TestAppRegistry;

    impl AppRegistry for TestAppRegistry {
        fn app_registry() -> ModelRegistry {
            ModelRegistry::new()
                .register::<UsersModel>()
                .register::<PostsModel>()
        }
    }

    #[test]
    fn model_registry_snapshot_contains_registered_models() {
        let registry = ModelRegistry::new()
            .register::<UsersModel>()
            .register::<PostsModel>();

        let snapshot = registry.snapshot();
        let names: Vec<&str> = snapshot.models().iter().map(|m| m.name.as_str()).collect();

        assert_eq!(names, vec!["users", "posts"]);
    }

    #[test]
    fn app_registry_trait_builds_registry_usable_for_snapshot() {
        let snapshot = TestAppRegistry::app_registry().snapshot();

        assert_eq!(snapshot.models().len(), 2);
        assert!(snapshot.models().iter().any(|m| m.name == "users"));
        assert!(snapshot.models().iter().any(|m| m.name == "posts"));
    }
}
