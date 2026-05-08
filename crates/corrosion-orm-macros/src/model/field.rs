use deluxe::ExtractAttributes;
use syn::{Ident, Type};

use crate::utils::is_option_type;

#[derive(ExtractAttributes)]
#[deluxe(attributes(Column))]
pub struct ColumnAttribute {
    #[deluxe(default = String::from(""))]
    pub(crate) name: String,
    #[deluxe(default = false)]
    pub(crate) unique: bool,
    #[deluxe(default = false)]
    pub(crate) nullable: bool,
    #[deluxe(default = None)]
    pub(crate) column_definition: Option<String>,
    #[deluxe(default = false)]
    pub(crate) index: bool,
}

#[derive(Clone, Debug)]
pub struct Field {
    #[allow(dead_code)]
    pub iden: Ident,
    pub name: String,
    pub ty: Type,
    #[allow(dead_code)]
    pub is_unique: bool,
    pub is_nullable: bool,
    pub column_definition: Option<String>,
    pub has_index: bool,
}

impl TryFrom<(ColumnAttribute, &syn::Field)> for Field {
    type Error = syn::Error;

    /// Attempts to convert a `(ColumnAttribute, &syn::Field)` pair into a `Field`.
    ///
    /// Returns `Err(syn::Error)` if the provided `syn::Field` has no identifier (i.e., the struct uses unnamed fields).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::convert::TryFrom;
    /// // assume `attr` is a ColumnAttribute and `syn_field` is a parsed `syn::Field`
    /// let field = Field::try_from((attr, &syn_field))?;
    /// ```
    fn try_from((attr, syn_field): (ColumnAttribute, &syn::Field)) -> Result<Self, Self::Error> {
        let field_name = if attr.name.is_empty() {
            syn_field.ident.as_ref().unwrap().to_string()
        } else {
            attr.name
        };
        let iden = syn_field.ident.clone().ok_or_else(|| {
            syn::Error::new_spanned(
                syn_field,
                "Corrosion ORM models must be structs with named fields",
            )
        })?;
        let is_nullable = attr.nullable || is_option_type(&syn_field.ty);
        Ok(Field {
            iden,
            name: field_name,
            ty: syn_field.ty.clone(),
            is_unique: attr.unique,
            is_nullable,
            column_definition: attr.column_definition,
            has_index: attr.index,
        })
    }
}
