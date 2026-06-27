use crate::driver::error::DriverError;
use crate::driver::from_row_db::FromRowDb;
use crate::model::repository::Repo;

use crate::{CorrosionOrmError, Executor, query::query_type::Value};

pub enum LazyCondition {
    ById(Value),
}

pub(crate) enum LazyStep<F> {
    NotLoaded(Option<LazyCondition>),
    Pending(F),
    Loaded(F),
}
/// A lazy-loaded single relation (`HasOne` / `BelongsTo`).
///
/// `Lazy<F>` stores the related entity in three possible states:
/// - `NotLoaded(Some(condition))`: not fetched yet, but has a load condition
/// - `Pending(entity)`: in-memory value set via `Lazy::from_entity`
/// - `Loaded(entity)`: fetched and cached
///
/// When an entity is read from the database, the `#[derive(Model)]`-generated
/// `FromRow` implementation initializes `Lazy::new()` and sets
/// `LazyCondition::ById` from the foreign-key column. Calling `load` fetches
/// the relation once and caches it; calling `load` while `Pending` simply
/// promotes the value to `Loaded` without a database round-trip.
///
/// Saving the owner uses `resolve_relation_id_value`:
/// - `Pending(entity)` => save entity and use its primary key
/// - `NotLoaded(ById(v))` => use `v` directly
/// - `Loaded(entity)` => use the entity primary key
///
/// # Required derives
/// - **Owner struct**: `#[derive(Model)]` with `#[HasOne]` or `#[BelongsTo]`
///   on a `Lazy<Related>` field.
/// - **Related struct (`F`)**: `#[derive(Model, Default, Clone)]`.
///   `Model` provides `Repo`, `FromRow`, `TableSchema`, and `get_id`/`set_id`.
///   `Default` is used by the generated `FromRow` to seed the relation and
///   assign the foreign key. `Clone` is required by `save()` to cascade
///   via `resolve_relation_id_value`.
///
/// # Examples
/// Has-one:
/// ``` rust,ignore
///     #[derive(Model)]
///     #[Table(name = "countries")]
///     pub struct Country {
///         #[Column(name = "id")]
///         #[PrimaryKey]
///         pub id: i32,
///         #[HasOne]
///         pub capital: Lazy<Capital>,
///     }
///
///     #[derive(Model, Default, Clone)]
///     #[Table(name = "capitals")]
///     pub struct Capital {
///         #[Column(name = "id")]
///         #[PrimaryKey]
///         pub id: i32,
///         pub name: String,
///     }
///
///     let country = Country {
///         id: 1,
///         capital: Lazy::from_entity(Capital { id: 2, name: "Warsaw".to_string() }),
///     };
///     country.save(&mut db).await?;
///
///     let mut fetched = Country::get_by_id(1, &mut db).await?.unwrap();
///     let capital = fetched.capital.load(&mut db).await?;
///     assert_eq!(capital.name, "Warsaw");
///     ```
/// Belongs-to:
/// ``` rust,ignore
///     #[derive(Model)]
///     #[Table(name = "articles_lazy")]
///     pub struct ArticleLazy {
///         #[Column(name = "id")]
///         #[PrimaryKey]
///         pub id: i32,
///         #[BelongsTo(foreign_key = "author_id", table = "authors_lazy")]
///         pub author: Lazy<AuthorLazy>,
///     }
///
///     #[derive(Model, Default, Clone)]
///     #[Table(name = "authors_lazy")]
///     pub struct AuthorLazy {
///         #[Column(name = "id")]
///         #[PrimaryKey]
///         pub id: i32,
///         pub name: String,
///     }
///
///     let mut article = ArticleLazy::get_by_id(1, &mut db).await?.unwrap();
///     let author = article.author.load(&mut db).await?;
///     assert_eq!(author.name, "Alice");
/// ```
pub struct Lazy<F> {
    step: LazyStep<F>,
}

impl<F> Default for Lazy<F> {
    fn default() -> Self {
        Self::new()
    }
}

impl<F> Lazy<F> {
    pub fn new() -> Self {
        Self {
            step: LazyStep::NotLoaded(None),
        }
    }
    pub fn from_entity(entity: F) -> Self {
        Self {
            step: LazyStep::Pending(entity),
        }
    }
    /// Sets the condition required to load this relation later
    pub fn set_condition(&mut self, condition: LazyCondition) {
        if let LazyStep::NotLoaded(ref mut cond) = self.step {
            *cond = Some(condition);
        }
    }
}

impl<F> Lazy<F> {
    /// Used by generated save() for lazy HasOne/BelongsTo:
    /// - Pending(entity) => save entity and return PK as Value
    /// - NotLoaded(ById(v)) => use v directly
    /// - Loaded(entity) => use entity PK
    pub async fn resolve_relation_id_value<E, FMap>(
        &self,
        db: &mut E,
        id_to_value: FMap,
    ) -> Result<Option<Value>, CorrosionOrmError>
    where
        E: Executor,
        F: Repo<E> + Send + Unpin + Clone,
        F::PrimaryKey: From<Value>,
        FMap: Fn(&F) -> Value,
    {
        match &self.step {
            LazyStep::NotLoaded(Some(LazyCondition::ById(v))) => Ok(Some(v.clone())),
            LazyStep::NotLoaded(None) => Ok(None),
            LazyStep::Pending(entity) => {
                let saved = entity.clone().save(db).await?;
                Ok(Some(id_to_value(&saved)))
            }
            LazyStep::Loaded(entity) => Ok(Some(id_to_value(entity))),
        }
    }

    /// Attempts to retrieve the relation ID value synchronously.
    ///
    /// Returns `Some(Value)` if the relation is already loaded or has an ID set.
    /// Returns `None` if the relation is `Pending` (needs saving) or has no ID.
    pub fn get_id_value_sync<FMap>(&self, id_to_value: FMap) -> Option<Value>
    where
        FMap: Fn(&F) -> Value,
    {
        match &self.step {
            LazyStep::NotLoaded(Some(LazyCondition::ById(v))) => Some(v.clone()),
            LazyStep::NotLoaded(None) => None,
            LazyStep::Pending(_) => None,
            LazyStep::Loaded(entity) => Some(id_to_value(entity)),
        }
    }

    pub async fn load<E: Executor>(&mut self, db: &mut E) -> Result<&mut F, CorrosionOrmError>
    where
        F: Repo<E> + Send + Unpin + Clone + FromRowDb + crate::schema::table::TableSchema,
        F::PrimaryKey: From<Value>,
    {
        if let LazyStep::Loaded(ref mut f) = self.step {
            return Ok(f);
        }

        if let LazyStep::Pending(_) = self.step
            && let LazyStep::Pending(f) =
                std::mem::replace(&mut self.step, LazyStep::NotLoaded(None))
        {
            self.step = LazyStep::Loaded(f);
            if let LazyStep::Loaded(ref mut loaded) = self.step {
                return Ok(loaded);
            }
        }

        let loaded_entity = match &self.step {
            LazyStep::NotLoaded(Some(condition)) => match condition {
                LazyCondition::ById(id_val) => {
                    let id = F::PrimaryKey::from(id_val.clone());
                    F::get_by_id(id, db)
                        .await?
                        .ok_or(CorrosionOrmError::DriverError(DriverError::NotFound))?
                }
            },
            LazyStep::NotLoaded(None) => {
                return Err(CorrosionOrmError::DriverError(DriverError::NotFound));
            }
            LazyStep::Pending(_) | LazyStep::Loaded(_) => unreachable!(),
        };

        self.step = LazyStep::Loaded(loaded_entity);

        if let LazyStep::Loaded(ref mut f) = self.step {
            Ok(f)
        } else {
            unreachable!()
        }
    }
}
