use crate::{
    CorrosionOrmError, Executor,
    driver::{error::DriverError, from_row_db::FromRowDb},
    model::repository::Repo,
    query::{WhereClause, query_type::Value},
    schema::table::TableSchema,
    types::ColumnTrait,
};

#[derive(Clone)]
pub enum LazyCollectionCondition<C: ColumnTrait> {
    ByForeignKey { fk_column: C, value: Value },
    ByFilter(WhereClause),
}

#[derive(Clone)]
enum LazyCollectionStep<F, C: ColumnTrait> {
    NotLoaded(Option<LazyCollectionCondition<C>>),
    Pending(Vec<F>),
    Loaded(Vec<F>),
}

/// A lazy-loaded collection for `HasMany` relations.
///
/// `LazyCollection<F, C>` stores three possible states:
/// - `NotLoaded(Some(condition))`: not fetched yet, but has a load condition
/// - `Pending(Vec<F>)`: in-memory value set via `LazyCollection::from_vec`
/// - `Loaded(Vec<F>)`: fetched and cached
///
/// When an entity is read from the database, the `#[derive(Model)]`-generated
/// `FromRow` implementation initializes `LazyCollection::new()` and sets a
/// `LazyCollectionCondition::ByForeignKey` using the owner's primary key.
/// Calling `load` fetches the collection once and caches it; calling `load`
/// while `Pending` simply promotes the value to `Loaded` without hitting the DB.
///
/// Note: the generated `save()` does **not** cascade into lazy collections.
/// If you need cascade saves, use an eager `Vec<Child>` relation or save
/// children manually.
///
/// # Required derives
/// - **Owner struct**: `#[derive(Model)]` with `#[HasMany(...)]` on a
///   `LazyCollection<Child, child::Column>` field.
/// - **Child struct (`F`)**: `#[derive(Model)]` so `Repo`, `FromRow`,
///   `TableSchema`, and the `child::Column` enum are available.
///
/// # Example
/// ``` rust,ignore
///     #[derive(Model)]
///     #[Table(name = "departments_lazy")]
///     pub struct DepartmentLazy {
///         #[Column(name = "id")]
///         #[PrimaryKey]
///         pub id: i32,
///         #[HasMany(foreign_key = "department_id", table = "employees_lazy")]
///         pub employees: LazyCollection<EmployeeLazy, employeelazy::Column>,
///     }
///
///     #[derive(Model)]
///     #[Table(name = "employees_lazy")]
///     pub struct EmployeeLazy {
///         #[Column(name = "id")]
///         #[PrimaryKey]
///         pub id: i32,
///         #[Column(name = "department_id")]
///         pub department_id: i32,
///         pub name: String,
///     }
///
///     let mut dept = DepartmentLazy::get_by_id(1, &mut db).await?.unwrap();
///     let employees = dept.employees.load(&mut db).await?;
///     assert_eq!(employees.len(), 2);
/// ```
#[derive(Clone)]
pub struct LazyCollection<F, C: ColumnTrait> {
    step: LazyCollectionStep<F, C>,
}
impl<F, C: ColumnTrait> Default for LazyCollection<F, C> {
    fn default() -> Self {
        Self::new()
    }
}
impl<F, C: ColumnTrait> LazyCollection<F, C> {
    pub fn new() -> Self {
        Self {
            step: LazyCollectionStep::NotLoaded(None),
        }
    }

    pub fn with_condition(condition: LazyCollectionCondition<C>) -> Self {
        Self {
            step: LazyCollectionStep::NotLoaded(Some(condition)),
        }
    }

    pub fn from_vec(vec: Vec<F>) -> Self {
        Self {
            step: LazyCollectionStep::Pending(vec),
        }
    }

    pub fn set_condition(&mut self, condition: LazyCollectionCondition<C>) {
        if let LazyCollectionStep::NotLoaded(ref mut cond) = self.step {
            *cond = Some(condition);
        }
    }

    pub async fn load<E: Executor>(&mut self, db: &mut E) -> Result<&mut Vec<F>, CorrosionOrmError>
    where
        F: Repo<E, Column = C>
            + Send
            + Unpin
            + TableSchema
            + FromRowDb
            + crate::model::CacheModel
            + Clone,
    {
        if let LazyCollectionStep::Loaded(ref mut items) = self.step {
            return Ok(items);
        }

        if let LazyCollectionStep::Pending(_) = self.step
            && let LazyCollectionStep::Pending(items) =
                std::mem::replace(&mut self.step, LazyCollectionStep::NotLoaded(None))
        {
            self.step = LazyCollectionStep::Loaded(items);
            if let LazyCollectionStep::Loaded(ref mut loaded) = self.step {
                return Ok(loaded);
            }
        }

        let loaded_items = match &self.step {
            LazyCollectionStep::NotLoaded(Some(LazyCollectionCondition::ByForeignKey {
                fk_column,
                value,
            })) => {
                F::find()
                    .filter(WhereClause::eq(*fk_column, value.clone()))
                    .all(db)
                    .await?
            }
            LazyCollectionStep::NotLoaded(Some(LazyCollectionCondition::ByFilter(filter))) => {
                F::find().filter(filter.clone()).all(db).await?
            }
            LazyCollectionStep::NotLoaded(None) => {
                return Err(CorrosionOrmError::DriverError(DriverError::NotFound));
            }
            LazyCollectionStep::Pending(_) | LazyCollectionStep::Loaded(_) => unreachable!(),
        };

        self.step = LazyCollectionStep::Loaded(loaded_items);

        if let LazyCollectionStep::Loaded(ref mut items) = self.step {
            Ok(items)
        } else {
            unreachable!()
        }
    }
}
