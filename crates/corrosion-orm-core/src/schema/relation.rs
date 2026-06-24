use serde::{Deserialize, Serialize};

use crate::schema::table::ColumnSchemaModel;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RelationType {
    HasOne,
    HasMany,
    BelongsTo,
    BelongsToMany,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelationModel {
    pub relation_type: RelationType,
    pub table: String,
    pub foreign_key: String,
    pub target_key: String,
    pub relation_name: String,
    pub field: ColumnSchemaModel,
    /// The table where the foreign key column exists
    pub source_table: String,
    /// Whether this relation should be eager-loaded by default.
    pub is_eager: bool,
}

impl RelationModel {
    /// Creates a `RelationBuilder` used to construct a `RelationModel`.
    ///
    /// Prefer the builder to avoid long argument lists and improve call-site
    /// readability.
    pub fn builder() -> RelationBuilder {
        RelationBuilder::default()
    }
}

/// Builder for `RelationModel`.
#[derive(Debug, Clone, Default)]
pub struct RelationBuilder {
    relation_type: Option<RelationType>,
    table: Option<String>,
    foreign_key: Option<String>,
    target_key: Option<String>,
    relation_name: Option<String>,
    field: Option<ColumnSchemaModel>,
    /// The table where the foreign key column exists
    source_table: Option<String>,
    /// Whether this relation should be eager-loaded by default.
    is_eager: Option<bool>,
}

impl RelationBuilder {
    pub fn relation_type(mut self, t: RelationType) -> Self {
        self.relation_type = Some(t);
        self
    }

    pub fn table<S: Into<String>>(mut self, s: S) -> Self {
        self.table = Some(s.into());
        self
    }

    pub fn foreign_key<S: Into<String>>(mut self, s: S) -> Self {
        self.foreign_key = Some(s.into());
        self
    }

    pub fn target_key<S: Into<String>>(mut self, s: S) -> Self {
        self.target_key = Some(s.into());
        self
    }

    pub fn relation_name<S: Into<String>>(mut self, s: S) -> Self {
        self.relation_name = Some(s.into());
        self
    }

    pub fn field(mut self, f: ColumnSchemaModel) -> Self {
        self.field = Some(f);
        self
    }

    pub fn source_table<S: Into<String>>(mut self, s: S) -> Self {
        self.source_table = Some(s.into());
        self
    }

    pub fn is_eager(mut self, b: bool) -> Self {
        self.is_eager = Some(b);
        self
    }

    /// Build the `RelationModel`, consuming the builder. Panics if required fields
    /// are missing.
    pub fn build(self) -> RelationModel {
        RelationModel {
            relation_type: self.relation_type.expect("relation_type is required"),
            table: self.table.expect("table is required"),
            foreign_key: self.foreign_key.expect("foreign_key is required"),
            target_key: self.target_key.expect("target_key is required"),
            relation_name: self.relation_name.expect("relation_name is required"),
            field: self.field.expect("field is required"),
            source_table: self.source_table.expect("source_table is required"),
            is_eager: self.is_eager.unwrap_or(false),
        }
    }
}
