use crate::errors::app_error::AppError;
use cedar_policy::{Entities, Schema};
use moka::future::Cache;
use redis::{AsyncCommands, Client as RedisClient};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, warn};

const DEFAULT_LOCAL_CACHE_SIZE: u64 = 1000;
const DEFAULT_LOCAL_TTL_SECS: u64 = 300; // 5 minutes
const DEFAULT_REDIS_TTL_SECS: u64 = 3600; // 1 hour

#[derive(Clone)]
pub struct CacheService {
    redis_client: RedisClient,
    local_cache: Cache<String, String>,
    schema: Arc<RwLock<Schema>>,
}

impl CacheService {
    pub fn new(redis_client: RedisClient, schema: Schema) -> Self {
        let local_cache = Cache::builder()
            .max_capacity(DEFAULT_LOCAL_CACHE_SIZE)
            .time_to_live(Duration::from_secs(DEFAULT_LOCAL_TTL_SECS))
            .build();

        Self {
            redis_client,
            local_cache,
            schema: Arc::new(RwLock::new(schema)),
        }
    }

    pub async fn get_entities(&self, cache_key: &str) -> Result<Option<Entities>, AppError> {
        if let Some(cache_value) = self.local_cache.get(cache_key).await {
            let schema = self.schema.read().await;
            let entities = Entities::from_json_str(&cache_value, Some(&*schema))?;
            return Ok(Some(entities));
        }

        match self.get_from_redis(cache_key).await {
            Ok(Some(cache_value)) => {
                let schema = self.schema.read().await;
                let entities = Entities::from_json_str(&cache_value, Some(&*schema))?;
                self.local_cache
                    .insert(cache_key.to_string(), cache_value)
                    .await;
                debug!("Loaded entity [CacheKey:{}] from Redis into the local cache", cache_key);
                Ok(Some(entities))
            }
            Ok(None) => {
                debug!("Entity [CacheKey: {}] not found in Redis", cache_key);
                Ok(None)
            }
            Err(e) => {
                warn!("Cannot get user entity from Redis: {}", e);
                Ok(None)
            }
        }
    }

    pub async fn cache_entities(
        &self,
        cache_key: &str,
        entities: Entities,
    ) -> Result<(), AppError> {
        // Parse the entity to validate its format, then keep it in the local cache
        let schema = self.schema.read().await;
        let parsed_entities = Entities::from_entities(entities, Some(&schema))?;
        let cache_v = parsed_entities.to_json_value()?.to_string();
        self.set_cache(cache_key, cache_v.as_str(), None).await?;
        Ok(())
    }

    pub async fn invalidate_user_entities(&self, cache_key: String) -> Result<(), AppError> {
        // Remove from the local cache
        self.local_cache.invalidate(&cache_key).await;

        // Remove from Redis
        if let Err(e) = self.delete_from_redis(&cache_key).await {
            warn!("Cannot delete entity [{}] from Redis: {}", cache_key, e);
        }

        debug!("User entity invalidated");
        Ok(())
    }

    pub async fn update_schema(&self, new_schema: Schema) {
        let mut schema = self.schema.write().await;
        *schema = new_schema;
        // Clear the local cache; a schema change can affect entity parsing
        self.local_cache.invalidate_all();
        debug!("Cache schema updated, local cache cleared");
    }

    // Cache stats
    pub async fn get_cache_stats(&self) -> CacheStats {
        CacheStats {
            local_cache_size: self.local_cache.entry_count(),
            local_cache_capacity: DEFAULT_LOCAL_CACHE_SIZE,
        }
    }

    pub async fn get_cache(&self, cache_key: &str) -> Result<Option<String>, AppError> {
        if let Some(cache) = self.local_cache.get(cache_key).await {
            return Ok(Some(cache));
        }

        if let Some(cache) = self.get_from_redis(cache_key).await? {
            return Ok(Some(cache));
        }

        Ok(None)
    }

    /*
       Unless there is a good reason not to, always set a TTL.
       ttl_secs should be larger than DEFAULT_LOCAL_TTL_SECS (the local cache TTL)
    */
    pub async fn set_cache(
        &self,
        cache_key: &str,
        cache_value: &str,
        ttl_secs: Option<u64>,
    ) -> Result<(), AppError> {
        let redis_result = self.set_to_redis(&cache_key, cache_value, ttl_secs).await;
        self.local_cache
            .insert(cache_key.to_string(), cache_value.to_string())
            .await;

        match redis_result {
            Ok(_) => {
                debug!("Entity [{}] cached", cache_key);
                Ok(())
            }
            Err(e) => {
                error!("Cannot cache entity [{}] in Redis: {}", cache_key, e);
                Ok(())
            }
        }
    }

    async fn get_from_redis(&self, key: &str) -> Result<Option<String>, AppError> {
        let mut conn = self.redis_client.get_multiplexed_tokio_connection().await?;
        let cache: Option<String> = conn.get(key).await?;
        Ok(cache)
    }

    async fn set_to_redis(
        &self,
        key: &str,
        value: &str,
        ttl_secs: Option<u64>,
    ) -> Result<(), AppError> {
        let mut conn = self.redis_client.get_multiplexed_tokio_connection().await?;
        if let Some(ttl_secs) = ttl_secs {
            debug!("Cache key: {key}, ttl_secs: {ttl_secs}");
            let _: () = conn.set_ex(key, value, ttl_secs).await?;
            return Ok(());
        }

        let _: () = conn.set(key, value).await?;
        Ok(())
    }

    async fn delete_from_redis(&self, key: &str) -> Result<(), AppError> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let _: () = conn.del(key).await?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct CacheStats {
    pub local_cache_size: u64,
    pub local_cache_capacity: u64,
}
