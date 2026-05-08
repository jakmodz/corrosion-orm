use crate::{
    schema::{
        relation::{RelationModel, RelationType},
        table::{
            ColumnSchemaModel, IndexModel, PrimaryKeyModel, SchemaValidationError, TableSchemaModel,
        },
    },
    types::column_type::SqlType,
};

static PRIMARY_KEY_TYPE: &str = "PRIMARY KEY";
static TAB: &str = "    ";

/// Trait representing a SQL dialect.
/// Implement `cast_type` to support a new database engine; all DDL methods have
/// working defaults built on top of it.
pub trait SqlDialect: Send + Sync {
    /// Maps an [`SqlType`] to its database-specific type name.
    fn cast_type(&self, sql_type: &SqlType) -> String;

    /// Formats the PRIMARY KEY column definition (no trailing newline or comma).
    fn cast_primary_key(&self, primary_key: &PrimaryKeyModel) -> String {
        format!(
            "{}{} {} {}",
            TAB,
            &primary_key.name,
            self.cast_type(&primary_key.ty),
            PRIMARY_KEY_TYPE
        )
    }
    /// Builds a FOREIGN KEY clause for a relation without a trailing comma or newline.
    ///
    /// The returned string is the column-level foreign key definition using the relation's
    /// `foreign_key`, referenced `table`, and `target_key`, prefixed with a single tab of indentation.
    ///
    /// # Parameters
    ///
    /// - `relation` - The relation whose `foreign_key`, `table`, and `target_key` are used to build the clause.
    ///
    /// # Returns
    ///
    /// A `String` containing the FOREIGN KEY clause, e.g. `"    FOREIGN KEY (user_id) REFERENCES users(id)"`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Example output (construction of RelationModel and dialect omitted):
    /// let fk = dialect.cast_foreign_key(&relation);
    /// assert_eq!(fk, "    FOREIGN KEY (user_id) REFERENCES users(id)");
    /// ```
    fn cast_foreign_key(&self, relation: &RelationModel) -> String {
        format!(
            "{}FOREIGN KEY ({}) REFERENCES {}({})",
            TAB, &relation.foreign_key, &relation.table, &relation.target_key
        )
    }
    /// Generates a column definition string for a relation's field.
    ///
    /// The result is the relation field rendered as a regular column definition
    /// (including indentation, name, type and constraints) with no trailing comma
    /// or newline.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // Construct a dialect and a relation model, then render the relation field.
    /// // Adjust paths/types to your crate layout.
    /// // let dialect = MySqlDialect::new();
    /// // let relation = RelationModel { field: some_column_schema, .. };
    /// // let col_def = dialect.cast_relation_field(&relation);
    /// // println!("{}", col_def);
    /// ```
    fn cast_relation_field(&self, relation: &RelationModel) -> String {
        self.cast_column(&relation.field)
    }

    /// Builds a single non-primary-key column definition string (indented, with type, nullability, and uniqueness) without a trailing comma or newline.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::schema::table::ColumnSchemaModel;
    /// use crate::schema::types::SqlType;
    /// use crate::dialect::tests::DummyDialect; // a test dialect implementing SqlDialect
    ///
    /// let col = ColumnSchemaModel {
    ///     name: "email".into(),
    ///     sql_type: SqlType::Varchar(255),
    ///     is_nullable: false,
    ///     is_unique: true,
    /// };
    ///
    /// let dialect = DummyDialect::default();
    /// let def = dialect.cast_column(&col);
    /// assert_eq!(def, "    email VARCHAR(255) NOT NULL UNIQUE");
    /// ```
    fn cast_column(&self, column: &ColumnSchemaModel) -> String {
        let mut s = format!(
            "{}{} {}",
            TAB,
            &column.name,
            self.cast_type(&column.sql_type)
        );
        if column.is_nullable {
            s.push_str(" NULL");
        } else {
            s.push_str(" NOT NULL");
        }
        if column.is_unique {
            s.push_str(" UNIQUE");
        }
        s
    }
    /// Generates `CREATE TABLE … (…);`.
    /// Validates the schema first — returns [`SchemaValidationError`] if invalid.
    fn generate_ddl(&self, schema: &TableSchemaModel) -> Result<String, SchemaValidationError> {
        self.build_create_table_ddl(schema, false)
    }

    fn generate_ddl_if_not_exists(
        &self,
        schema: &TableSchemaModel,
    ) -> Result<String, SchemaValidationError> {
        self.build_create_table_ddl(schema, true)
    }

    /// Builds a complete `CREATE TABLE` DDL statement (including index statements) for the given table schema.
    ///
    /// Validates the provided `TableSchemaModel`, then assembles a `CREATE TABLE` statement that includes the primary key,
    /// all field columns, relation fields and corresponding foreign key constraints (for `BelongsTo` and `HasOne` relations),
    /// and appends `CREATE INDEX` statements for each index in the schema.
    ///
    /// # Returns
    ///
    /// `Ok(String)` containing the full SQL DDL: the `CREATE TABLE` statement followed by any `CREATE INDEX` statements.
    ///
    /// # Errors
    ///
    /// Returns `SchemaValidationError` if `schema.validate()` fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // Given an implementation `dialect` of `SqlDialect` and a `schema: TableSchemaModel`,
    /// // produce the DDL (this example is illustrative; types and values omitted).
    /// let ddl = dialect.build_create_table_ddl(&schema, true).unwrap();
    /// println!("{}", ddl);
    /// ```
    fn build_create_table_ddl(
        &self,
        schema: &TableSchemaModel,
        if_not_exists: bool,
    ) -> Result<String, SchemaValidationError> {
        schema.validate()?;
        let guard = if if_not_exists { "IF NOT EXISTS " } else { "" };

        let mut columns: Vec<String> = Vec::with_capacity(1 + schema.fields.len());
        columns.push(self.cast_primary_key(&schema.primary_key));
        for field in &schema.fields {
            columns.push(self.cast_column(field));
        }
        for relation in &schema.relations {
            match relation.relation_type {
                RelationType::BelongsTo | RelationType::HasOne => {
                    columns.push(self.cast_relation_field(relation));
                }
                RelationType::HasMany | RelationType::BelongsToMany => {}
            }
        }

        for relation in &schema.relations {
            match relation.relation_type {
                RelationType::BelongsTo | RelationType::HasOne => {
                    columns.push(self.cast_foreign_key(relation));
                }
                RelationType::HasMany | RelationType::BelongsToMany => {}
            }
        }

        let mut ddl = format!(
            "CREATE TABLE {}{} (\n{}\n);\n",
            guard,
            &schema.name,
            columns.join(",\n")
        );
        for index in &schema.indexes {
            ddl.push_str(&self.generate_create_index_ddl(&schema.name, index));
        }

        Ok(ddl)
    }

    /// Generates `DROP TABLE <name>;`.
    fn generate_drop_table_ddl(&self, table_name: &str) -> String {
        format!("DROP TABLE {};\n", table_name)
    }

    /// Generates `DROP TABLE IF EXISTS <name>;`.
    fn generate_drop_table_if_exists_ddl(&self, table_name: &str) -> String {
        format!("DROP TABLE IF EXISTS {};\n", table_name)
    }

    /// Builds a `CREATE INDEX` statement for a table, including `IF NOT EXISTS` and the `UNIQUE` keyword when applicable.
    ///
    /// # Examples
    ///
    /// ```
    /// // Assuming `dialect` implements `SqlDialect` and `IndexModel` is available:
    /// let index = IndexModel { name: "idx_users_on_email".into(), fields: vec!["email".into()], unique: true };
    /// let sql = dialect.generate_create_index_ddl("users", &index);
    /// assert_eq!(sql, "CREATE UNIQUE INDEX IF NOT EXISTS idx_users_on_email ON users (email);\n");
    /// ```
    fn generate_create_index_ddl(&self, table_name: &str, index: &IndexModel) -> String {
        let unique = if index.unique { "UNIQUE " } else { "" };
        let columns = index.fields.join(", ");
        format!(
            "CREATE {}INDEX IF NOT EXISTS {} ON {} ({});\n",
            unique, &index.name, table_name, columns
        )
    }

    /// Generates the full DDL for a table:
    /// `CREATE TABLE IF NOT EXISTS` + one `CREATE INDEX` per entry in `schema.indexes`.
    fn generate_full_ddl(
        &self,
        schema: &TableSchemaModel,
    ) -> Result<String, SchemaValidationError> {
        let mut ddl = self.generate_ddl_if_not_exists(schema)?;
        for index in &schema.indexes {
            ddl.push_str(&self.generate_create_index_ddl(&schema.name, index));
        }
        Ok(ddl)
    }

    fn bind_param(&self, count: &usize) -> String;
}
