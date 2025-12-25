pub struct Config {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub database_pool_size: u32,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            server_host: std::env::var("HOST").unwrap_or("127.0.0.1".to_string()),
            database_pool_size: std::env::var("DB_POOL_SIZE")
                .ok()
                .and_then(|size| size.parse().ok())
                .unwrap_or(5),
            server_port: std::env::var("PORT")
                .ok()
                .and_then(|port| port.parse().ok())
                .unwrap_or(3000),
        }
    }
}
