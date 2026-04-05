use syn::parse::{Parse, ParseStream};
use syn::token::Comma;
use syn::{Ident, LitBool, LitStr, Token};

/// Attribute for defining indexes at the table level
///
/// # Examples
/// ```ignore
/// #[Index(name = "idx_email", fields = ["email"], unique = true)]
/// #[Index(name = "idx_created", fields = ["created_at", "user_id"])]
/// ```
#[derive(Clone)]
pub struct IndexAttribute {
    pub(crate) name: String,
    pub(crate) fields: Vec<String>,
    pub(crate) unique: bool,
}

impl Parse for IndexAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = String::new();
        let mut fields = Vec::new();
        let mut unique = false;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            if key == "name" {
                let lit: LitStr = input.parse()?;
                name = lit.value();
            } else if key == "fields" {
                let content;
                let _bracket = syn::bracketed!(content in input);

                loop {
                    let field_lit: LitStr = content.parse()?;
                    fields.push(field_lit.value());

                    if !content.peek(Comma) {
                        break;
                    }
                    content.parse::<Comma>()?;
                }
            } else if key == "unique" {
                let lit: LitBool = input.parse()?;
                unique = lit.value;
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(IndexAttribute {
            name,
            fields,
            unique,
        })
    }
}

/// Represents a parsed index definition that matches IndexModel from core
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexDefinition {
    pub name: String,
    pub fields: Vec<String>,
    pub unique: bool,
}

impl From<IndexAttribute> for IndexDefinition {
    fn from(attr: IndexAttribute) -> Self {
        IndexDefinition {
            name: attr.name,
            fields: attr.fields,
            unique: attr.unique,
        }
    }
}

impl IndexDefinition {
    /// Create a new index definition
    pub fn new(name: String, fields: Vec<String>, unique: bool) -> Self {
        IndexDefinition {
            name,
            fields,
            unique,
        }
    }

    /// Generate an index name from table name and field names
    /// Format: idx_{table}_{field1}_{field2}...
    pub fn generate_name(table_name: &str, fields: &[String]) -> String {
        if fields.is_empty() {
            return format!("idx_{}", table_name);
        }
        let fields_str = fields.join("_");
        format!("idx_{}_{}", table_name, fields_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_index_name() {
        let name = IndexDefinition::generate_name("users", &["email".to_string()]);
        assert_eq!(name, "idx_users_email");

        let name = IndexDefinition::generate_name(
            "users",
            &["created_at".to_string(), "user_id".to_string()],
        );
        assert_eq!(name, "idx_users_created_at_user_id");

        let name = IndexDefinition::generate_name("users", &[]);
        assert_eq!(name, "idx_users");
    }

    #[test]
    fn test_index_definition_creation() {
        let idx = IndexDefinition::new(
            "idx_test_email".to_string(),
            vec!["email".to_string()],
            true,
        );
        assert_eq!(idx.name, "idx_test_email");
        assert_eq!(idx.fields, vec!["email"]);
        assert!(idx.unique);
    }

    #[test]
    fn test_index_definition_from_attribute() {
        let attr = IndexAttribute {
            name: "idx_custom".to_string(),
            fields: vec!["field1".to_string(), "field2".to_string()],
            unique: false,
        };
        let idx: IndexDefinition = attr.into();
        assert_eq!(idx.name, "idx_custom");
        assert_eq!(idx.fields.len(), 2);
        assert!(!idx.unique);
    }
}
