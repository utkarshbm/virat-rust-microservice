use cache::redis_client::RedisCache;
use std::sync::Arc;

pub async fn logout(cache: Arc<RedisCache>, uuid: &str) -> Result<(), String> {
    cache.delete_token(uuid).await.map_err(|e| e.to_string())
}
