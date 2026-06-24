#[cfg(test)]
mod tests {
    use crate::test_entities::{Product, User};
    use corrosion_orm_core::query::insert_plan::InsertPlanGenerator;
    use corrosion_orm_core::query::query_type::Value;
    #[test]
    fn test_user_insert_plan() {
        let user = User {
            id: 1,
            name: "Alice".to_string(),
        };
        let values = user.get_insert_values();
        let plan = user.generate_insert_plan(values);

        assert_eq!(plan.table, "users");
        assert_eq!(plan.columns, vec!["id", "name"]);
        assert_eq!(
            plan.values,
            vec![Value::Int(1), Value::String("Alice".to_string())]
        );
    }

    #[test]
    fn test_product_insert_plan_excludes_autoincrement() {
        let product = Product {
            id: 1, // This should be ignored in the plan because it's AutoIncrement
            name: "Laptop".to_string(),
        };
        let values = product.get_insert_values();
        let plan = product.generate_insert_plan(values);

        assert_eq!(plan.table, "products");
        assert_eq!(plan.columns, vec!["name"]);
        assert_eq!(plan.values, vec![Value::String("Laptop".to_string())]);
    }

    #[test]
    fn test_insert_plan_with_provided_values() {
        let user = User {
            id: 1,
            name: "Alice".to_string(),
        };
        let provided_values = vec![Value::Int(2), Value::String("Bob".to_string())];
        let plan = user.generate_insert_plan(provided_values.clone());

        assert_eq!(plan.table, "users");
        assert_eq!(plan.columns, vec!["id", "name"]);
        assert_eq!(plan.values, provided_values);
    }

    #[test]
    fn test_post_insert_plan_includes_relation_fk() {
        use crate::test_entities::Post;
        let user = User {
            id: 10,
            name: "Author".to_string(),
        };
        let post = Post {
            id: 1,
            teacher_id: 20,
            user,
        };
        let values = post.get_insert_values();
        let plan = post.generate_insert_plan(values);

        assert_eq!(plan.table, "Post"); // Note: Post table name seems to be "Post" by default from Post struct name
        assert_eq!(plan.columns, vec!["id", "teacher_id", "user_id"]);
        assert_eq!(
            plan.values,
            vec![Value::Int(1), Value::Int64(20), Value::Int(10)]
        );
    }
}
