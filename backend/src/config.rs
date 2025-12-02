use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub solana_network: String,
    pub solana_rpc_url: String,
    pub program_id: String,
    pub ipfs_api_url: String,
    pub pinata_api_key: String,
    pub pinata_secret_key: String,
    pub pinata_gateway_url: String,
    pub database_url: String,
    pub redis_url: String,
    pub api_port: u16,
    pub api_host: String,
    pub local_llm_url: String,
    pub llm_model: String,
    pub inference_nodes: u32,
    pub log_level: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            solana_network: "devnet".to_string(),
            solana_rpc_url: "https://api.devnet.solana.com".to_string(),
            program_id: String::new(),
            ipfs_api_url: "http://localhost:5001".to_string(),
            pinata_api_key: String::new(),
            pinata_secret_key: String::new(),
            pinata_gateway_url: "https://gateway.pinata.cloud/ipfs/".to_string(),
            database_url: "postgresql://user:password@localhost:5432/daollm".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            api_port: 8000,
            api_host: "0.0.0.0".to_string(),
            local_llm_url: "http://localhost:8001".to_string(),
            llm_model: "llama3".to_string(),
            inference_nodes: 3,
            log_level: "INFO".to_string(),
        }
    }
}

impl Settings {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let mut settings = config::Config::builder();

        // Load from .env file if exists
        dotenv::dotenv().ok();

        // Try to load from environment variables
        settings = settings
            .add_source(config::Environment::with_prefix("").separator("_"));

        let settings = settings.build()?;
        settings.try_deserialize()
    }
}

