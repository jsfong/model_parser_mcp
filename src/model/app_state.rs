use std::sync::{Arc, LazyLock};

use crate::model::cubs_model::ModelData;
use crate::model::database_util::connect_to_db;
use crate::model::element_graph::ElementGraph;
use quick_cache::sync::Cache;
const CACHE_SIZE: usize = 2;

// pub static APP_STATE: LazyLock<AppState> = LazyLock::new(|| {
//     tokio::runtime::Runtime::new().unwrap().block_on(AppState::new())
// });

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

        // Model Cache
        let model_cache: Arc<Cache<String, ModelData>> = Arc::new(Cache::new({
            std::env::var("CACHE_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(CACHE_SIZE)
        }));

        // Graph Cache
        let graph_cache: Arc<Cache<String, ElementGraph>> = Arc::new(Cache::new({
            std::env::var("CACHE_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(CACHE_SIZE)
        }));

        AppState {
            pg_pool,
            model_cache: QuickCache { data: model_cache },
            graph_cache: QuickCache { data: graph_cache },
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

    // pub fn global() -> &'static AppState {
    //     &APP_STATE
    // }
}

#[derive(Clone, Debug)]
pub struct QuickCache<T> {
    pub data: Arc<Cache<String, T>>,
}

impl<T> QuickCache<T> {
    pub fn get(&self, key: &str, version: &str) -> Option<T>
    where
        T: Clone,
    {
        println!(
            "[QuickCache]Retrieving from cache with key: {} and version:{} ",
            key, version
        );
        let key = format!("{}-{}", key, version);
        self.data.get(&key)
    }

    pub fn insert(&self, key: &str, version: &str, value: &T)
    where
        T: Clone,
    {
        println!(
            "[QuickCache] insert into cache with key: {} and version:{} of capacity: {}",
             key, version, self.data.capacity()
        );
        let key = format!("{}-{}", key, version);
        self.data.insert(key.to_string(), value.clone());
    }

  
}
