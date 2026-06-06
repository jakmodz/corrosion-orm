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

impl From<(&ColumnAttribute, PrimaryKeyAttribute, &syn::Field)> for PrimaryKeyField {
    /// Construct a PrimaryKeyField from a ColumnAttribute, PrimaryKeyAttribute, and a syn::Field.
    ///
    /// The resulting PrimaryKeyField uses the column attribute's name if present; otherwise it uses
    /// the syn field's identifier. The field's type and identifier are copied from the syn::Field,
    /// and the primary key generation strategy is taken from the PrimaryKeyAttribute (falling back
    /// to the ColumnAttribute when provided there).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use crate::model::primary_key::PrimaryKeyField;
    /// use crate::model::field::ColumnAttribute;
    /// use corrosion_orm_macros::PrimaryKeyAttribute;
    /// use syn::parse_quote;
    ///
    /// let col_attr = ColumnAttribute { name: String::new(), ..Default::default() };
    /// let pk_attr = PrimaryKeyAttribute { generation_strategy: None };
    /// let syn_field: syn::Field = parse_quote!(pub id: i32);
    /// let pk_field = PrimaryKeyField::from((&col_attr, pk_attr, &syn_field));
    /// ```
    fn from(
        (col_attr, pk_attr, syn_field): (&ColumnAttribute, PrimaryKeyAttribute, &syn::Field),
    ) -> Self {
        let field_name = if col_attr.name.is_empty() {
            syn_field.ident.as_ref().unwrap().to_string()
        } else {
            col_attr.name.clone()
        };
        let generation_strategy = pk_attr
            .generation_strategy
            .or(col_attr.generation_strategy.clone());
        PrimaryKeyField {
            iden: syn_field.ident.clone().unwrap(),
            name: field_name,
            ty: syn_field.ty.clone(),
            generation_strategy,
        }
    }
}
