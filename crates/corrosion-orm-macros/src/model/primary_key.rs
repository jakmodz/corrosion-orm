use crate::model::field::ColumnAttribute;
use corrosion_orm_core::types::generation_strategy::GenerationType;
use deluxe::ExtractAttributes;
use syn::Type;

#[derive(ExtractAttributes)]
#[deluxe(attributes(PrimaryKey))]
pub struct PrimaryKeyAttribute {
    #[deluxe(default = None)]
    pub(crate) generation_strategy: Option<GenerationType>,
}
#[derive(Debug)]
pub struct PrimaryKeyField {
    #[allow(unused)]
    pub iden: syn::Ident,
    pub name: String,
    pub ty: Type,
    pub generation_strategy: Option<GenerationType>,
}

impl From<(ColumnAttribute, PrimaryKeyAttribute, &syn::Field)> for PrimaryKeyField {
    fn from(
        (col_attr, pk_attr, syn_field): (ColumnAttribute, PrimaryKeyAttribute, &syn::Field),
    ) -> Self {
        let field_name = if col_attr.name.is_empty() {
            syn_field.ident.as_ref().unwrap().to_string()
        } else {
            col_attr.name
        };
        PrimaryKeyField {
            iden: syn_field.ident.clone().unwrap(),
            name: field_name,
            ty: syn_field.ty.clone(),
            generation_strategy: pk_attr.generation_strategy,
        }
    }
}
