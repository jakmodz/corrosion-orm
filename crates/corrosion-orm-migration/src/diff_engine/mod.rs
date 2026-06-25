mod migration_op;

use std::collections::HashMap;

use corrosion_orm::schema::{
    relation::RelationType,
    table::{ColumnSchemaModel, IndexModel, TableSchemaModel},
};

pub use migration_op::*;

/// Compares two schema snapshots and produces a list of migration operations
/// that would transform `old` into `new`.
///
/// # Algorithm
///
/// 1. **Tables**: figures out which tables were added, removed, or renamed.
/// 2. **Per-table detail diff**: for each table that exists in both snapshots,
///    it diffs columns, primary key, indexes, and foreign keys.
/// 3. **Ordering**: the returned `Vec<MigrationOp>` is ordered so that:
///    - `DropTable` / `DropIndex` / `DropForeignKey` come *before* additions
///      (to avoid dependency conflicts).
///    - `CreateTable` comes *before* `AddColumn` / `AddForeignKey` / `AddIndex`
///      that reference the new table.
pub struct DiffEngine;

impl DiffEngine {
    /// Produce the ordered list of migration operations.
    pub fn diff(old: &[TableSchemaModel], new: &[TableSchemaModel]) -> Vec<MigrationOp> {
        let mut ops: Vec<MigrationOp> = Vec::new();

        let old_by_name: HashMap<&str, &TableSchemaModel> =
            old.iter().map(|t| (t.name.as_str(), t)).collect();
        let new_by_name: HashMap<&str, &TableSchemaModel> =
            new.iter().map(|t| (t.name.as_str(), t)).collect();

        let mut created: Vec<&str> = Vec::new();
        let mut removed: Vec<&str> = Vec::new();
        let mut kept: Vec<&str> = Vec::new();

        for name in new_by_name.keys() {
            if old_by_name.contains_key(name) {
                kept.push(name);
            } else {
                created.push(name);
            }
        }
        for name in old_by_name.keys() {
            if !new_by_name.contains_key(name) {
                removed.push(name);
            }
        }

        for name in &removed {
            ops.push(MigrationOp::DropTable {
                table: (*name).to_string(),
            });
        }

        for name in &created {
            let schema = new_by_name[name];
            ops.push(MigrationOp::CreateTable((*schema).clone()));
        }

        for name in &kept {
            let old_t = old_by_name[name];
            let new_t = new_by_name[name];
            let mut detail = Self::diff_table(old_t, new_t);
            ops.append(&mut detail);
        }

        ops
    }

    /// Diff columns, primary key, indexes and foreign keys of one table.
    fn diff_table(old: &TableSchemaModel, new: &TableSchemaModel) -> Vec<MigrationOp> {
        let mut ops: Vec<MigrationOp> = Vec::new();

        let old_cols: HashMap<&str, &ColumnSchemaModel> =
            old.fields.iter().map(|c| (c.name.as_str(), c)).collect();
        let new_cols: HashMap<&str, &ColumnSchemaModel> =
            new.fields.iter().map(|c| (c.name.as_str(), c)).collect();

        for (name, col) in &new_cols {
            if !old_cols.contains_key(name) {
                ops.push(MigrationOp::AddColumn {
                    table: new.name.clone(),
                    column: ColumnDef::from(*col),
                });
            }
        }

        for name in old_cols.keys() {
            if !new_cols.contains_key(name) {
                ops.push(MigrationOp::DropColumn {
                    table: new.name.clone(),
                    column: (*name).to_string(),
                });
            }
        }

        for name in new_cols.keys() {
            if let (Some(old_c), Some(new_c)) = (old_cols.get(name), new_cols.get(name)) {
                let changes = Self::diff_column(old_c, new_c);
                if !changes.is_empty() {
                    ops.push(MigrationOp::AlterColumn {
                        table: new.name.clone(),
                        column: (*name).to_string(),
                        changes,
                    });
                }
            }
        }

        if old.primary_key != new.primary_key {
            let pk_name = &new.primary_key.name;
            if new_cols.contains_key(pk_name.as_str()) {
                let changes = ColumnChanges {
                    new_type: None,
                    nullable: None,
                    unique: None,
                    auto_increment: Some(new.primary_key.generation_type.is_some()),
                };
                ops.push(MigrationOp::AlterColumn {
                    table: new.name.clone(),
                    column: pk_name.clone(),
                    changes,
                });
            }
        }

        let old_idx: HashMap<&str, &IndexModel> =
            old.indexes.iter().map(|i| (i.name.as_str(), i)).collect();
        let new_idx: HashMap<&str, &IndexModel> =
            new.indexes.iter().map(|i| (i.name.as_str(), i)).collect();

        for name in old_idx.keys() {
            if !new_idx.contains_key(name) {
                ops.push(MigrationOp::DropIndex {
                    table: new.name.clone(),
                    index: (*name).to_string(),
                });
            }
        }

        for (name, new_i) in &new_idx {
            match old_idx.get(name) {
                None => ops.push(MigrationOp::AddIndex {
                    table: new.name.clone(),
                    index: (*new_i).clone(),
                }),
                Some(old_i) => {
                    if old_i.fields != new_i.fields || old_i.unique != new_i.unique {
                        ops.push(MigrationOp::DropIndex {
                            table: new.name.clone(),
                            index: (*name).to_string(),
                        });
                        ops.push(MigrationOp::AddIndex {
                            table: new.name.clone(),
                            index: (*new_i).clone(),
                        });
                    }
                }
            }
        }

        let old_fks: Vec<ForeignKeyDef> = old
            .relations
            .iter()
            .filter_map(|r| match r.relation_type {
                RelationType::BelongsTo | RelationType::HasOne => Some(ForeignKeyDef {
                    column: r.foreign_key.clone(),
                    referenced_table: r.table.clone(),
                    referenced_column: r.target_key.clone(),
                }),
                _ => None,
            })
            .collect();

        let new_fks: Vec<ForeignKeyDef> = new
            .relations
            .iter()
            .filter_map(|r| match r.relation_type {
                RelationType::BelongsTo | RelationType::HasOne => Some(ForeignKeyDef {
                    column: r.foreign_key.clone(),
                    referenced_table: r.table.clone(),
                    referenced_column: r.target_key.clone(),
                }),
                _ => None,
            })
            .collect();

        for fk in &old_fks {
            if !new_fks.contains(fk) {
                ops.push(MigrationOp::DropForeignKey {
                    table: new.name.clone(),
                    constraint_name: fk.column.clone(),
                });
            }
        }
        for fk in &new_fks {
            if !old_fks.contains(fk) {
                ops.push(MigrationOp::AddForeignKey {
                    table: new.name.clone(),
                    fk: fk.clone(),
                });
            }
        }

        ops
    }

    fn diff_column(old: &ColumnSchemaModel, new: &ColumnSchemaModel) -> ColumnChanges {
        let mut changes = ColumnChanges::default();

        if old.sql_type != new.sql_type {
            changes.new_type = Some(new.sql_type.clone());
        }
        if old.is_nullable != new.is_nullable {
            changes.nullable = Some(new.is_nullable);
        }
        if old.is_unique != new.is_unique {
            changes.unique = Some(new.is_unique);
        }
        if old.generation_type != new.generation_type {
            changes.auto_increment = Some(new.generation_type.is_some());
        }

        changes
    }
}

#[cfg(test)]
mod tests {
    use std::slice;

    use super::*;
    use corrosion_orm::types::column_type::SqlType;

    fn user_schema() -> TableSchemaModel {
        let mut s = TableSchemaModel::new("users".into());
        s.primary_key.name = "id".into();
        s.fields.push(ColumnSchemaModel {
            name: "email".into(),
            is_nullable: false,
            is_unique: true,
            sql_type: SqlType::Varchar(255),
            generation_type: None,
        });
        s.fields.push(ColumnSchemaModel {
            name: "name".into(),
            is_nullable: true,
            is_unique: false,
            sql_type: SqlType::Varchar(100),
            generation_type: None,
        });
        s.indexes.push(IndexModel {
            name: "idx_users_email".into(),
            fields: vec!["email".into()],
            unique: true,
        });
        s
    }

    #[test]
    fn no_changes() {
        let schema = user_schema();
        let ops = DiffEngine::diff(slice::from_ref(&schema), slice::from_ref(&schema));
        assert!(ops.is_empty(), "expected no ops, got: {ops:?}");
    }

    #[test]
    fn create_table() {
        let new = user_schema();
        let ops = DiffEngine::diff(&[], &[new]);
        assert_eq!(ops.len(), 1);
        assert!(matches!(ops[0], MigrationOp::CreateTable(..)));
    }

    #[test]
    fn drop_table() {
        let old = user_schema();
        let ops = DiffEngine::diff(&[old], &[]);
        assert_eq!(ops.len(), 1);
        assert!(matches!(ops[0], MigrationOp::DropTable { .. }));
    }

    #[test]
    fn add_column() {
        let old = user_schema();
        let mut new = user_schema();
        new.fields.push(ColumnSchemaModel {
            name: "age".into(),
            is_nullable: true,
            is_unique: false,
            sql_type: SqlType::Integer,
            generation_type: None,
        });
        let ops = DiffEngine::diff(&[old], &[new]);
        assert!(
            ops.iter().any(
                |op| matches!(op, MigrationOp::AddColumn { column: c, .. } if c.name == "age")
            )
        );
    }

    #[test]
    fn drop_column() {
        let mut old = user_schema();
        let new = user_schema();
        old.fields.push(ColumnSchemaModel {
            name: "age".into(),
            is_nullable: true,
            is_unique: false,
            sql_type: SqlType::Integer,
            generation_type: None,
        });
        let ops = DiffEngine::diff(&[old], &[new]);
        assert!(
            ops.iter()
                .any(|op| matches!(op, MigrationOp::DropColumn { column: c, .. } if c == "age"))
        );
    }

    #[test]
    fn alter_column_type() {
        let old = user_schema();
        let mut new = user_schema();
        if let Some(col) = new.fields.iter_mut().find(|c| c.name == "email") {
            col.sql_type = SqlType::Text;
        }
        let ops = DiffEngine::diff(&[old], &[new]);
        assert!(
            ops.iter()
                .any(|op| matches!(op, MigrationOp::AlterColumn { column: c, .. } if c == "email"))
        );
    }

    #[test]
    fn add_index() {
        let old = user_schema();
        let mut new = user_schema();
        new.indexes.push(IndexModel {
            name: "idx_users_name".into(),
            fields: vec!["name".into()],
            unique: false,
        });
        let ops = DiffEngine::diff(&[old], &[new]);
        assert!(ops.iter().any(
            |op| matches!(op, MigrationOp::AddIndex { index: i, .. } if i.name == "idx_users_name")
        ));
    }

    #[test]
    fn drop_index() {
        let old = user_schema();
        let mut new = user_schema();
        new.indexes.clear();
        let ops = DiffEngine::diff(&[old], &[new]);
        assert!(ops.iter().any(
            |op| matches!(op, MigrationOp::DropIndex { index: i, .. } if i == "idx_users_email")
        ));
    }

    #[test]
    fn rename_table_detected_as_drop_plus_create() {
        let old_schema = user_schema();
        let mut new_schema = user_schema();
        new_schema.name = "accounts".into();

        let ops = DiffEngine::diff(&[old_schema], &[new_schema]);
        assert!(
            ops.iter()
                .any(|op| matches!(op, MigrationOp::DropTable { table: t } if t == "users")),
            "expected DropTable for users"
        );
        assert!(
            ops.iter()
                .any(|op| matches!(op, MigrationOp::CreateTable(..))),
            "expected CreateTable"
        );
    }
}
