use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub cache: CacheConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub capacity: usize,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(
        Self {
            database: DatabaseConfig {
                url: env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "sqlite:data/tao_database.db".to_string()),
            },
            server: ServerConfig {
                host: env::var("SERVER_HOST")
                    .unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "3000".to_string())
                    .parse()
                    .unwrap_or(3000),
            },
            cache: CacheConfig {
                capacity: env::var("CACHE_CAPACITY")
                    .unwrap_or_else(|_| "1000".to_string())
                    .parse()
                    .unwrap_or(1000),
            },
        })
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}