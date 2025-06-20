use std::sync::Arc;
use crate::{
    database::TaoDatabase,
    tao_interface::TaoInterface,
    config::Config,
};

#[derive(Clone)]
pub struct AppState {
    pub tao_interface: TaoInterface,
    pub config: Config,
}

impl AppState {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        // Initialize database
        let database = TaoDatabase::new(&config.database.url, config.cache.capacity).await?;
        database.init().await?;
        let database = Arc::new(database);
        
        // Initialize unified TAO interface (contains EntityContext internally)
        let tao_interface = TaoInterface::new(database);

        Ok(Self {
            tao_interface,
            config,
        })
    }
}