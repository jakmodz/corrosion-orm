use std::{hash::Hash, time::Duration};

#[cfg(not(feature = "cache"))]
use std::marker::PhantomData;
#[cfg(feature = "cache")]
use std::{collections::HashMap, sync::RwLock};

use crate::query::query_type::QueryContext;

/// Model contract used by generated repositories to access tiered caches.
pub trait CacheModel: Sized + Send + Sync + 'static {
    type PrimaryKey: Clone + Eq + Hash + Send + Sync + 'static;

    fn cache_id(&self) -> Self::PrimaryKey;

    fn entity_cache() -> &'static TieredEntityCache<(usize, Self::PrimaryKey), Self>;
    fn query_cache() -> &'static TieredQueryCache<String, Vec<Self::PrimaryKey>>;
}

/// Builds a deterministic cache key for a SQL query + bind values.
pub fn build_query_key<T>(ctx: &QueryContext) -> String {
    format!(
        "{}|{}|{:?}",
        std::any::type_name::<T>(),
        ctx.sql,
        ctx.values
    )
}

/// Builds a per-executor scope id used to isolate static caches between connections/tests.
pub fn scope_id<E: crate::Executor>(executor: &E) -> usize {
    executor.cache_scope()
}

#[inline]
fn scoped_query_key(scope: usize, query_key: &str) -> String {
    format!("{}|{}", scope, query_key)
}

/// Returns entity by ID from the unified cache.
#[cfg(feature = "cache")]
pub fn get_entity<T>(scope: usize, id: &T::PrimaryKey) -> Option<T>
where
    T: CacheModel + Clone,
{
    T::entity_cache().get(&(scope, id.clone()))
}

/// Returns entity by ID from the unified cache.
#[cfg(not(feature = "cache"))]
pub fn get_entity<T>(_scope: usize, _id: &T::PrimaryKey) -> Option<T>
where
    T: CacheModel,
{
    None
}

/// Inserts entity into L1 + L2 cache.
#[cfg(feature = "cache")]
pub async fn put_entity<T>(scope: usize, entity: &T)
where
    T: CacheModel + Clone,
{
    T::entity_cache()
        .insert((scope, entity.cache_id()), entity.clone())
        .await;
}

/// Inserts entity into L1 + L2 cache.
#[cfg(not(feature = "cache"))]
pub async fn put_entity<T>(_scope: usize, _entity: &T)
where
    T: CacheModel,
{
}

/// Invalidates entity cache key in both levels.
#[cfg(feature = "cache")]
pub async fn invalidate_entity<T>(scope: usize, id: &T::PrimaryKey)
where
    T: CacheModel + Clone,
{
    T::entity_cache().invalidate(&(scope, id.clone())).await;
}

/// Invalidates entity cache key in both levels.
#[cfg(not(feature = "cache"))]
pub async fn invalidate_entity<T>(_scope: usize, _id: &T::PrimaryKey)
where
    T: CacheModel,
{
}

/// Clears all query cache entries for this model.
pub fn invalidate_queries<T>()
where
    T: CacheModel,
{
    T::query_cache().invalidate_all();
}

/// Gets cached query index (query key -> ids).
pub fn get_query_ids<T>(scope: usize, query_key: &str) -> Option<Vec<T::PrimaryKey>>
where
    T: CacheModel,
{
    T::query_cache().get(&scoped_query_key(scope, query_key))
}

/// Stores query index (query key -> ids).
pub async fn put_query_ids<T>(scope: usize, query_key: String, ids: Vec<T::PrimaryKey>)
where
    T: CacheModel,
{
    T::query_cache()
        .insert(scoped_query_key(scope, &query_key), ids)
        .await;
}

/// Unified two-level cache for entities.
///
/// - L1: in-process HashMap (very fast)
/// - L2: moka cache (when `cache` feature is enabled)
#[cfg(feature = "cache")]
pub struct TieredEntityCache<K, V> {
    l1: RwLock<HashMap<K, V>>,
    l2: moka::future::Cache<K, V>,
}

#[cfg(feature = "cache")]
impl<K, V> TieredEntityCache<K, V>
where
    K: Eq + Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new(capacity: usize, ttl: Duration, tti: Duration) -> Self {
        Self {
            l1: RwLock::new(HashMap::new()),
            l2: moka::future::CacheBuilder::new(capacity)
                .time_to_live(ttl)
                .time_to_idle(tti)
                .build(),
        }
    }

    pub fn invalidate_all(&self)
    where
        K: Eq + Hash,
    {
        self.l1
            .write()
            .expect("tiered entity cache L1 write lock poisoned")
            .clear();
        self.l2.invalidate_all();
    }
}

#[cfg(feature = "cache")]
impl<K, V> TieredEntityCache<K, V>
where
    K: Clone + Eq + Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn get(&self, key: &K) -> Option<V> {
        if let Some(v) = self
            .l1
            .read()
            .expect("tiered entity cache L1 read lock poisoned")
            .get(key)
            .cloned()
        {
            return Some(v);
        }

        if let Some(v) = self.l2.get(key) {
            self.l1
                .write()
                .expect("tiered entity cache L1 write lock poisoned")
                .insert(key.clone(), v.clone());
            return Some(v);
        }

        None
    }

    pub async fn insert(&self, key: K, value: V) {
        self.l1
            .write()
            .expect("tiered entity cache L1 write lock poisoned")
            .insert(key.clone(), value.clone());
        self.l2.insert(key, value).await;
    }

    pub async fn invalidate(&self, key: &K) {
        self.l1
            .write()
            .expect("tiered entity cache L1 write lock poisoned")
            .remove(key);
        self.l2.invalidate(key).await;
    }
}

#[cfg(not(feature = "cache"))]
pub struct TieredEntityCache<K, V>(PhantomData<(K, V)>);

#[cfg(not(feature = "cache"))]
impl<K, V> TieredEntityCache<K, V> {
    pub fn new(_capacity: usize, _ttl: Duration, _tti: Duration) -> Self {
        Self(PhantomData)
    }

    pub fn get(&self, _key: &K) -> Option<V> {
        None
    }

    pub async fn insert(&self, _key: K, _value: V) {}

    pub async fn invalidate(&self, _key: &K) {}

    pub fn invalidate_all(&self) {}
}

/// Unified two-level cache for query index results.
#[cfg(feature = "cache")]
pub struct TieredQueryCache<K, V> {
    l1: RwLock<HashMap<K, V>>,
    l2: moka::future::Cache<K, V>,
}

#[cfg(feature = "cache")]
impl<K, V> TieredQueryCache<K, V>
where
    K: Eq + Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            l1: RwLock::new(HashMap::new()),
            l2: moka::future::CacheBuilder::new(capacity).build(),
        }
    }

    pub fn invalidate_all(&self)
    where
        K: Eq + Hash,
    {
        self.l1
            .write()
            .expect("tiered query cache L1 write lock poisoned")
            .clear();
        self.l2.invalidate_all();
    }
}

#[cfg(feature = "cache")]
impl<K, V> TieredQueryCache<K, V>
where
    K: Clone + Eq + Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn get(&self, key: &K) -> Option<V> {
        if let Some(v) = self
            .l1
            .read()
            .expect("tiered query cache L1 read lock poisoned")
            .get(key)
            .cloned()
        {
            return Some(v);
        }

        if let Some(v) = self.l2.get(key) {
            self.l1
                .write()
                .expect("tiered query cache L1 write lock poisoned")
                .insert(key.clone(), v.clone());
            return Some(v);
        }

        None
    }

    pub async fn insert(&self, key: K, value: V) {
        self.l1
            .write()
            .expect("tiered query cache L1 write lock poisoned")
            .insert(key.clone(), value.clone());
        self.l2.insert(key, value).await;
    }

    pub async fn invalidate(&self, key: &K) {
        self.l1
            .write()
            .expect("tiered query cache L1 write lock poisoned")
            .remove(key);
        self.l2.invalidate(key).await;
    }
}

#[cfg(not(feature = "cache"))]
pub struct TieredQueryCache<K, V>(PhantomData<(K, V)>);

#[cfg(not(feature = "cache"))]
impl<K, V> TieredQueryCache<K, V> {
    pub fn new(_capacity: usize) -> Self {
        Self(PhantomData)
    }

    pub fn get(&self, _key: &K) -> Option<V> {
        None
    }

    pub async fn insert(&self, _key: K, _value: V) {}

    pub async fn invalidate(&self, _key: &K) {}

    pub fn invalidate_all(&self) {}
}
