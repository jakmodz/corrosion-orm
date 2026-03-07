use crate::model::{
    ColumnAttribute, Field, TableAttribute, TableData,
    primary_key::{PrimaryKeyAttribute, PrimaryKeyField},
};
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

    syn::Result::Ok(TableData {
        ident: ast.ident.clone(),
        name: table_attribute
            .name
            .is_empty()
            .then(|| ast.ident.to_string())
            .unwrap_or(table_attribute.name),
        fields,
        primary_key,
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
mod tests {
    #[test]
    fn test_parse_model() {
        use super::*;
        use syn::parse_quote;
        let input: DeriveInput = parse_quote! {
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

        let table_data = parse_model(&mut input.clone()).unwrap();
        assert_eq!(table_data.name, "users");
        assert_eq!(table_data.fields[0].name, "username");
        assert_eq!(table_data.primary_key.name, "id");
    }
    #[test]
    fn fail_multiple_primary_keys() {
        use super::*;
        use syn::parse_quote;
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
        use super::*;
        use syn::parse_quote;
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
        use super::*;
        use cargo_orm_core::types::generation_strategy::GenerationType;
        use syn::parse_quote;
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
        use super::*;
        use syn::parse_quote;
        let mut input: DeriveInput = parse_quote! {
            #[Table(name = "users")]
            struct User {
                #[Column(name = "id")]
                #[PrimaryKey]
                id: i32,
                #[Column(name = "username", column_definition = "INTEGER",unique)]
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
}
