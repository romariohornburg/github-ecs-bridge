use thiserror::Error;

#[derive(Clone, Debug)]
pub struct Settings {
    /// GitHub webhook secret for HMAC-SHA256 signature validation.
    pub webhook_secret: String,
    /// Elasticsearch base URL (e.g. https://localhost:9200).
    pub elasticsearch_url: String,
    /// Pre-encoded Elasticsearch API key (the full base64 `id:api_key` string).
    pub elasticsearch_api_key: String,
    /// Target index or data stream name.
    pub elasticsearch_index: String,
    /// TCP bind address.
    pub listen_addr: String,
}

impl Settings {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok();

        let webhook_secret = std::env::var("WEBHOOK_SECRET")
            .map_err(|_| ConfigError::MissingEnv("WEBHOOK_SECRET"))?;
        let elasticsearch_url = std::env::var("ELASTICSEARCH_URL")
            .map_err(|_| ConfigError::MissingEnv("ELASTICSEARCH_URL"))?;
        let elasticsearch_api_key = std::env::var("ELASTICSEARCH_API_KEY")
            .map_err(|_| ConfigError::MissingEnv("ELASTICSEARCH_API_KEY"))?;
        let elasticsearch_index = std::env::var("ELASTICSEARCH_INDEX")
            .unwrap_or_else(|_| "logs-github.audit-default".into());
        let listen_addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:3001".into());

        Ok(Self {
            webhook_secret,
            elasticsearch_url,
            elasticsearch_api_key,
            elasticsearch_index,
            listen_addr,
        })
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnv(&'static str),
}
