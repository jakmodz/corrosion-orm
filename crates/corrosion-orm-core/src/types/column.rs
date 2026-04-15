//! Type-safe column wrapper structs for compile-time type checking in queries.

use crate::{
    query::{
        order_by::{OrderBy, OrderDirection},
        query_type::Value,
        where_clause::{Condition, WhereClause, WhereClauseType},
    },
    types::ColumnTrait,
};
macro_rules! create_boiler_plate {
    ($name:ident) => {
        impl<C: ColumnTrait> $name<C> {
            pub fn asc(&self) -> OrderBy<C> {
                OrderBy::new(self.column.clone(), OrderDirection::Asc)
            }
            pub fn desc(&self) -> OrderBy<C> {
                OrderBy::new(self.column.clone(), OrderDirection::Desc)
            }
            /// Creates a `WhereClause` for equality comparison with a value.
            pub fn eq<V: Into<Value>>(self, val: V) -> WhereClause<C> {
                WhereClause::eq(self.column, val)
            }
            /// Creates a `WhereClause` for inequality comparison with a value.
            pub fn ne<V: Into<Value>>(self, val: V) -> WhereClause<C> {
                WhereClause::ne(self.column, val)
            }
            /// Creates a `WhereClause` for `IS NULL` comparison.
            pub fn is_null(self) -> WhereClause<C> {
                WhereClause::is_null(self.column)
            }
        }
    };
}

/// A string column wrapper with methods for string-specific operations.
pub struct StringColumn<C: ColumnTrait> {
    pub column: C,
}
create_boiler_plate!(StringColumn);
impl<C: ColumnTrait> StringColumn<C> {
    pub const fn new(column: C) -> Self {
        Self { column }
    }

    /// Creates a `WhereClause` for `LIKE` comparison with a value.
    pub fn like<V: Into<Value>>(self, val: V) -> WhereClause<C> {
        WhereClause::new(WhereClauseType::Condition(Condition::like(
            self.column,
            val,
        )))
    }
    /// Creates a `WhereClause` for `NOT LIKE` comparison with a value.
    pub fn not_like<V: Into<Value>>(self, val: V) -> WhereClause<C> {
        WhereClause::new(WhereClauseType::Condition(Condition::not_like(
            self.column,
            val,
        )))
    }
    /// Creates a `WhereClause` for `CONTAINS` comparison with a value.
    pub fn contains<V: Into<Value>>(self, val: V) -> WhereClause<C> {
        let val_str = match val.into() {
            Value::String(s) => s,
            other => format!("{:?}", other),
        };
        WhereClause::new(WhereClauseType::Condition(Condition::like(
            self.column,
            Value::String(format!("%{}%", val_str)),
        )))
    }
    /// Creates a `WhereClause` for `STARTS WITH` comparison with a value.
    pub fn starts_with<V: Into<Value>>(self, val: V) -> WhereClause<C> {
        let val_str = match val.into() {
            Value::String(s) => s,
            other => format!("{:?}", other),
        };
        WhereClause::new(WhereClauseType::Condition(Condition::like(
            self.column,
            Value::String(format!("{}%", val_str)),
        )))
    }
    /// Creates a `WhereClause` for `ENDS WITH` comparison with a value.
    pub fn ends_with<V: Into<Value>>(self, val: V) -> WhereClause<C> {
        let val_str = match val.into() {
            Value::String(s) => s,
            other => format!("{:?}", other),
        };
        WhereClause::new(WhereClauseType::Condition(Condition::like(
            self.column,
            Value::String(format!("%{}", val_str)),
        )))
    }
    /// Creates a `WhereClause` for `IN` comparison with a list of values.
    pub fn in_<V: Into<Value>>(self, vals: Vec<V>) -> WhereClause<C> {
        let values: Vec<Value> = vals.into_iter().map(|v| v.into()).collect();
        WhereClause::in_(self.column, values)
    }
}

/// A numeric column wrapper with methods for numeric-specific operations.
pub struct NumericColumn<C: ColumnTrait> {
    pub column: C,
}
create_boiler_plate!(NumericColumn);
impl<C: ColumnTrait> NumericColumn<C> {
    pub const fn new(column: C) -> Self {
        Self { column }
    }
    /// Creates a `WhereClause` for greater-than comparison with a value.
    pub fn gt<V: Into<Value>>(self, val: V) -> WhereClause<C> {
        WhereClause::gt(self.column, val)
    }
    /// Creates a `WhereClause` for greater-than-or-equal comparison with a value.
    pub fn gte<V: Into<Value>>(self, val: V) -> WhereClause<C> {
        WhereClause::new(WhereClauseType::Condition(Condition::gte(self.column, val)))
    }
    /// Creates a `WhereClause` for less-than comparison with a value.
    pub fn lt<V: Into<Value>>(self, val: V) -> WhereClause<C> {
        WhereClause::lt(self.column, val)
    }
    /// Creates a `WhereClause` for less-than-or-equal comparison with a value.
    pub fn lte<V: Into<Value>>(self, val: V) -> WhereClause<C> {
        WhereClause::new(WhereClauseType::Condition(Condition::lte(self.column, val)))
    }

    pub fn between<V: Into<Value>>(self, min: V, max: V) -> WhereClause<C> {
        WhereClause::new(WhereClauseType::Condition(Condition::between(
            self.column,
            min,
            max,
        )))
    }

    pub fn in_<V: Into<Value>>(self, vals: Vec<V>) -> WhereClause<C> {
        let values: Vec<Value> = vals.into_iter().map(|v| v.into()).collect();
        WhereClause::in_(self.column, values)
    }
}

/// A date/timestamp column wrapper with methods for date-specific operations.
pub struct DateLikeColumn<C: ColumnTrait> {
    column: C,
}
create_boiler_plate!(DateLikeColumn);
impl<C: ColumnTrait> DateLikeColumn<C> {
    pub const fn new(column: C) -> Self {
        Self { column }
    }

    pub fn gt<V: Into<Value>>(self, val: V) -> WhereClause<C> {
        WhereClause::gt(self.column, val)
    }

    pub fn gte<V: Into<Value>>(self, val: V) -> WhereClause<C> {
        WhereClause::new(WhereClauseType::Condition(Condition::gte(self.column, val)))
    }

    pub fn lt<V: Into<Value>>(self, val: V) -> WhereClause<C> {
        WhereClause::lt(self.column, val)
    }

    pub fn lte<V: Into<Value>>(self, val: V) -> WhereClause<C> {
        WhereClause::new(WhereClauseType::Condition(Condition::lte(self.column, val)))
    }

    pub fn between<V: Into<Value>>(self, min: V, max: V) -> WhereClause<C> {
        WhereClause::new(WhereClauseType::Condition(Condition::between(
            self.column,
            min,
            max,
        )))
    }
}

/// A boolean column wrapper with methods for boolean-specific operations.
pub struct BooleanColumn<C: ColumnTrait> {
    column: C,
}
create_boiler_plate!(BooleanColumn);
impl<C: ColumnTrait> BooleanColumn<C> {
    pub const fn new(column: C) -> Self {
        Self { column }
    }
    pub fn gt<V: Into<Value>>(self, val: V) -> WhereClause<C> {
        WhereClause::gt(self.column, val)
    }
    pub fn gte<V: Into<Value>>(self, val: V) -> WhereClause<C> {
        WhereClause::new(WhereClauseType::Condition(Condition::gte(self.column, val)))
    }
    pub fn lt<V: Into<Value>>(self, val: V) -> WhereClause<C> {
        WhereClause::lt(self.column, val)
    }
    pub fn lte<V: Into<Value>>(self, val: V) -> WhereClause<C> {
        WhereClause::new(WhereClauseType::Condition(Condition::lte(self.column, val)))
    }

    pub fn between<V: Into<Value>>(self, min: V, max: V) -> WhereClause<C> {
        WhereClause::new(WhereClauseType::Condition(Condition::between(
            self.column,
            min,
            max,
        )))
    }
}
