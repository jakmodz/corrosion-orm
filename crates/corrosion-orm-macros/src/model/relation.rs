use corrosion_orm_core::schema::relation::RelationType;
use deluxe::ExtractAttributes;
use syn::Ident;

#[derive(ExtractAttributes)]
#[deluxe(attributes(HasOne))]
pub struct HasOneAttribute {
    #[deluxe(default = None)]
    pub table: Option<String>,
    #[deluxe(default = None)]
    pub foreign_key: Option<String>,
}

#[derive(ExtractAttributes)]
#[deluxe(attributes(HasMany))]
pub struct HasManyAttribute {
    #[deluxe(default = None)]
    pub table: Option<String>,
    #[deluxe(default = None)]
    pub foreign_key: Option<String>,
}

#[derive(ExtractAttributes)]
#[deluxe(attributes(BelongsTo))]
pub struct BelongsToAttribute {
    #[deluxe(default = None)]
    pub table: Option<String>,
    #[deluxe(default = None)]
    pub foreign_key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RelationDefinition {
    pub relation_type: RelationType,
    pub table: Option<String>,
    pub foreign_key: String,
    pub relation_name: String,
    pub ty: syn::Type,
    pub ident: Ident,
}
