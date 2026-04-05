use deluxe::ExtractAttributes;

use crate::model::{Field, IndexDefinition, primary_key::PrimaryKeyField};

#[derive(ExtractAttributes)]
#[deluxe(attributes(Table))]
pub struct TableAttribute {
    #[deluxe(default = String::from(""))]
    pub(crate) name: String,
}

#[derive(Debug)]
pub struct TableData {
    pub ident: syn::Ident,
    pub name: String,
    pub fields: Vec<Field>,
    pub primary_key: PrimaryKeyField,
    pub indexes: Vec<IndexDefinition>,
}
