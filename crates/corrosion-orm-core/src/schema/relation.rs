use crate::schema::table::ColumnSchemaModel;

#[derive(Debug, Clone)]
pub enum RelationType {
    HasOne,
    HasMany,
    BelongsTo,
    BelongsToMany,
}

#[derive(Debug, Clone)]
pub struct RelationModel {
    pub relation_type: RelationType,
    pub table: String,
    pub foreign_key: String,
    pub target_key: String,
    pub relation_name: String,
    pub field: ColumnSchemaModel,
    /// The table where the foreign key column exists
    pub source_table: String,
}

impl RelationModel {
    pub fn new(
        relation_type: RelationType,
        table: String,
        foreign_key: String,
        target_key: String,
        relation_name: String,
        source_table: String,
        field: ColumnSchemaModel,
    ) -> Self {
        Self {
            relation_type,
            table,
            foreign_key,
            target_key,
            relation_name,
            source_table,
            field,
        }
    }
}
