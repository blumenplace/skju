use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use dotenvy::var;


static VALKEY_URL: &str = "VALKEY_URL";

static VALKEY_URL_DEFAULT: &str = "redis://cache:6379";

#[derive(Clone)]
pub struct Cache {
    manager: ConnectionManager,
}

impl Cache {

    pub async fn from_env() -> redis::RedisResult<Self> {
        let redis_url = var(VALKEY_URL).unwrap_or_else(|_| VALKEY_URL_DEFAULT.into());
        let cache = Cache::new(&redis_url).await?;
        Ok(cache)
    }

    pub async fn new(url: &str) -> redis::RedisResult<Self> {
        let client = redis::Client::open(url)?;
        let manager = ConnectionManager::new(client).await?;
        Ok(Self { manager })
    }

    pub async fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        let mut conn = self.manager.clone();
        let val: Option<Vec<u8>> = conn.get(key).await.ok().flatten();
        val.and_then(|b| serde_json::from_slice(&b).ok())
    }

    pub async fn set<T: serde::Serialize>(&self, key: &str, value: &T, ttl_secs: u64) {
        let mut conn = self.manager.clone();
        if let Ok(serialized) = serde_json::to_vec(value) {
            let _: redis::RedisResult<()> = conn.set_ex(key, serialized, ttl_secs).await;
        }
    }
}