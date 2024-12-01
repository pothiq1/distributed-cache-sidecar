use async_trait::async_trait;
use bytes::Bytes;
use redis::AsyncCommands;

#[async_trait]
pub trait Fallback: Send + Sync {
    async fn get(&self, key: &str) -> Option<Bytes>;
}

pub struct RedisFallback {
    client: redis::aio::MultiplexedConnection,
}

impl RedisFallback {
    pub async fn new(redis_url: &str) -> Self {
        let client = redis::Client::open(redis_url).expect("Invalid Redis URL");
        let connection = client
            .get_multiplexed_async_connection()
            .await
            .expect("Failed to connect to Redis");
        Self { client: connection }
    }
}

#[async_trait]
impl Fallback for RedisFallback {
    async fn get(&self, key: &str) -> Option<Bytes> {
        let mut conn = self.client.clone();
        let result: redis::RedisResult<Vec<u8>> = conn.get(key).await;
        match result {
            Ok(data) => Some(Bytes::from(data)),
            Err(_) => None,
        }
    }
}
