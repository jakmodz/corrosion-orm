use corrosion_orm::dialect::sql_dialect::SqlDialect;
use corrosion_orm::schema::table::{
    ColumnSchemaModel, IndexModel, PrimaryKeyModel, TableSchemaModel,
};
use corrosion_orm::types::column_type::SqlType;

/// A single schema-level migration operation.
///
/// Each variant describes one atomic DDL change that transforms the old schema
/// into the new one.  `DiffEngine::diff` returns a `Vec<MigrationOp>` that can
/// be rendered into SQL by the chosen dialect.
#[derive(Debug, Clone, PartialEq)]
pub enum MigrationOp {
    /// Create a new table with its full schema.
    CreateTable(TableSchemaModel),
    /// Drop an existing table.
    DropTable {
        table: String,
    },
    RenameTable {
        from: String,
        to: String,
    },

    AddColumn {
        table: String,
        column: ColumnDef,
    },
    DropColumn {
        table: String,
        column: String,
    },
    AlterColumn {
        table: String,
        column: String,
        changes: ColumnChanges,
    },

    AddIndex {
        table: String,
        index: IndexModel,
    },
    DropIndex {
        table: String,
        index: String,
    },

    AddForeignKey {
        table: String,
        fk: ForeignKeyDef,
    },
    DropForeignKey {
        table: String,
        constraint_name: String,
    },
}

/// Simplified column descriptor for DDL generation.
///
/// We intentionally do **not** reuse `ColumnSchemaModel` here because that
/// type is coupled to the ORM's `GenerationType` enum, while this layer
/// operates at the raw SQL level and only needs a boolean
/// `auto_increment` flag.
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDef {
    pub name: String,
    pub sql_type: SqlType,
    pub nullable: bool,
    pub unique: bool,
    pub auto_increment: bool,
}

impl From<&ColumnSchemaModel> for ColumnDef {
    fn from(c: &ColumnSchemaModel) -> Self {
        Self {
            name: c.name.clone(),
            sql_type: c.sql_type.clone(),
            nullable: c.is_nullable,
            unique: c.is_unique,
            auto_increment: c.generation_type.is_some(),
        }
    }
}

/// Simplified primary-key descriptor for DDL generation.
///
/// Same rationale as `ColumnDef` — avoids leaking `GenerationType` into
/// the diff engine.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryKeyDef {
    pub name: String,
    pub sql_type: SqlType,
    pub auto_increment: bool,
}

impl From<&PrimaryKeyModel> for PrimaryKeyDef {
    fn from(pk: &PrimaryKeyModel) -> Self {
        Self {
            name: pk.name.clone(),
            sql_type: pk.ty.clone(),
            auto_increment: pk.generation_type.is_some(),
        }
    }
}

/// Describes a foreign-key constraint (which column points where).
#[derive(Debug, Clone, PartialEq)]
pub struct ForeignKeyDef {
    pub column: String,
    pub referenced_table: String,
    pub referenced_column: String,
}

/// Describes which properties of a column changed.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ColumnChanges {
    pub new_type: Option<SqlType>,
    pub nullable: Option<bool>,
    pub unique: Option<bool>,
    pub auto_increment: Option<bool>,
}

impl ColumnChanges {
    pub fn is_empty(&self) -> bool {
        self.new_type.is_none()
            && self.nullable.is_none()
            && self.unique.is_none()
            && self.auto_increment.is_none()
    }
}

use corrosion_orm::query::alter::{Alter, AlterAction};
use corrosion_orm::query::create::Create;
use corrosion_orm::query::drop::Drop;
use corrosion_orm::query::{ToSql, query_type::QueryContext};

/// Render a single migration operation into a complete SQL statement (with trailing `;`).
pub(crate) fn op_to_sql(op: &MigrationOp, dialect: &dyn SqlDialect) -> String {
    let mut ctx = QueryContext::new();

    match op {
        MigrationOp::CreateTable(schema) => {
            let q = Create::new(schema.clone());
            q.to_sql(&mut ctx, dialect);
        }

        MigrationOp::DropTable { table } => {
            let q = Drop::new(table);
            q.to_sql(&mut ctx, dialect);
        }

        MigrationOp::RenameTable { from: _from, to } => {
            let q = Alter::new(_from.clone(), AlterAction::RenameTable { to: to.clone() });
            q.to_sql(&mut ctx, dialect);
        }

        MigrationOp::AddColumn { table, column } => {
            let col_schema = ColumnSchemaModel {
                name: column.name.clone(),
                is_nullable: column.nullable,
                is_unique: column.unique,
                sql_type: column.sql_type.clone(),
                generation_type: if column.auto_increment {
                    Some(corrosion_orm::types::generation_strategy::GenerationType::AutoIncrement)
                } else {
                    None
                },
            };
            let q = Alter::new(table.clone(), AlterAction::AddColumn { column: col_schema });
            q.to_sql(&mut ctx, dialect);
        }

        MigrationOp::DropColumn { table, column } => {
            let q = Alter::new(
                table.clone(),
                AlterAction::DropColumn {
                    name: column.clone(),
                },
            );
            q.to_sql(&mut ctx, dialect);
        }

        MigrationOp::AlterColumn {
            table,
            column,
            changes,
        } => {
            if let Some(ref new_type) = changes.new_type {
                ctx.sql.push_str(&format!(
                    "-- SQLite does not support ALTER COLUMN type; manually recreate table {table}\n--  desired: ALTER TABLE {table} ALTER COLUMN {column} {}",
                    dialect.cast_type(new_type),
                ));
            }
            if changes.nullable.is_some() {
                ctx.sql.push_str(&format!(
                    "-- TODO: {table}.{column} nullability change (not supported by SQLite)"
                ));
            }
            if changes.unique.is_some() {
                ctx.sql.push_str(&format!(
                    "-- TODO: {table}.{column} uniqueness change (not supported by SQLite)"
                ));
            }
            if changes.auto_increment == Some(true) {
                ctx.sql.push_str(&format!(
                    "-- TODO: make {table}.{column} AUTO_INCREMENT (not supported by SQLite)"
                ));
            }
        }

        MigrationOp::AddIndex { table, index } => {
            ctx.sql
                .push_str(&dialect.generate_create_index_ddl(table, index));
            ctx.sql.push(';');
        }

        MigrationOp::DropIndex { table: _, index } => {
            ctx.sql.push_str(&format!("DROP INDEX IF EXISTS {index}"));
            ctx.sql.push(';');
        }

        MigrationOp::AddForeignKey { table, fk } => {
            ctx.sql.push_str(&format!(
                "ALTER TABLE {table} ADD FOREIGN KEY ({}) REFERENCES {} ({})",
                fk.column, fk.referenced_table, fk.referenced_column
            ));
            ctx.sql.push(';');
        }

        MigrationOp::DropForeignKey {
            table,
            constraint_name,
        } => {
            ctx.sql.push_str(&format!(
                "ALTER TABLE {table} DROP CONSTRAINT {constraint_name}"
            ));
            ctx.sql.push(';');
        }
    }

    ctx.sql
}

/// Render the operations for the `up` direction (apply changes).
pub fn render_up_sql(ops: &[MigrationOp], dialect: &dyn SqlDialect) -> String {
    let mut sql = String::new();
    for op in ops {
        sql.push_str(&op_to_sql(op, dialect));
        sql.push('\n');
    }
    sql
}

/// Render the operations for the `down` direction (revert changes) as raw SQL strings.
pub fn render_down_sql(ops: &[MigrationOp], dialect: &dyn SqlDialect) -> String {
    let mut sql = String::new();
    for op in ops.iter().rev() {
        sql.push_str(&reverse_op_to_sql(op, dialect));
        sql.push('\n');
    }
    sql
}

/// Render the SQL for the *reverse* of a migration operation (used for `down`).
pub(crate) fn reverse_op_to_sql(op: &MigrationOp, dialect: &dyn SqlDialect) -> String {
    let mut ctx = QueryContext::new();
    match op {
        MigrationOp::CreateTable(schema) => {
            let q = Drop::new(&schema.name);
            q.to_sql(&mut ctx, dialect);
        }
        MigrationOp::DropTable { table } => {
            ctx.sql.push_str(&format!(
                "-- REVERSE: cannot recreate {table}, schema unknown"
            ));
        }
        MigrationOp::RenameTable { from, to } => {
            let q = Alter::new(to.clone(), AlterAction::RenameTable { to: from.clone() });
            q.to_sql(&mut ctx, dialect);
        }
        MigrationOp::AddColumn { table, column } => {
            let q = Alter::new(
                table.clone(),
                AlterAction::DropColumn {
                    name: column.name.clone(),
                },
            );
            q.to_sql(&mut ctx, dialect);
        }
        MigrationOp::DropColumn { table, column } => {
            ctx.sql.push_str(&format!(
                "-- REVERSE: cannot recreate {table}.{column}, schema unknown"
            ));
        }
        MigrationOp::AlterColumn {
            table,
            column,
            changes,
        } => {
            // Reverse each change that we applied:
            if changes.new_type.is_some() {
                ctx.sql.push_str(&format!(
                    "-- REVERSE: ALTER COLUMN {table}.{column} (old type unknown)"
                ));
                ctx.sql.push(';');
            }
            if let Some(nullable) = changes.nullable {
                // If we set it to nullable, reverse means set not null.
                if nullable {
                    ctx.sql.push_str(&format!(
                        "ALTER TABLE {table} ALTER COLUMN {column} SET NOT NULL"
                    ));
                } else {
                    ctx.sql.push_str(&format!(
                        "ALTER TABLE {table} ALTER COLUMN {column} DROP NOT NULL"
                    ));
                }
                ctx.sql.push(';');
            }
            if let Some(unique) = changes.unique {
                if unique {
                    ctx.sql.push_str(&format!(
                        "-- REVERSE: DROP UNIQUE on {table}.{column} (constraint name unknown)"
                    ));
                } else {
                    ctx.sql
                        .push_str(&format!("ALTER TABLE {table} ADD UNIQUE ({column})"));
                }
                ctx.sql.push(';');
            }
            if changes.auto_increment.is_some() {
                ctx.sql.push_str(&format!(
                    "-- REVERSE: AUTO_INCREMENT change on {table}.{column} (old value unknown)"
                ));
                ctx.sql.push(';');
            }
        }
        MigrationOp::AddIndex { table: _, index } => {
            ctx.sql
                .push_str(&format!("DROP INDEX IF EXISTS {}", index.name));
        }
        MigrationOp::DropIndex { index, .. } => {
            ctx.sql
                .push_str(&format!("-- REVERSE: cannot recreate index {index}"));
        }
        MigrationOp::AddForeignKey { table, fk } => {
            ctx.sql.push_str(&format!(
                "ALTER TABLE {table} DROP CONSTRAINT {}",
                fk.column
            ));
        }
        MigrationOp::DropForeignKey {
            table,
            constraint_name,
        } => {
            ctx.sql.push_str(&format!(
                "-- REVERSE: cannot recreate FK {constraint_name} on {table}"
            ));
        }
    }
    if ctx.sql.is_empty() || ctx.sql.ends_with(';') {
        ctx.sql
    } else {
        ctx.sql.push(';');
        ctx.sql
    }
}

/// Generate Rust code for the `up` direction using raw SQL via `QueryContext`.
pub fn render_up_code(ops: &[MigrationOp], dialect: &dyn SqlDialect) -> String {
    let mut code = String::new();
    for op in ops {
        let sql = op_to_sql(op, dialect);
        if !sql.trim().is_empty() {
            code.push_str(&escape_sql_statement(&sql));
            code.push('\n');
        }
    }
    code
}

/// Generate Rust code for the `down` direction using reverse operation SQL.
pub fn render_down_code(ops: &[MigrationOp], dialect: &dyn SqlDialect) -> String {
    let mut code = String::new();
    for op in ops.iter().rev() {
        let sql = reverse_op_to_sql(op, dialect);
        if !sql.trim().is_empty() {
            code.push_str(&escape_sql_statement(&sql));
            code.push('\n');
        }
    }
    code
}

fn escape_sql_statement(sql: &str) -> String {
    let escaped = sql
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    format!(
        "_db.execute_query(&mut QueryContext::from(\"{}\")).await?;",
        escaped
    )
}
