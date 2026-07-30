use redis::AsyncCommands;

pub struct RedisCache {
    client: redis::Client,
}

impl RedisCache {
    pub fn new(url: &str) -> Self {
        Self { client: redis::Client::open(url).unwrap() }
    }

    pub async fn store_token(&self, uuid: &str, token: &str, ttl_secs: usize) -> redis::RedisResult<()> {
        let mut conn = self.client.get_async_connection().await?;
        conn.set_ex(format!("session:{uuid}"), token, ttl_secs).await
    }

    pub async fn get_token(&self, uuid: &str) -> redis::RedisResult<Option<String>> {
        let mut conn = self.client.get_async_connection().await?;
        conn.get(format!("session:{uuid}")).await
    }

    pub async fn delete_token(&self, uuid: &str) -> redis::RedisResult<()> {
        let mut conn = self.client.get_async_connection().await?;
        conn.del(format!("session:{uuid}")).await
    }
}