use crate::{
    dialect::sql_dialect::SqlDialect,
    query::{ToSql, query_type::QueryContext},
    schema::table::ColumnSchemaModel,
};

pub struct Alter {
    pub table: String,
    #[cfg(feature = "sqlite")]
    action: AlterAction,
    #[cfg(not(feature = "sqlite"))]
    actions: Vec<AlterAction>,
}

pub enum AlterAction {
    AddColumn { column: ColumnSchemaModel },
    DropColumn { name: String },
    RenameColumn { from: String, to: String },
    ModifyType { column: ColumnSchemaModel },
    RenameTable { to: String },
}

impl Alter {
    #[cfg(feature = "sqlite")]
    pub fn new<S: Into<String>>(table: S, action: AlterAction) -> Self {
        Self {
            table: table.into(),
            action,
        }
    }

    #[cfg(not(feature = "sqlite"))]
    pub fn new<S: Into<String>>(table: S) -> Self {
        Self {
            table: table.into(),
            actions: Vec::new(),
        }
    }

    #[cfg(not(feature = "sqlite"))]
    pub fn action(mut self, action: AlterAction) -> Self {
        self.actions.push(action);
        self
    }

    #[cfg(not(feature = "sqlite"))]
    pub fn actions(mut self, actions: Vec<AlterAction>) -> Self {
        self.actions.extend(actions);
        self
    }
}

impl ToSql for AlterAction {
    fn to_sql(&self, ctx: &mut QueryContext, dialect: &dyn SqlDialect) {
        match self {
            AlterAction::AddColumn { column } => {
                ctx.sql
                    .push_str(&format!("ADD {}", dialect.cast_column(column)));
            }
            AlterAction::DropColumn { name } => {
                ctx.sql.push_str(&format!("DROP COLUMN {}", name));
            }
            AlterAction::RenameColumn { from, to } => {
                ctx.sql
                    .push_str(&format!("RENAME COLUMN {} TO {}", from, to));
            }
            AlterAction::ModifyType { column } => {
                ctx.sql
                    .push_str(&format!("ALTER COLUMN {}", dialect.cast_column(column)));
            }
            AlterAction::RenameTable { to } => {
                ctx.sql.push_str(&format!("RENAME TO {}", to));
            }
        }
    }
}

impl ToSql for Alter {
    fn to_sql(&self, ctx: &mut QueryContext, dialect: &dyn SqlDialect) {
        ctx.sql.push_str(&format!("ALTER TABLE {} ", self.table));

        #[cfg(feature = "sqlite")]
        {
            self.action.to_sql(ctx, dialect);
        }
        #[cfg(not(feature = "sqlite"))]
        {
            for (i, action) in self.actions.iter().enumerate() {
                if i > 0 {
                    ctx.sql.push_str(", ");
                }
                action.to_sql(ctx, dialect);
            }
        }
    }
}

// IR do bazy danych zeby zapisac do bazy danych
//Kocpet: Parsowanie tabeli do json i wczytawanie go z ./snapschoits zapisywanie snapchostow po czyms np po migracji
//
