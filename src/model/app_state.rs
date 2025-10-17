use std::sync::Arc;
use std::time::Duration;

use mini_moka::sync::Cache;

use crate::model::cubs_model::ModelData;
use crate::model::database_util::connect_to_db;
use crate::model::element_graph::ElementGraph;
// use quick_cache::sync::Cache;
const CACHE_SIZE: usize = 2;

#[derive(Clone, Debug)]
pub struct AppState {
    pg_pool: sqlx::Pool<sqlx::Postgres>,
    model_cache: QuickCache<ModelData>,
    graph_cache: QuickCache<ElementGraph>,
}

impl AppState {
    pub async fn new() -> Self {
        // DB pool
        let pg_pool = connect_to_db().await;

        // Moka Cache
        let moka_model_cache: Cache<String, Arc<ModelData>> = Cache::builder()
            .max_capacity(CACHE_SIZE as u64)
            .time_to_live(Duration::from_secs(3600))
            .time_to_idle(Duration::from_secs(600))
            .build();

        let moka_graph_cache: Cache<String, Arc<ElementGraph>> = Cache::builder()
            .max_capacity(CACHE_SIZE as u64)
            .time_to_live(Duration::from_secs(3600))
            .time_to_idle(Duration::from_secs(600))
            .build();

        AppState {
            pg_pool,
            model_cache: QuickCache {
                data: moka_model_cache,
            },
            graph_cache: QuickCache {
                data: moka_graph_cache,
            },
        }
    }

    pub fn get_pg_pool_ref(&self) -> &sqlx::Pool<sqlx::Postgres> {
        &self.pg_pool
    }

    pub fn get_model_cache(&self) -> QuickCache<ModelData> {
        self.model_cache.clone()
    }

    pub fn get_graph_cache(&self) -> QuickCache<ElementGraph> {
        self.graph_cache.clone()
    }
}

#[derive(Debug)]
pub struct QuickCache<T>
where
    T: Send + Sync + 'static,
{
    data: Cache<String, Arc<T>>,
}

impl<T> Clone for QuickCache<T>
where
    T: Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl<T> QuickCache<T>
where
    T: Send + Sync + 'static,
{
    pub fn new(capacity: u64) -> Self {
        Self {
            data: Cache::new(capacity),
        }
    }

    pub fn get_ref(&self, key: &str, version: &str) -> Option<Arc<T>>
    where
        T: Clone + Send + Sync + 'static,
    {
        println!(
            "[QuickCache]Retrieving from cache with key: {} and version:{} ",
            key, version
        );
        let key = format!("{}-{}", key, version);
        self.data.get(&key).map(|arc| Arc::clone(&arc))
    }

    pub fn insert(&self, key: &str, version: &str, value: &T)
    where
        T: Clone + Send + Sync + 'static,
    {
        println!(
            "[QuickCache] insert into cache with key: {} and version:{} of capacity: {}",
            key,
            version,
            self.data.entry_count()
        );
        let key = format!("{}-{}", key, version);
        self.data.insert(key.to_string(), Arc::new(value.clone()));
    }
}

