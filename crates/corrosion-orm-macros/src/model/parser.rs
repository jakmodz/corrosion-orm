use crate::model::{
    ColumnAttribute, Field, IndexAttribute, IndexDefinition, TableAttribute, TableData,
    primary_key::{PrimaryKeyAttribute, PrimaryKeyField},
};
use std::collections::HashSet;
use syn::{DeriveInput, Fields, spanned::Spanned};

pub fn parse_model(ast: &mut DeriveInput) -> syn::Result<TableData> {
    let table_attribute: TableAttribute = deluxe::extract_attributes(ast)?;
    let (fields, primary_key) = if let syn::Data::Struct(s) = &mut ast.data {
        parse_fields(&mut s.fields)?
    } else {
        return Err(syn::Error::new(
            ast.span(),
            "Model can only be derived for structs",
        ));
    };

    let primary_key = primary_key.ok_or_else(|| {
        syn::Error::new(
            ast.span(),
            "A Model must have exactly one field marked with #[PrimaryKey]",
        )
    })?;

    let table_name = if table_attribute.name.is_empty() {
        ast.ident.to_string()
    } else {
        table_attribute.name
    };

    let table_indexes: Vec<IndexDefinition> = ast
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("Index"))
        .filter_map(|attr| {
            let index_attr: IndexAttribute = attr.parse_args::<IndexAttribute>().ok()?;
            Some(IndexDefinition::from(index_attr))
        })
        .collect();

    let mut field_indexes = Vec::new();
    for field in &fields {
        if field.has_index {
            let idx_name = IndexDefinition::generate_name(&table_name, &[field.name.clone()]);
            field_indexes.push(IndexDefinition::new(
                idx_name,
                vec![field.name.clone()],
                false,
            ));
        }
    }

    let mut all_indexes = table_indexes;
    all_indexes.extend(field_indexes);

    let indexes = all_indexes
        .into_iter()
        .map(|idx| {
            if idx.name.is_empty() {
                IndexDefinition::new(
                    IndexDefinition::generate_name(&table_name, &idx.fields),
                    idx.fields,
                    idx.unique,
                )
            } else {
                idx
            }
        })
        .collect::<Vec<_>>();

    validate_unique_index_names(&indexes)?;

    syn::Result::Ok(TableData {
        ident: ast.ident.clone(),
        name: table_name,
        fields,
        primary_key,
        indexes,
    })
}

fn parse_fields(fields: &mut Fields) -> syn::Result<(Vec<Field>, Option<PrimaryKeyField>)> {
    let mut fields_vec = Vec::new();
    let mut primary_key: Option<PrimaryKeyField> = None;

    for field in fields.iter_mut() {
        let has_pk = field.attrs.iter().any(|a| a.path().is_ident("PrimaryKey"));

        let col_attr: ColumnAttribute = deluxe::extract_attributes(field)?;
        let pk_attr: PrimaryKeyAttribute = deluxe::extract_attributes(field)?;

        if has_pk {
            if primary_key.is_some() {
                return Err(syn::Error::new(
                    field.span(),
                    "Only one field can be marked with #[PrimaryKey]",
                ));
            }
            primary_key = Some(PrimaryKeyField::from((col_attr, pk_attr, &*field)));
        } else {
            fields_vec.push(Field::from((col_attr, &*field)));
        }
    }
    Ok((fields_vec, primary_key))
}

fn validate_unique_index_names(indexes: &[IndexDefinition]) -> syn::Result<()> {
    let mut seen_names = HashSet::new();

    for idx in indexes {
        if !seen_names.insert(&idx.name) {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!(
                    "Duplicate index name: '{}'. Index names must be unique.",
                    idx.name
                ),
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_parse_model() {
        let mut input: DeriveInput = parse_quote! {
            #[Table(name = "users")]
            struct User {
                #[Column(name = "id")]
                #[PrimaryKey]
                id: i32,
                #[Column(name = "username", unique)]
                username: String,
                #[Column(name = "email", unique = false, nullable)]
                email: Option<String>,
            }
        };

        let table_data = parse_model(&mut input).unwrap();
        assert_eq!(table_data.name, "users");
        assert_eq!(table_data.fields[0].name, "username");
        assert_eq!(table_data.primary_key.name, "id");
    }

    #[test]
    fn fail_multiple_primary_keys() {
        let mut input: DeriveInput = parse_quote! {
            #[Table(name = "users")]
            struct User {
                #[Column(name = "id")]
                #[PrimaryKey]
                id: i32,
                #[Column(name = "username", unique)]
                #[PrimaryKey]
                username: String,
                #[Column(name = "email", unique = false, nullable)]
                email: Option<String>,
            }
        };
        let result = parse_model(&mut input);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Only one field can be marked with #[PrimaryKey]"
        );
    }

    #[test]
    fn fail_missing_primary_key() {
        let mut input: DeriveInput = parse_quote! {
            #[Table(name = "users")]
            struct User {
                #[Column(name = "id")]
                id: i32,
                #[Column(name = "username", unique)]
                username: String,
                #[Column(name = "email", unique = false, nullable)]
                email: Option<String>,
            }
        };
        let result = parse_model(&mut input);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "A Model must have exactly one field marked with #[PrimaryKey]"
        );
    }

    #[test]
    fn primary_key_with_generation_strategy() {
        use corrosion_orm_core::types::generation_strategy::GenerationType;
        let mut input: DeriveInput = parse_quote! {
            #[Table(name = "users")]
            struct User {
                #[Column(name = "id")]
                #[PrimaryKey(generation_strategy = {auto_increment})]
                id: i32,
                #[Column(name = "username", unique)]
                username: String,
                #[Column(name = "email", unique = false, nullable)]
                email: Option<String>,
            }
        };
        let table_data = parse_model(&mut input).unwrap();
        assert_eq!(
            table_data.primary_key.generation_strategy.unwrap(),
            GenerationType::AutoIncrement
        );
    }

    #[test]
    fn test_column_definition() {
        let mut input: DeriveInput = parse_quote! {
            #[Table(name = "users")]
            struct User {
                #[Column(name = "id")]
                #[PrimaryKey]
                id: i32,
                #[Column(name = "username", column_definition = "INTEGER", unique)]
                username: String,
                #[Column(name = "email", unique = false, nullable)]
                email: Option<String>,
            }
        };
        let table_data = parse_model(&mut input).unwrap();
        assert_eq!(
            table_data.fields[0].column_definition,
            Some(String::from("INTEGER"))
        );
    }

    #[test]
    fn test_field_level_index() {
        let mut input: DeriveInput = parse_quote! {
            #[Table(name = "users")]
            struct User {
                #[Column(name = "id")]
                #[PrimaryKey]
                id: i32,
                #[Column(name = "email", index)]
                email: String,
            }
        };
        let table_data = parse_model(&mut input).unwrap();
        assert_eq!(table_data.indexes.len(), 1);
        assert_eq!(table_data.indexes[0].fields, vec!["email"]);
        assert_eq!(table_data.indexes[0].name, "idx_users_email");
    }

    #[test]
    fn test_table_level_index() {
        let mut input: DeriveInput = parse_quote! {
            #[Table(name = "users")]
            #[Index(name = "idx_email", fields = ["email"], unique = true)]
            struct User {
                #[Column(name = "id")]
                #[PrimaryKey]
                id: i32,
                #[Column(name = "email")]
                email: String,
            }
        };
        let table_data = parse_model(&mut input).unwrap();
        assert_eq!(table_data.indexes.len(), 1);
        assert_eq!(table_data.indexes[0].name, "idx_email");
        assert!(table_data.indexes[0].unique);
    }

    #[test]
    fn test_auto_generate_index_name() {
        let mut input: DeriveInput = parse_quote! {
            #[Table(name = "users")]
            #[Index(fields = ["email", "created_at"])]
            struct User {
                #[Column(name = "id")]
                #[PrimaryKey]
                id: i32,
                #[Column(name = "email")]
                email: String,
                #[Column(name = "created_at")]
                created_at: String,
            }
        };
        let table_data = parse_model(&mut input).unwrap();
        assert_eq!(table_data.indexes.len(), 1);
        assert_eq!(table_data.indexes[0].name, "idx_users_email_created_at");
    }

    #[test]
    fn test_duplicate_index_names() {
        let mut input: DeriveInput = parse_quote! {
            #[Table(name = "users")]
            #[Index(name = "idx_email", fields = ["email"])]
            #[Index(name = "idx_email", fields = ["username"])]
            struct User {
                #[Column(name = "id")]
                #[PrimaryKey]
                id: i32,
                #[Column(name = "email")]
                email: String,
            }
        };
        let result = parse_model(&mut input);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Duplicate index name")
        );
    }

    #[test]
    fn test_mixed_field_and_table_indexes() {
        let mut input: DeriveInput = parse_quote! {
            #[Table(name = "users")]
            #[Index(name = "idx_composite", fields = ["email", "username"], unique = true)]
            struct User {
                #[Column(name = "id")]
                #[PrimaryKey]
                id: i32,
                #[Column(name = "email", index)]
                email: String,
                #[Column(name = "username")]
                username: String,
            }
        };
        let table_data = parse_model(&mut input).unwrap();
        assert_eq!(table_data.indexes.len(), 2);
        let has_composite = table_data
            .indexes
            .iter()
            .any(|idx| idx.name == "idx_composite");
        let has_email = table_data
            .indexes
            .iter()
            .any(|idx| idx.name == "idx_users_email");
        assert!(has_composite);
        assert!(has_email);
    }
}
