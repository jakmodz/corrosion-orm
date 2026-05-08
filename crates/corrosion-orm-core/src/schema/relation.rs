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
    /// Creates a RelationModel from the provided relation metadata.
    ///
    /// The returned model stores the relation kind, related table and key names,
    /// relation name, the source table, and the associated column schema.
    ///
    /// # Examples
    ///
    /// ```
    /// use corrosion_orm_core::schema::{relation::RelationModel, relation::RelationType, table::ColumnSchemaModel};
    /// use corrosion_orm_core::types::column_type::SqlType;
    ///
    /// let field = ColumnSchemaModel {
    ///     name: "user_id".to_string(),
    ///     is_nullable: false,
    ///     is_unique: false,
    ///     sql_type: SqlType::Integer,
    /// };
    /// let rel = RelationModel::new(
    ///     RelationType::HasOne,
    ///     "profiles".to_string(),
    ///     "user_id".to_string(),
    ///     "id".to_string(),
    ///     "profile".to_string(),
    ///     "users".to_string(),
    ///     field,
    /// );
    /// ```
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
