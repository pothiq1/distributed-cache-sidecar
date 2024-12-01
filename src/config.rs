//config.rs

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub max_memory: usize,
    pub default_ttl: u64,
    pub frequency_threshold: u64,
    pub replication_factor: usize,
    pub local_address: String,
    pub redis_url: String,
    pub enable_monitoring: bool,
    pub tls_cert_path: Option<String>,
    pub tls_key_path: Option<String>,
    pub jwt_secret: Option<String>,
    pub transaction_timeout: u64,
    pub enable_transactions: bool,
}

impl Config {
    pub fn load() -> Self {
        // Load from environment variables or a configuration file
        // For simplicity, we'll load from environment variables here
        Self {
            max_memory: std::env::var("MAX_MEMORY")
                .unwrap_or_else(|_| "104857600".to_string())
                .parse()
                .unwrap(),
            default_ttl: std::env::var("DEFAULT_TTL")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .unwrap(),
            frequency_threshold: std::env::var("FREQUENCY_THRESHOLD")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .unwrap(),
            replication_factor: std::env::var("REPLICATION_FACTOR")
                .unwrap_or_else(|_| "2".to_string())
                .parse()
                .unwrap(),
            local_address: std::env::var("LOCAL_ADDRESS")
                .unwrap_or_else(|_| "0.0.0.0:50051".to_string()),
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1/".to_string()),
            enable_monitoring: std::env::var("ENABLE_MONITORING")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap(),
            tls_cert_path: std::env::var("TLS_CERT_PATH").ok(),
            tls_key_path: std::env::var("TLS_KEY_PATH").ok(),
            jwt_secret: std::env::var("JWT_SECRET").ok(),
            transaction_timeout: std::env::var("TRANSACTION_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap(),
            enable_transactions: std::env::var("ENABLE_TRANSACTIONS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap(),
        }
    }
}
